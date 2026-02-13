"""CalculiX Rust Solver Web Interface and Validation API."""

from __future__ import annotations

import html
import logging
import subprocess
import time
from datetime import datetime
from pathlib import Path
from typing import Any, List

import schemas
from database import (
    Example,
    KPI,
    TestCase,
    TestModule,
    TestRun,
    ValidationResult,
    get_db,
    init_db,
)
from fastapi import Depends, FastAPI, File, HTTPException, Request, UploadFile
from fastapi.responses import FileResponse, HTMLResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates
from nastran_converter import BdfToInpConverter, Op2ResultReader, is_pynastran_available
from sqlalchemy import desc, func
from sqlalchemy.orm import Session

# Configuration
PROJECT_ROOT = Path(__file__).resolve().parents[1]
FIXTURES_DIR = PROJECT_ROOT / "tests" / "fixtures" / "solver"
VALIDATION_DIR = PROJECT_ROOT / "validation" / "solver"
CCX_CLI_BIN = PROJECT_ROOT / "target" / "release" / "ccx-cli"
TEMPLATES_DIR = Path(__file__).resolve().parent / "templates"
STATIC_DIR = Path(__file__).resolve().parent / "static"
UPLOADS_DIR = Path(__file__).resolve().parent / "uploads"

# Logging
logging.basicConfig(level=logging.INFO)
LOGGER = logging.getLogger(__name__)

# Initialize validation database
VALIDATION_ENABLED = True
try:
    init_db()
    LOGGER.info("Validation database initialized")
except Exception as exc:
    VALIDATION_ENABLED = False
    LOGGER.warning("Could not initialize validation database: %s", exc)

# FastAPI app
app = FastAPI(
    title="CalculiX Rust Solver Web Interface",
    description="Run ccx-cli workflows and expose validation dashboard/API endpoints.",
    version="0.1.0",
)

# Mount static files
if STATIC_DIR.exists():
    app.mount("/static", StaticFiles(directory=STATIC_DIR), name="static")

# Templates
templates = Jinja2Templates(directory=TEMPLATES_DIR)


def _run_command(cmd: list[str], timeout: int = 300) -> dict[str, Any]:
    """Run a command and return the result."""
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
            cwd=PROJECT_ROOT,
        )
        return {
            "returncode": result.returncode,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "success": result.returncode == 0,
        }
    except subprocess.TimeoutExpired:
        return {
            "returncode": -1,
            "stdout": "",
            "stderr": f"Command timed out after {timeout} seconds",
            "success": False,
        }
    except Exception as exc:
        return {
            "returncode": -1,
            "stdout": "",
            "stderr": str(exc),
            "success": False,
        }


def _ensure_validation_enabled() -> None:
    if not VALIDATION_ENABLED:
        raise HTTPException(
            status_code=503,
            detail="Validation database is unavailable. Check SQLAlchemy/database configuration.",
        )


# ============================================================================
# Existing Webapp Routes
# ============================================================================


@app.get("/", response_class=HTMLResponse)
async def index(request: Request):
    """Home page with overview and quick actions."""
    fixtures = list(FIXTURES_DIR.glob("*.inp")) if FIXTURES_DIR.exists() else []
    ref_files = list(VALIDATION_DIR.glob("*.dat.ref")) if VALIDATION_DIR.exists() else []

    context = {
        "request": request,
        "num_fixtures": len(fixtures),
        "num_references": len(ref_files),
        "ccx_cli_exists": CCX_CLI_BIN.exists(),
        "ccx_cli_path": str(CCX_CLI_BIN),
    }
    return templates.TemplateResponse("index.html", context)


@app.get("/health")
async def health():
    """Health check endpoint."""
    return {
        "status": "healthy",
        "ccx_cli_exists": CCX_CLI_BIN.exists(),
        "ccx_cli_path": str(CCX_CLI_BIN),
        "fixtures_dir": str(FIXTURES_DIR),
        "validation_dir": str(VALIDATION_DIR),
        "validation_enabled": VALIDATION_ENABLED,
    }


