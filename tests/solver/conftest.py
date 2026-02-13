"""Pytest configuration for solver tests.

This module provides fixtures and configuration for testing the ccx-solver
with all test fixtures in tests/fixtures/solver/.
"""

import os
import subprocess
from pathlib import Path
from typing import Dict, List, Optional

import pytest


# Project root directory
PROJECT_ROOT = Path(__file__).resolve().parents[2]
FIXTURES_DIR = PROJECT_ROOT / "tests" / "fixtures" / "solver"
VALIDATION_DIR = PROJECT_ROOT / "validation" / "solver"
SCRATCH_DIR = PROJECT_ROOT / "scratch" / "solver"
CCX_CLI_BIN = PROJECT_ROOT / "target" / "release" / "ccx-cli"

# Create scratch directory for outputs
SCRATCH_DIR.mkdir(parents=True, exist_ok=True)


@pytest.fixture(scope="session")
def project_root() -> Path:
    """Project root directory."""
    return PROJECT_ROOT


@pytest.fixture(scope="session")
def fixtures_dir() -> Path:
    """Solver fixtures directory."""
    return FIXTURES_DIR


@pytest.fixture(scope="session")
def validation_dir() -> Path:
    """Validation reference files directory."""
    return VALIDATION_DIR


@pytest.fixture(scope="session")
def ccx_cli_bin() -> Path:
    """Path to ccx-cli binary."""
    return CCX_CLI_BIN


@pytest.fixture(scope="session")
def ccx_cli_available(ccx_cli_bin: Path) -> bool:
    """Check if ccx-cli binary is available."""
    return ccx_cli_bin.exists()


@pytest.fixture(scope="session")
def all_fixtures(fixtures_dir: Path) -> List[Path]:
    """Get all .inp fixture files."""
    if not fixtures_dir.exists():
        return []
    return sorted(fixtures_dir.glob("*.inp"))


@pytest.fixture
def run_ccx_analyze(ccx_cli_bin: Path, tmp_path: Path):
    """Fixture to run ccx-cli analyze command.

    Returns a function that takes an INP file path and returns the result.
    Saves output files to scratch/solver/ directory.
    """
    def _run_analyze(inp_file: Path, timeout: int = 60) -> Dict:
        """Run ccx-cli analyze on a fixture.

        Args:
            inp_file: Path to the .inp file
            timeout: Command timeout in seconds

        Returns:
            Dictionary with:
                - success: bool - whether command succeeded
                - returncode: int - exit code
                - stdout: str - standard output
                - stderr: str - standard error
                - duration: float - execution time in seconds
                - output_file: Path - path to saved output file
        """
        import time

        if not ccx_cli_bin.exists():
            return {
                "success": False,
                "returncode": -1,
                "stdout": "",
                "stderr": f"ccx-cli not found: {ccx_cli_bin}",
                "duration": 0.0,
                "output_file": None,
            }

        # Prepare output file path
        output_file = SCRATCH_DIR / f"{inp_file.stem}_analyze.txt"

        start_time = time.time()
        try:
            result = subprocess.run(
                [str(ccx_cli_bin), "analyze", str(inp_file)],
                capture_output=True,
                text=True,
                timeout=timeout,
                cwd=PROJECT_ROOT,
            )
            duration = time.time() - start_time

            # Save output to scratch directory
            with open(output_file, "w") as f:
                f.write(f"=== ANALYZE: {inp_file.name} ===\n")
                f.write(f"Duration: {duration:.3f}s\n")
                f.write(f"Exit Code: {result.returncode}\n\n")
                f.write("=== STDOUT ===\n")
                f.write(result.stdout)
                f.write("\n\n=== STDERR ===\n")
                f.write(result.stderr)

            return {
                "success": result.returncode == 0,
                "returncode": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "duration": duration,
                "output_file": output_file,
            }
        except subprocess.TimeoutExpired:
            duration = time.time() - start_time
            return {
                "success": False,
                "returncode": -1,
                "stdout": "",
                "stderr": f"Command timed out after {timeout} seconds",
                "duration": duration,
                "output_file": None,
            }
        except Exception as e:
            duration = time.time() - start_time
            return {
                "success": False,
                "returncode": -1,
                "stdout": "",
                "stderr": str(e),
                "duration": duration,
                "output_file": None,
            }

    return _run_analyze


