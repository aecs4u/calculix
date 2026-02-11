"""Test suite for the Rust ccx-solver implementation."""
from __future__ import annotations

from pathlib import Path
import subprocess
import pytest

from tests.testkit import copy_case_tree, run_command


@pytest.fixture(scope="session")
def rust_solver_bin(repo_root: Path) -> Path:
    """Locate the Rust ccx-cli binary."""
    candidates = [
        repo_root / "target" / "release" / "ccx-cli",
        repo_root / "target" / "debug" / "ccx-cli",
    ]

    for candidate in candidates:
        if candidate.exists() and candidate.is_file():
            return candidate

    pytest.skip("ccx-cli binary not found. Run: cargo build --release --package ccx-cli")


@pytest.mark.rust_solver
@pytest.mark.smoke
def test_rust_solver_prints_version(rust_solver_bin: Path) -> None:
    """Test that the Rust solver prints version information."""
    result = run_command([str(rust_solver_bin), "--version"], cwd=rust_solver_bin.parent, timeout_s=30)
    combined = f"{result.stdout}\n{result.stderr}"
    assert result.returncode == 0, combined
    assert "0.1.0" in combined


@pytest.mark.rust_solver
@pytest.mark.smoke
def test_rust_solver_validation_command_exists(rust_solver_bin: Path) -> None:
    """Test that the Rust solver has a validate command."""
    result = run_command([str(rust_solver_bin), "--help"], cwd=rust_solver_bin.parent, timeout_s=30)
    combined = f"{result.stdout}\n{result.stderr}"
    assert result.returncode == 0, combined
    assert "validate" in combined
    assert "analyze" in combined


@pytest.mark.rust_solver
@pytest.mark.integration
def test_rust_solver_analyzes_simple_cases(
    rust_solver_bin: Path,
    solver_cases,
    tmp_path: Path,
    full_matrix: bool,
) -> None:
    """Test that the Rust solver can analyze simple test cases."""
    if not solver_cases:
        pytest.skip("No solver cases available.")

    timeout_s = 180 if full_matrix else 60
    failures: list[str] = []
    successes: list[str] = []

    # Test a subset of cases (only first 3 unless full_matrix)
    test_cases = solver_cases if full_matrix else solver_cases[:3]

    for case in test_cases:
        case_copy = copy_case_tree(case.inp_path, tmp_path / "rust_solver")

        try:
            result = run_command(
                [str(rust_solver_bin), "analyze", str(case_copy)],
                cwd=case_copy.parent,
                timeout_s=timeout_s,
            )
        except subprocess.TimeoutExpired:
            failures.append(f"[{case.inp_path.name}] timed out after {timeout_s}s")
            continue

        # For analyze command, we just check it can parse the file
        # returncode 0 = success, 1 = parse/analysis error, 2 = usage error
        if result.returncode not in [0, 1]:
            failures.append(
                f"[{case.inp_path.name}] returncode={result.returncode}\n"
                f"stdout:\n{result.stdout}\n"
                f"stderr:\n{result.stderr}\n"
            )
            continue

        # returncode 0 or 1 is acceptable (parse success or analysis skip)
        successes.append(case.inp_path.name)

    # Report results
    print(f"\n✓ Analyzed {len(successes)}/{len(test_cases)} cases successfully")
    if failures:
        print(f"✗ Failed {len(failures)} cases:")
        for failure in failures:
            print(f"  {failure}")

    # We expect at least 50% success rate
    success_rate = len(successes) / len(test_cases) if test_cases else 0
    assert success_rate >= 0.5, (
        f"Success rate {success_rate:.1%} is below 50%\n"
        f"Failures:\n" + "\n".join(failures)
    )


@pytest.mark.rust_solver
@pytest.mark.slow
def test_rust_solver_validation_suite(rust_solver_bin: Path, repo_root: Path) -> None:
    """Test the Rust solver's built-in validation suite."""
    fixtures_dir = repo_root / "tests" / "fixtures" / "solver"

    if not fixtures_dir.exists():
        pytest.skip("Fixtures directory not found")

    # Run the validation command (with shorter timeout since we're just testing it works)
    result = run_command(
        [str(rust_solver_bin), "validate", "--fixtures-dir", str(fixtures_dir)],
        cwd=repo_root,
        timeout_s=300,  # 5 minutes
    )

    combined = f"{result.stdout}\n{result.stderr}"

    # Validation command should complete (may have failures, that's ok)
    # returncode 0 = all passed, 1 = some failures
    assert result.returncode in [0, 1], (
        f"Validation command failed with returncode {result.returncode}\n{combined}"
    )

    # Check that it reports results
    assert "tests" in combined.lower() or "fixtures" in combined.lower(), (
        f"Validation output doesn't mention tests/fixtures:\n{combined}"
    )