@app.get("/fixtures", response_class=HTMLResponse)
async def list_fixtures(request: Request):
    """List all available test fixtures."""
    if not FIXTURES_DIR.exists():
        raise HTTPException(status_code=404, detail=f"Fixtures directory not found: {FIXTURES_DIR}")

    fixtures = []
    for inp_file in sorted(FIXTURES_DIR.glob("*.inp")):
        ref_file = VALIDATION_DIR / f"{inp_file.stem}.dat.ref"
        fixtures.append(
            {
                "name": inp_file.name,
                "stem": inp_file.stem,
                "has_reference": ref_file.exists() if VALIDATION_DIR.exists() else False,
                "size": inp_file.stat().st_size,
            }
        )

    context = {
        "request": request,
        "fixtures": fixtures,
        "total": len(fixtures),
    }
    return templates.TemplateResponse("fixtures.html", context)


@app.get("/fixtures/{fixture_name}")
async def view_fixture(fixture_name: str):
    """View a specific fixture file."""
    inp_file = FIXTURES_DIR / fixture_name
    if not inp_file.exists():
        raise HTTPException(status_code=404, detail=f"Fixture not found: {fixture_name}")

    try:
        content = inp_file.read_text()
        return {
            "name": fixture_name,
            "content": content,
            "lines": len(content.splitlines()),
            "size": inp_file.stat().st_size,
        }
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"Error reading fixture: {exc}")


@app.post("/analyze/{fixture_name}")
async def analyze_fixture(fixture_name: str):
    """Analyze a fixture using ccx-cli analyze."""
    if not CCX_CLI_BIN.exists():
        raise HTTPException(status_code=503, detail=f"ccx-cli not found: {CCX_CLI_BIN}")

    inp_file = FIXTURES_DIR / fixture_name
    if not inp_file.exists():
        raise HTTPException(status_code=404, detail=f"Fixture not found: {fixture_name}")

    start_time = time.time()
    result = _run_command([str(CCX_CLI_BIN), "analyze", str(inp_file)])
    duration = time.time() - start_time

    return {
        "fixture": fixture_name,
        "success": result["success"],
        "returncode": result["returncode"],
        "stdout": result["stdout"],
        "stderr": result["stderr"],
        "duration_seconds": round(duration, 3),
    }


@app.post("/validate")
async def run_validation(request: Request, fixtures_dir: str | None = None):
    """Run full validation suite."""
    if not CCX_CLI_BIN.exists():
        raise HTTPException(status_code=503, detail=f"ccx-cli not found: {CCX_CLI_BIN}")

    body_fixtures_dir = None
    if request.headers.get("content-type", "").startswith("application/json"):
        try:
            payload = await request.json()
            if isinstance(payload, dict):
                body_fixtures_dir = payload.get("fixtures_dir")
        except Exception:
            body_fixtures_dir = None

    effective_fixtures_dir = body_fixtures_dir or fixtures_dir
    target_dir = Path(effective_fixtures_dir) if effective_fixtures_dir else FIXTURES_DIR

    start_time = time.time()
    result = _run_command(
        [str(CCX_CLI_BIN), "validate", "--fixtures-dir", str(target_dir)],
        timeout=600,
    )
    duration = time.time() - start_time

    return {
        "fixtures_dir": str(target_dir),
        "success": result["success"],
        "returncode": result["returncode"],
        "stdout": result["stdout"],
        "stderr": result["stderr"],
        "duration_seconds": round(duration, 3),
    }


@app.get("/version")
async def get_version():
    """Get ccx-cli version."""
    if not CCX_CLI_BIN.exists():
        raise HTTPException(status_code=503, detail=f"ccx-cli not found: {CCX_CLI_BIN}")

    result = _run_command([str(CCX_CLI_BIN), "--version"])

    return {
        "success": result["success"],
        "version": result["stdout"].strip() if result["success"] else None,
        "error": result["stderr"] if not result["success"] else None,
    }


@app.get("/status", response_class=HTMLResponse)
async def status_page(request: Request):
    """Status page showing system information."""
    version_result = await get_version()

    fixtures = list(FIXTURES_DIR.glob("*.inp")) if FIXTURES_DIR.exists() else []
    ref_files = list(VALIDATION_DIR.glob("*.dat.ref")) if VALIDATION_DIR.exists() else []

    matched = 0
    for fixture in fixtures:
        ref_file = VALIDATION_DIR / f"{fixture.stem}.dat.ref"
        if ref_file.exists():
            matched += 1

    context = {
        "request": request,
        "version": version_result.get("version", "Unknown"),
        "ccx_cli_path": str(CCX_CLI_BIN),
        "ccx_cli_exists": CCX_CLI_BIN.exists(),
        "fixtures_dir": str(FIXTURES_DIR),
        "validation_dir": str(VALIDATION_DIR),
        "num_fixtures": len(fixtures),
        "num_references": len(ref_files),
        "num_matched": matched,
    }
    return templates.TemplateResponse("status.html", context)