@pytest.fixture
def run_ccx_solve(ccx_cli_bin: Path, tmp_path: Path):
    """Fixture to run ccx-cli solve command.

    Returns a function that takes an INP file path and returns the result.
    Saves output files to scratch/solver/ directory.
    """
    def _run_solve(inp_file: Path, timeout: int = 300) -> Dict:
        """Run ccx-cli solve on a fixture.

        Args:
            inp_file: Path to the .inp file
            timeout: Command timeout in seconds (default 5 minutes)

        Returns:
            Dictionary with result details including output_file path
        """
        import time

        if not ccx_cli_bin.exists():
            return {
                "success": False,
                "returncode": -1,
                "stdout": "",
                "stderr": f"ccx-cli not found: {ccx_cli_bin}",
                "duration": 0.0,
                "output_file": None,
            }

        # Prepare output file path
        output_file = SCRATCH_DIR / f"{inp_file.stem}_solve.txt"

        start_time = time.time()
        try:
            result = subprocess.run(
                [str(ccx_cli_bin), "solve", str(inp_file)],
                capture_output=True,
                text=True,
                timeout=timeout,
                cwd=PROJECT_ROOT,
            )
            duration = time.time() - start_time

            # Save output to scratch directory
            with open(output_file, "w") as f:
                f.write(f"=== SOLVE: {inp_file.name} ===\n")
                f.write(f"Duration: {duration:.3f}s\n")
                f.write(f"Exit Code: {result.returncode}\n\n")
                f.write("=== STDOUT ===\n")
                f.write(result.stdout)
                f.write("\n\n=== STDERR ===\n")
                f.write(result.stderr)

            return {
                "success": result.returncode == 0,
                "returncode": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "duration": duration,
                "output_file": output_file,
            }
        except subprocess.TimeoutExpired:
            duration = time.time() - start_time
            return {
                "success": False,
                "returncode": -1,
                "stdout": "",
                "stderr": f"Command timed out after {timeout} seconds",
                "duration": duration,
                "output_file": None,
            }
        except Exception as e:
            duration = time.time() - start_time
            return {
                "success": False,
                "returncode": -1,
                "stdout": "",
                "stderr": str(e),
                "duration": duration,
                "output_file": None,
            }

    return _run_solve


@pytest.fixture
def reference_file(validation_dir: Path):
    """Get reference file for a fixture.

    Returns a function that takes a fixture name and returns the reference file path.
    """
    def _get_reference(fixture_name: str) -> Optional[Path]:
        """Get reference .dat.ref file for a fixture.

        Args:
            fixture_name: Name of the fixture (e.g., "beamcr4.inp")

        Returns:
            Path to reference file or None if not found
        """
        if not validation_dir.exists():
            return None

        # Get stem (filename without extension)
        stem = Path(fixture_name).stem
        ref_file = validation_dir / f"{stem}.dat.ref"

        return ref_file if ref_file.exists() else None

    return _get_reference


def pytest_configure(config):
    """Configure pytest with custom markers."""
    config.addinivalue_line(
        "markers", "analyze: mark test as analyze-only (parsing test)"
    )
    config.addinivalue_line(
        "markers", "solve: mark test as full solve test"
    )
    config.addinivalue_line(
        "markers", "slow: mark test as slow (> 10 seconds)"
    )
    config.addinivalue_line(
        "markers", "fast: mark test as fast (< 1 second)"
    )
    config.addinivalue_line(
        "markers", "beam: mark test as beam element test"
    )
    config.addinivalue_line(
        "markers", "truss: mark test as truss element test"
    )
    config.addinivalue_line(
        "markers", "shell: mark test as shell element test"
    )
    config.addinivalue_line(
        "markers", "solid: mark test as solid element test"
    )


def pytest_collection_modifyitems(config, items):
    """Automatically mark tests based on their fixture names."""
    for item in items:
        # Get fixture name from test parameters if available
        if hasattr(item, "callspec") and "fixture_file" in item.callspec.params:
            fixture_file = item.callspec.params["fixture_file"]
            fixture_name = fixture_file.name.lower()

            # Mark based on element type in filename
            if "beam" in fixture_name:
                item.add_marker(pytest.mark.beam)
            if "truss" in fixture_name or "bar" in fixture_name:
                item.add_marker(pytest.mark.truss)
            if "shell" in fixture_name or "plate" in fixture_name:
                item.add_marker(pytest.mark.shell)
            if "solid" in fixture_name or "brick" in fixture_name or "hex" in fixture_name:
                item.add_marker(pytest.mark.solid)
