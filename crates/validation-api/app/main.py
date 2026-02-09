"""CalculiX Rust Solver Validation API."""

from datetime import datetime
from typing import List

from fastapi import Depends, FastAPI, HTTPException, Request
from fastapi.responses import HTMLResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates
from sqlalchemy import desc, func
from sqlalchemy.orm import Session

from . import database, schemas
from .database import (
    Example,
    KPI,
    TestCase,
    TestModule,
    TestRun,
    ValidationResult,
    get_db,
    init_db,
)

# Initialize database
init_db()

app = FastAPI(
    title="CalculiX Rust Solver Validation API",
    description="Track and visualize validation results for the CalculiX Rust solver",
    version="0.1.0",
)

templates = Jinja2Templates(directory="app/templates")


# ============================================================================
# Dashboard & Web UI
# ============================================================================


@app.get("/", response_class=HTMLResponse)
async def dashboard(request: Request, db: Session = Depends(get_db)):
    """Main dashboard view."""
    # Get latest KPI
    latest_kpi = db.query(KPI).order_by(desc(KPI.date)).first()

    # Get test statistics
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

    # Get example statistics
    total_examples = db.query(Example).count()
    examples_with_validations = (
        db.query(func.count(func.distinct(ValidationResult.example_id))).scalar() or 0
    )

    # Get module count
    total_modules = db.query(TestModule).count()

    # Calculate average test time
    avg_test_time = (
        db.query(func.avg(TestRun.execution_time_ms))
        .filter(TestRun.execution_time_ms.isnot(None))
        .scalar()
        or 0.0
    )

    # Get supported element types
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

    # Get recent test runs
    recent_runs = (
        db.query(TestRun)
        .join(TestCase)
        .join(TestModule)
        .order_by(desc(TestRun.run_date))
        .limit(10)
        .all()
    )

    # Get validation results
    recent_validations = (
        db.query(ValidationResult)
        .join(Example)
        .order_by(desc(ValidationResult.run_date))
        .limit(10)
        .all()
    )

    return templates.TemplateResponse(
        "dashboard.html",
        {
            "request": request,
            "stats": stats,
            "recent_runs": recent_runs,
            "recent_validations": recent_validations,
        },
    )


@app.get("/modules", response_class=HTMLResponse)
async def modules_page(request: Request, db: Session = Depends(get_db)):
    """Test modules overview page."""
    modules = db.query(TestModule).all()

    # Enhance with statistics
    modules_with_stats = []
    for module in modules:
        test_count = len(module.test_cases)
        passing = sum(
            1
            for tc in module.test_cases
            if tc.runs and tc.runs[-1].passed
        )
        modules_with_stats.append(
            {
                "module": module,
                "test_count": test_count,
                "passing": passing,
                "pass_rate": (passing / test_count * 100) if test_count > 0 else 0,
            }
        )

    return templates.TemplateResponse(
        "modules.html", {"request": request, "modules": modules_with_stats}
    )


@app.get("/examples", response_class=HTMLResponse)
async def examples_page(request: Request, db: Session = Depends(get_db)):
    """Examples overview page."""
    examples = db.query(Example).all()

    # Enhance with validation stats
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
        "examples.html", {"request": request, "examples": examples_with_stats}
    )


# ============================================================================
# API Endpoints - Test Modules
# ============================================================================


@app.get("/api/modules", response_model=List[schemas.TestModule])
def get_modules(db: Session = Depends(get_db)):
    """Get all test modules."""
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


# ============================================================================
# API Endpoints - Test Cases
# ============================================================================


@app.get("/api/test-cases", response_model=List[schemas.TestCase])
def get_test_cases(module_id: int = None, db: Session = Depends(get_db)):
    """Get all test cases, optionally filtered by module."""
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


# ============================================================================
# API Endpoints - Test Runs
# ============================================================================


@app.get("/api/test-runs", response_model=List[schemas.TestRun])
def get_test_runs(test_case_id: int = None, limit: int = 100, db: Session = Depends(get_db)):
    """Get test run history."""
    query = db.query(TestRun).order_by(desc(TestRun.run_date))
    if test_case_id:
        query = query.filter(TestRun.test_case_id == test_case_id)
    return query.limit(limit).all()


@app.post("/api/test-runs", response_model=schemas.TestRun)
def create_test_run(test_run: schemas.TestRunCreate, db: Session = Depends(get_db)):
    """Record a new test run."""
    db_test_run = TestRun(**test_run.dict())
    db.add(db_test_run)
    db.commit()
    db.refresh(db_test_run)
    return db_test_run


# ============================================================================
# API Endpoints - Examples
# ============================================================================


@app.get("/api/examples", response_model=List[schemas.Example])
def get_examples(db: Session = Depends(get_db)):
    """Get all example problems."""
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


# ============================================================================
# API Endpoints - Validation Results
# ============================================================================


@app.get("/api/validation-results", response_model=List[schemas.ValidationResult])
def get_validation_results(
    example_id: int = None, limit: int = 100, db: Session = Depends(get_db)
):
    """Get validation results."""
    query = db.query(ValidationResult).order_by(desc(ValidationResult.run_date))
    if example_id:
        query = query.filter(ValidationResult.example_id == example_id)
    return query.limit(limit).all()


@app.post("/api/validation-results", response_model=schemas.ValidationResult)
def create_validation_result(
    validation: schemas.ValidationResultCreate, db: Session = Depends(get_db)
):
    """Record a new validation result."""
    db_validation = ValidationResult(**validation.dict())
    db.add(db_validation)
    db.commit()
    db.refresh(db_validation)
    return db_validation


# ============================================================================
# API Endpoints - KPIs
# ============================================================================


@app.get("/api/kpis", response_model=List[schemas.KPI])
def get_kpis(limit: int = 30, db: Session = Depends(get_db)):
    """Get KPI history."""
    return db.query(KPI).order_by(desc(KPI.date)).limit(limit).all()


@app.post("/api/kpis", response_model=schemas.KPI)
def create_kpi(kpi: schemas.KPICreate, db: Session = Depends(get_db)):
    """Record a new KPI snapshot."""
    db_kpi = KPI(**kpi.dict())
    db.add(db_kpi)
    db.commit()
    db.refresh(db_kpi)
    return db_kpi


@app.get("/api/kpis/latest", response_model=schemas.KPI)
def get_latest_kpi(db: Session = Depends(get_db)):
    """Get the most recent KPI snapshot."""
    kpi = db.query(KPI).order_by(desc(KPI.date)).first()
    if not kpi:
        raise HTTPException(status_code=404, detail="No KPIs recorded yet")
    return kpi


# ============================================================================
# API Endpoints - Statistics
# ============================================================================


@app.get("/api/stats/dashboard", response_model=schemas.DashboardStats)
def get_dashboard_stats(db: Session = Depends(get_db)):
    """Get comprehensive dashboard statistics."""
    # Get latest KPI
    latest_kpi = db.query(KPI).order_by(desc(KPI.date)).first()

    # Test statistics
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

    # Example statistics
    total_examples = db.query(Example).count()
    validated_examples = (
        db.query(func.count(func.distinct(ValidationResult.example_id))).scalar() or 0
    )

    # Module count
    total_modules = db.query(TestModule).count()

    # Average test time
    avg_test_time = (
        db.query(func.avg(TestRun.execution_time_ms))
        .filter(TestRun.execution_time_ms.isnot(None))
        .scalar()
        or 0.0
    )

    # Supported element types
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


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8000)