@app.get("/api/fixtures")
async def api_list_fixtures():
    """API endpoint to list fixtures."""
    if not FIXTURES_DIR.exists():
        return {"fixtures": [], "total": 0}

    fixtures = []
    for inp_file in sorted(FIXTURES_DIR.glob("*.inp")):
        ref_file = VALIDATION_DIR / f"{inp_file.stem}.dat.ref"
        fixtures.append(
            {
                "name": inp_file.name,
                "stem": inp_file.stem,
                "has_reference": ref_file.exists() if VALIDATION_DIR.exists() else False,
                "size": inp_file.stat().st_size,
            }
        )

    return {"fixtures": fixtures, "total": len(fixtures)}


@app.get("/run", response_class=HTMLResponse)
async def run_ccx_page(request: Request):
    """CCX CLI command execution page."""
    fixtures = []
    if FIXTURES_DIR.exists():
        for inp_file in sorted(FIXTURES_DIR.glob("*.inp")):
            fixtures.append(
                {
                    "name": inp_file.name,
                    "stem": inp_file.stem,
                }
            )

    context = {
        "request": request,
        "ccx_cli_exists": CCX_CLI_BIN.exists(),
        "ccx_cli_path": str(CCX_CLI_BIN),
        "fixtures": fixtures,
    }
    return templates.TemplateResponse("run_ccx.html", context)


@app.post("/solve/{fixture_name}")
async def solve_fixture(fixture_name: str):
    """Solve a fixture using ccx-cli solve."""
    if not CCX_CLI_BIN.exists():
        raise HTTPException(status_code=503, detail=f"ccx-cli not found: {CCX_CLI_BIN}")

    inp_file = FIXTURES_DIR / fixture_name
    if not inp_file.exists():
        raise HTTPException(status_code=404, detail=f"Fixture not found: {fixture_name}")

    start_time = time.time()
    result = _run_command([str(CCX_CLI_BIN), "solve", str(inp_file)])
    duration = time.time() - start_time

    return {
        "fixture": fixture_name,
        "success": result["success"],
        "returncode": result["returncode"],
        "stdout": result["stdout"],
        "stderr": result["stderr"],
        "duration_seconds": round(duration, 3),
    }


@app.get("/validation/dashboard", response_class=HTMLResponse)
async def validation_dashboard(request: Request, db: Session = Depends(get_db)):
    """Validation tracking dashboard (kept for compatibility)."""
    _ensure_validation_enabled()

    total_examples = db.query(Example).count()
    total_validations = db.query(ValidationResult).count()
    context = {
        "request": request,
        "total_examples": total_examples,
        "total_validations": total_validations,
        "validation_enabled": True,
    }
    return templates.TemplateResponse("validation_dashboard.html", context)


@app.get("/features", response_class=HTMLResponse)
async def features_page(request: Request):
    """Feature parity comparison page."""
    element_types_implemented = 6
    element_types_total = 40

    analysis_types_implemented = 4
    analysis_types_total = 16

    material_models_implemented = 1
    material_models_total = 10

    solver_backends = 2

    element_progress = (element_types_implemented / element_types_total) * 100
    analysis_progress = (analysis_types_implemented / analysis_types_total) * 100
    material_progress = (material_models_implemented / material_models_total) * 100

    overall_progress = int(
        element_progress * 0.4 + analysis_progress * 0.4 + material_progress * 0.2
    )

    context = {
        "request": request,
        "overall_progress": overall_progress,
        "element_types_implemented": element_types_implemented,
        "element_types_total": element_types_total,
        "analysis_types_implemented": analysis_types_implemented,
        "analysis_types_total": analysis_types_total,
        "material_models_implemented": material_models_implemented,
        "material_models_total": material_models_total,
        "solver_backends": solver_backends,
    }
    return templates.TemplateResponse("features.html", context)


# ============================================================================
# Merged Validation API Routes (from former crates/validation-api)
# ============================================================================


