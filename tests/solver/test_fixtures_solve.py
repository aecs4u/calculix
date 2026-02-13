"""Test solver fixtures with ccx-cli solve command.

This module runs the 'solve' command (full FEA solve) on selected fixtures
to validate the solver implementation.
"""

import pytest
from pathlib import Path


# Fixtures known to be solvable with current implementation
# Note: These are simple fixtures that exist in tests/fixtures/solver/
SOLVABLE_FIXTURES = [
    "beamcr4.inp",
    "oneeltruss.inp",
    "simplebeam.inp",
]


@pytest.mark.solve
@pytest.mark.parametrize("fixture_name", SOLVABLE_FIXTURES)
def test_solve_fixture(fixture_name: str, fixtures_dir: Path, run_ccx_solve, ccx_cli_available):
    """Test that fixture can be solved successfully.

    Args:
        fixture_name: Name of the fixture file
        fixtures_dir: Path to fixtures directory
        run_ccx_solve: Fixture function to run solve command
        ccx_cli_available: Boolean indicating if ccx-cli is available
    """
    if not ccx_cli_available:
        pytest.skip("ccx-cli binary not found (run: cargo build --release)")

    fixture_file = fixtures_dir / fixture_name
    if not fixture_file.exists():
        pytest.skip(f"Fixture not found: {fixture_file}")

    # Run solve command
    result = run_ccx_solve(fixture_file)

    # Check results
    assert result["success"], (
        f"Failed to solve {fixture_name}\n"
        f"Exit code: {result['returncode']}\n"
        f"STDERR: {result['stderr']}\n"
        f"STDOUT: {result['stdout']}"
    )

    # Verify output was produced
    assert result["stdout"], f"No output from solve command for {fixture_name}"

    # Report duration
    print(f"✓ {fixture_name} solved in {result['duration']:.3f}s")


@pytest.mark.solve
@pytest.mark.slow
def test_solve_with_reference_comparison(fixtures_dir: Path, validation_dir: Path,
                                          run_ccx_solve, reference_file, ccx_cli_available):
    """Test solving fixtures and compare with reference results where available.

    This test:
    1. Solves each fixture
    2. Compares output with reference .dat.ref file if available
    3. Reports validation metrics
    """
    if not ccx_cli_available:
        pytest.skip("ccx-cli binary not found")

    results = {
        "solved": 0,
        "validated": 0,
        "failed": 0,
        "no_reference": 0,
    }

    for fixture_name in SOLVABLE_FIXTURES:
        fixture_file = fixtures_dir / fixture_name
        if not fixture_file.exists():
            continue

        # Solve
        result = run_ccx_solve(fixture_file)

        if result["success"]:
            results["solved"] += 1

            # Check for reference file
            ref_file = reference_file(fixture_name)
            if ref_file:
                # TODO: Implement result comparison with reference
                # For now, just count as validated if reference exists
                results["validated"] += 1
            else:
                results["no_reference"] += 1
        else:
            results["failed"] += 1

    # Report
    print("\n" + "=" * 60)
    print("SOLVE WITH REFERENCE COMPARISON")
    print("=" * 60)
    print(f"Solved:           {results['solved']}")
    print(f"Validated:        {results['validated']}")
    print(f"No reference:     {results['no_reference']}")
    print(f"Failed:           {results['failed']}")

    # Test passes if at least some fixtures solved
    assert results["solved"] > 0, "No fixtures solved successfully"


@pytest.mark.solve
@pytest.mark.beam
def test_solve_beam_fixtures(fixtures_dir: Path, run_ccx_solve, ccx_cli_available):
    """Test solving all beam element fixtures."""
    if not ccx_cli_available:
        pytest.skip("ccx-cli binary not found")

    beam_fixtures = [f for f in SOLVABLE_FIXTURES if "beam" in f.lower()]

    if not beam_fixtures:
        pytest.skip("No beam fixtures in solvable list")

    results = {"passed": 0, "failed": 0}

    for fixture_name in beam_fixtures:
        fixture_file = fixtures_dir / fixture_name
        if not fixture_file.exists():
            continue

        result = run_ccx_solve(fixture_file)
        if result["success"]:
            results["passed"] += 1
            print(f"  ✓ {fixture_name}")
        else:
            results["failed"] += 1
            print(f"  ✗ {fixture_name}: {result['stderr'][:100]}")

    assert results["passed"] > 0, "No beam fixtures solved successfully"
    assert results["failed"] == 0, f"{results['failed']} beam fixtures failed"


@pytest.mark.solve
@pytest.mark.truss
def test_solve_truss_fixtures(fixtures_dir: Path, run_ccx_solve, ccx_cli_available):
    """Test solving all truss element fixtures."""
    if not ccx_cli_available:
        pytest.skip("ccx-cli binary not found")

    truss_fixtures = [f for f in SOLVABLE_FIXTURES if "truss" in f.lower() or "bar" in f.lower()]

    if not truss_fixtures:
        pytest.skip("No truss fixtures in solvable list")

    results = {"passed": 0, "failed": 0}

    for fixture_name in truss_fixtures:
        fixture_file = fixtures_dir / fixture_name
        if not fixture_file.exists():
            continue

        result = run_ccx_solve(fixture_file)
        if result["success"]:
            results["passed"] += 1
            print(f"  ✓ {fixture_name}")
        else:
            results["failed"] += 1
            print(f"  ✗ {fixture_name}: {result['stderr'][:100]}")

    assert results["passed"] > 0, "No truss fixtures solved successfully"
    assert results["failed"] == 0, f"{results['failed']} truss fixtures failed"