@app.get("/dashboard", response_class=HTMLResponse)
async def dashboard_page(request: Request, db: Session = Depends(get_db)):
    """Validation dashboard page."""
    _ensure_validation_enabled()

    latest_kpi = db.query(KPI).order_by(desc(KPI.date)).first()

    total_tests = db.query(TestCase).count()
    latest_runs = (
        db.query(TestRun)
        .join(TestCase)
        .group_by(TestCase.id)
        .with_entities(TestCase.id, func.max(TestRun.run_date).label("last_run"))
        .subquery()
    )

    passing_tests = (
        db.query(func.count(TestRun.id))
        .join(
            latest_runs,
            (TestRun.test_case_id == latest_runs.c.id)
            & (TestRun.run_date == latest_runs.c.last_run),
        )
        .filter(TestRun.passed == True)
        .scalar()
        or 0
    )

    total_examples = db.query(Example).count()
    examples_with_validations = (
        db.query(func.count(func.distinct(ValidationResult.example_id))).scalar() or 0
    )

    total_modules = db.query(TestModule).count()

    avg_test_time = (
        db.query(func.avg(TestRun.execution_time_ms))
        .filter(TestRun.execution_time_ms.isnot(None))
        .scalar()
        or 0.0
    )

    element_types = (
        db.query(Example.element_type)
        .filter(Example.element_type.isnot(None))
        .distinct()
        .all()
    )
    supported_elements = [et[0] for et in element_types if et[0]]

    stats = schemas.DashboardStats(
        total_tests=total_tests,
        passing_tests=passing_tests,
        failing_tests=total_tests - passing_tests,
        pass_rate=(passing_tests / total_tests * 100) if total_tests > 0 else 0.0,
        total_examples=total_examples,
        validated_examples=examples_with_validations,
        total_modules=total_modules,
        avg_test_time_ms=round(avg_test_time, 2),
        last_update=latest_kpi.date if latest_kpi else datetime.utcnow(),
        supported_elements=supported_elements,
        lines_of_code=latest_kpi.lines_of_code if latest_kpi else 0,
    )

    recent_runs = (
        db.query(TestRun)
        .join(TestCase)
        .join(TestModule)
        .order_by(desc(TestRun.run_date))
        .limit(10)
        .all()
    )

    recent_validations = (
        db.query(ValidationResult)
        .join(Example)
        .order_by(desc(ValidationResult.run_date))
        .limit(10)
        .all()
    )

    return templates.TemplateResponse(
        "validation_dashboard.html",
        {
            "request": request,
            "stats": stats,
            "recent_runs": recent_runs,
            "recent_validations": recent_validations,
            "total_examples": total_examples,
            "total_validations": db.query(ValidationResult).count(),
            "validation_enabled": True,
        },
    )


@app.get("/modules", response_class=HTMLResponse)
async def modules_page(request: Request, db: Session = Depends(get_db)):
    """Test modules overview page."""
    _ensure_validation_enabled()

    modules = db.query(TestModule).all()

    modules_with_stats = []
    for module in modules:
        test_count = len(module.test_cases)
        passing = sum(1 for tc in module.test_cases if tc.runs and tc.runs[-1].passed)
        modules_with_stats.append(
            {
                "module": module,
                "test_count": test_count,
                "passing": passing,
                "pass_rate": (passing / test_count * 100) if test_count > 0 else 0,
            }
        )

    return templates.TemplateResponse(
        "validation_modules.html", {"request": request, "modules": modules_with_stats}
    )


@app.get("/modules/{module_code}", response_class=HTMLResponse)
async def module_detail_page(request: Request, module_code: str, db: Session = Depends(get_db)):
    """Module detail page with test cases and KPIs."""
    _ensure_validation_enabled()

    module = db.query(TestModule).filter(TestModule.name == module_code).first()
    if not module:
        raise HTTPException(status_code=404, detail="Module not found")

    test_cases_with_runs = []
    for tc in module.test_cases:
        last_run = tc.runs[-1] if tc.runs else None
        test_cases_with_runs.append(
            {
                "test_case": tc,
                "last_run": last_run,
            }
        )

    latest_kpi = db.query(KPI).order_by(desc(KPI.date)).first()

    return templates.TemplateResponse(
        "validation_module_detail.html",
        {
            "request": request,
            "module": module,
            "test_cases": test_cases_with_runs,
            "latest_kpi": latest_kpi,
        },
    )


@app.get("/modules/{module_code}/{test_code}", response_class=HTMLResponse)
async def test_case_detail_page(
    request: Request, module_code: str, test_code: str, db: Session = Depends(get_db)
):
    """Test case detail page with run history."""
    from urllib.parse import unquote

    _ensure_validation_enabled()

    module_code = unquote(module_code)
    test_code = unquote(test_code)

    module = db.query(TestModule).filter(TestModule.name == module_code).first()
    if not module:
        raise HTTPException(status_code=404, detail=f"Module '{module_code}' not found")

    test_case = (
        db.query(TestCase)
        .filter(TestCase.module_id == module.id, TestCase.name == test_code)
        .first()
    )
    if not test_case:
        raise HTTPException(
            status_code=404,
            detail=f"Test case '{test_code}' not found in module '{module_code}'",
        )

    test_runs = sorted(test_case.runs, key=lambda run: run.run_date, reverse=True)
    latest_kpi = db.query(KPI).order_by(desc(KPI.date)).first()

    return templates.TemplateResponse(
        "validation_test_case_detail.html",
        {
            "request": request,
            "module": module,
            "test_case": test_case,
            "test_runs": test_runs,
            "latest_kpi": latest_kpi,
        },
    )


@app.get("/examples", response_class=HTMLResponse)
async def examples_page(request: Request, db: Session = Depends(get_db)):
    """Examples overview page."""
    _ensure_validation_enabled()

    examples = db.query(Example).all()

    examples_with_stats = []
    for example in examples:
        validations = example.validations
        num_validations = len(validations)
        all_passed = all(v.passed for v in validations) if validations else False
        max_error = (
            max(abs(v.relative_error) for v in validations if v.relative_error is not None)
            if validations
            else 0.0
        )
        avg_error = (
            sum(abs(v.relative_error) for v in validations if v.relative_error is not None)
            / num_validations
            if num_validations > 0
            else 0.0
        )
        last_run = max((v.run_date for v in validations), default=None)

        examples_with_stats.append(
            {
                "example": example,
                "num_validations": num_validations,
                "all_passed": all_passed,
                "max_error_percent": max_error * 100 if max_error else 0,
                "avg_error_percent": avg_error * 100 if avg_error else 0,
                "last_run": last_run,
            }
        )

    return templates.TemplateResponse(
        "validation_examples.html", {"request": request, "examples": examples_with_stats}
    )


@app.get("/examples/{example_id}", response_class=HTMLResponse)
async def example_detail_page(request: Request, example_id: int, db: Session = Depends(get_db)):
    """Example detail page with validations."""
    _ensure_validation_enabled()

    example = db.query(Example).filter(Example.id == example_id).first()
    if not example:
        raise HTTPException(status_code=404, detail="Example not found")

    validations = (
        db.query(ValidationResult)
        .filter(ValidationResult.example_id == example_id)
        .order_by(ValidationResult.run_date.desc())
        .all()
    )

    return templates.TemplateResponse(
        "validation_example_detail.html",
        {
            "request": request,
            "example": example,
            "validations": validations,
        },
    )


@app.get("/api/modules", response_model=List[schemas.TestModule])
def get_modules(db: Session = Depends(get_db)):
    """Get all test modules."""
    _ensure_validation_enabled()

    modules = db.query(TestModule).all()
    result = []
    for module in modules:
        module_dict = {
            "id": module.id,
            "name": module.name,
            "description": module.description,
            "created_at": module.created_at,
            "num_tests": len(module.test_cases),
            "passing_tests": sum(
                1 for tc in module.test_cases if tc.runs and tc.runs[-1].passed
            ),
        }
        result.append(schemas.TestModule(**module_dict))
    return result


@app.post("/api/modules", response_model=schemas.TestModule)
def create_module(module: schemas.TestModuleCreate, db: Session = Depends(get_db)):
    """Create a new test module."""
    _ensure_validation_enabled()

    db_module = TestModule(**module.dict())
    db.add(db_module)
    db.commit()
    db.refresh(db_module)
    return schemas.TestModule(
        id=db_module.id,
        name=db_module.name,
        description=db_module.description,
        created_at=db_module.created_at,
        num_tests=0,
        passing_tests=0,
    )


@app.get("/api/test-cases", response_model=List[schemas.TestCase])
def get_test_cases(module_id: int | None = None, db: Session = Depends(get_db)):
    """Get all test cases, optionally filtered by module."""
    _ensure_validation_enabled()

    query = db.query(TestCase)
    if module_id:
        query = query.filter(TestCase.module_id == module_id)
    test_cases = query.all()

    result = []
    for tc in test_cases:
        last_run = tc.runs[-1] if tc.runs else None
        tc_dict = {
            "id": tc.id,
            "module_id": tc.module_id,
            "name": tc.name,
            "description": tc.description,
            "test_type": tc.test_type,
            "created_at": tc.created_at,
            "last_run": last_run.run_date if last_run else None,
            "last_result": last_run.passed if last_run else None,
        }
        result.append(schemas.TestCase(**tc_dict))
    return result


@app.post("/api/test-cases", response_model=schemas.TestCase)
def create_test_case(test_case: schemas.TestCaseCreate, db: Session = Depends(get_db)):
    """Create a new test case."""
    _ensure_validation_enabled()

    db_test_case = TestCase(**test_case.dict())
    db.add(db_test_case)
    db.commit()
    db.refresh(db_test_case)
    return schemas.TestCase(
        id=db_test_case.id,
        module_id=db_test_case.module_id,
        name=db_test_case.name,
        description=db_test_case.description,
        test_type=db_test_case.test_type,
        created_at=db_test_case.created_at,
        last_run=None,
        last_result=None,
    )


@app.get("/api/test-runs", response_model=List[schemas.TestRun])
def get_test_runs(
    test_case_id: int | None = None,
    limit: int = 100,
    db: Session = Depends(get_db),
):
    """Get test run history."""
    _ensure_validation_enabled()

    query = db.query(TestRun).order_by(desc(TestRun.run_date))
    if test_case_id:
        query = query.filter(TestRun.test_case_id == test_case_id)
    return query.limit(limit).all()


@app.post("/api/test-runs", response_model=schemas.TestRun)
def create_test_run(test_run: schemas.TestRunCreate, db: Session = Depends(get_db)):
    """Record a new test run."""
    _ensure_validation_enabled()

    db_test_run = TestRun(**test_run.dict())
    db.add(db_test_run)
    db.commit()
    db.refresh(db_test_run)
    return db_test_run


@app.get("/api/examples", response_model=List[schemas.Example])
def get_examples(db: Session = Depends(get_db)):
    """Get all example problems."""
    _ensure_validation_enabled()

    examples = db.query(Example).all()
    result = []
    for example in examples:
        last_validation = (
            max((v.run_date for v in example.validations), default=None)
            if example.validations
            else None
        )
        example_dict = {
            "id": example.id,
            "name": example.name,
            "description": example.description,
            "input_file_path": example.input_file_path,
            "element_type": example.element_type,
            "num_nodes": example.num_nodes,
            "num_elements": example.num_elements,
            "num_dofs": example.num_dofs,
            "created_at": example.created_at,
            "num_validations": len(example.validations),
            "last_validation": last_validation,
        }
        result.append(schemas.Example(**example_dict))
    return result


@app.post("/api/examples", response_model=schemas.Example)
def create_example(example: schemas.ExampleCreate, db: Session = Depends(get_db)):
    """Create a new example problem."""
    _ensure_validation_enabled()

    db_example = Example(**example.dict())
    db.add(db_example)
    db.commit()
    db.refresh(db_example)
    return schemas.Example(
        id=db_example.id,
        name=db_example.name,
        description=db_example.description,
        input_file_path=db_example.input_file_path,
        element_type=db_example.element_type,
        num_nodes=db_example.num_nodes,
        num_elements=db_example.num_elements,
        num_dofs=db_example.num_dofs,
        created_at=db_example.created_at,
        num_validations=0,
        last_validation=None,
    )


@app.get("/api/validation-results", response_model=List[schemas.ValidationResult])
def get_validation_results(
    example_id: int | None = None,
    limit: int = 100,
    db: Session = Depends(get_db),
):
    """Get validation results."""
    _ensure_validation_enabled()

    query = db.query(ValidationResult).order_by(desc(ValidationResult.run_date))
    if example_id:
        query = query.filter(ValidationResult.example_id == example_id)
    return query.limit(limit).all()


@app.post("/api/validation-results", response_model=schemas.ValidationResult)
def create_validation_result(
    validation: schemas.ValidationResultCreate,
    db: Session = Depends(get_db),
):
    """Record a new validation result."""
    _ensure_validation_enabled()

    db_validation = ValidationResult(**validation.dict())
    db.add(db_validation)
    db.commit()
    db.refresh(db_validation)
    return db_validation


@app.get("/api/kpis", response_model=List[schemas.KPI])
def get_kpis(limit: int = 30, db: Session = Depends(get_db)):
    """Get KPI history."""
    _ensure_validation_enabled()

    return db.query(KPI).order_by(desc(KPI.date)).limit(limit).all()


@app.post("/api/kpis", response_model=schemas.KPI)
def create_kpi(kpi: schemas.KPICreate, db: Session = Depends(get_db)):
    """Record a new KPI snapshot."""
    _ensure_validation_enabled()

    db_kpi = KPI(**kpi.dict())
    db.add(db_kpi)
    db.commit()
    db.refresh(db_kpi)
    return db_kpi


@app.get("/api/kpis/latest", response_model=schemas.KPI)
def get_latest_kpi(db: Session = Depends(get_db)):
    """Get the latest KPI snapshot."""
    _ensure_validation_enabled()

    kpi = db.query(KPI).order_by(desc(KPI.date)).first()
    if not kpi:
        raise HTTPException(status_code=404, detail="No KPIs recorded yet")
    return kpi


@app.get("/api/stats/dashboard", response_model=schemas.DashboardStats)
def get_dashboard_stats(db: Session = Depends(get_db)):
    """Get comprehensive dashboard statistics."""
    _ensure_validation_enabled()

    latest_kpi = db.query(KPI).order_by(desc(KPI.date)).first()

    total_tests = db.query(TestCase).count()
    latest_runs = (
        db.query(TestRun)
        .join(TestCase)
        .group_by(TestCase.id)
        .with_entities(TestCase.id, func.max(TestRun.run_date).label("last_run"))
        .subquery()
    )
    passing_tests = (
        db.query(func.count(TestRun.id))
        .join(
            latest_runs,
            (TestRun.test_case_id == latest_runs.c.id)
            & (TestRun.run_date == latest_runs.c.last_run),
        )
        .filter(TestRun.passed == True)
        .scalar()
        or 0
    )

    total_examples = db.query(Example).count()
    validated_examples = (
        db.query(func.count(func.distinct(ValidationResult.example_id))).scalar() or 0
    )

    total_modules = db.query(TestModule).count()

    avg_test_time = (
        db.query(func.avg(TestRun.execution_time_ms))
        .filter(TestRun.execution_time_ms.isnot(None))
        .scalar()
        or 0.0
    )

    element_types = (
        db.query(Example.element_type)
        .filter(Example.element_type.isnot(None))
        .distinct()
        .all()
    )
    supported_elements = [et[0] for et in element_types if et[0]]

    return schemas.DashboardStats(
        total_tests=total_tests,
        passing_tests=passing_tests,
        failing_tests=total_tests - passing_tests,
        pass_rate=(passing_tests / total_tests * 100) if total_tests > 0 else 0.0,
        total_examples=total_examples,
        validated_examples=validated_examples,
        total_modules=total_modules,
        avg_test_time_ms=round(avg_test_time, 2),
        last_update=latest_kpi.date if latest_kpi else datetime.utcnow(),
        supported_elements=supported_elements,
        lines_of_code=latest_kpi.lines_of_code if latest_kpi else 0,
    )


@app.get("/api/nastran/status")
async def nastran_status():
    """Check pyNastran integration status."""
    available = is_pynastran_available()
    return {
        "pynastran_available": available,
        "bdf_converter": available,
        "op2_reader": available,
    }


@app.post("/api/nastran/convert/bdf-to-inp")
async def convert_bdf_to_inp(file: UploadFile = File(...)):
    """Convert uploaded BDF file to CalculiX INP format."""
    if not is_pynastran_available():
        raise HTTPException(
            status_code=503,
            detail="pyNastran is not available. Install with: pip install pyNastran",
        )

    UPLOADS_DIR.mkdir(exist_ok=True)

    bdf_path = UPLOADS_DIR / file.filename
    with bdf_path.open("wb") as handle:
        content = await file.read()
        handle.write(content)

    inp_filename = bdf_path.stem + ".inp"
    inp_path = UPLOADS_DIR / inp_filename

    try:
        stats = BdfToInpConverter.convert_file(str(bdf_path), str(inp_path))
        return {
            "status": "success",
            "input_file": file.filename,
            "output_file": inp_filename,
            "download_url": f"/api/nastran/download/{inp_filename}",
            "statistics": stats,
        }
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"Conversion failed: {exc}")


@app.get("/api/nastran/download/{filename}")
async def download_converted_file(filename: str):
    """Download converted INP file."""
    file_path = UPLOADS_DIR / filename
    if not file_path.exists():
        raise HTTPException(status_code=404, detail="File not found")
    return FileResponse(file_path, media_type="text/plain", filename=filename)


@app.post("/api/nastran/read/op2")
async def read_op2_results(file: UploadFile = File(...)):
    """Read Nastran OP2 binary result file."""
    if not is_pynastran_available():
        raise HTTPException(
            status_code=503,
            detail="pyNastran is not available. Install with: pip install pyNastran",
        )

    UPLOADS_DIR.mkdir(exist_ok=True)

    op2_path = UPLOADS_DIR / file.filename
    with op2_path.open("wb") as handle:
        content = await file.read()
        handle.write(content)

    try:
        data = Op2ResultReader.read_results(str(op2_path))

        num_displacements = len(data.get("displacements", {}))
        num_stresses = len(data.get("stresses", {}))
        num_eigenvalues = len(data.get("eigenvalues", []))

        return {
            "status": "success",
            "input_file": file.filename,
            "statistics": {
                "num_displacements": num_displacements,
                "num_stresses": num_stresses,
                "num_eigenvalues": num_eigenvalues,
            },
            "data": data,
        }
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"Failed to read OP2: {exc}")


@app.post("/api/nastran/extract/frequencies")
async def extract_modal_frequencies(file: UploadFile = File(...)):
    """Extract natural frequencies from modal OP2 file."""
    if not is_pynastran_available():
        raise HTTPException(
            status_code=503,
            detail="pyNastran is not available. Install with: pip install pyNastran",
        )

    UPLOADS_DIR.mkdir(exist_ok=True)

    op2_path = UPLOADS_DIR / file.filename
    with op2_path.open("wb") as handle:
        content = await file.read()
        handle.write(content)

    try:
        frequencies = Op2ResultReader.extract_frequencies(str(op2_path))
        return {
            "status": "success",
            "input_file": file.filename,
            "num_modes": len(frequencies),
            "frequencies_hz": frequencies,
        }
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"Failed to extract frequencies: {exc}")


# Backward-compatible command endpoint used by run_ccx.html
@app.post("/api/run-command")
async def run_command(command: str, input_file: str | None = None, input_dir: str | None = None):
    """Run a selected ccx-cli command for the web command console."""
    if not CCX_CLI_BIN.exists():
        raise HTTPException(status_code=503, detail=f"ccx-cli not found: {CCX_CLI_BIN}")

    cmd: list[str] = [str(CCX_CLI_BIN)]

    if command == "version":
        cmd.append("--version")
    elif command == "help":
        cmd.append("--help")
    elif command == "analyze":
        if not input_file:
            raise HTTPException(status_code=400, detail="input_file is required for analyze")
        cmd.extend(["analyze", str(FIXTURES_DIR / input_file)])
    elif command == "solve":
        if not input_file:
            raise HTTPException(status_code=400, detail="input_file is required for solve")
        cmd.extend(["solve", str(FIXTURES_DIR / input_file)])
    elif command == "validate":
        dir_value = input_dir or str(FIXTURES_DIR)
        cmd.extend(["validate", "--fixtures-dir", dir_value])
    else:
        raise HTTPException(status_code=400, detail=f"Unsupported command: {html.escape(command)}")

    start_time = time.time()
    result = _run_command(cmd, timeout=600)
    duration = time.time() - start_time

    return {
        "success": result["success"],
        "returncode": result["returncode"],
        "stdout": result["stdout"],
        "stderr": result["stderr"],
        "duration_seconds": round(duration, 3),
        "command": " ".join(cmd),
    }


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8000)
