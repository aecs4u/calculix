"""Test all solver fixtures with ccx-cli analyze command.

This module runs the 'analyze' command (parsing only) on all fixtures
in tests/fixtures/solver/ to validate input file parsing.
"""

import pytest
from pathlib import Path


def get_all_fixtures():
    """Get list of all fixture files for parametrization."""
    fixtures_dir = Path(__file__).parent.parent.parent / "tests" / "fixtures" / "solver"
    if not fixtures_dir.exists():
        return []
    return sorted(fixtures_dir.glob("*.inp"))


@pytest.mark.analyze
@pytest.mark.parametrize("fixture_file", get_all_fixtures(), ids=lambda p: p.name)
def test_analyze_fixture(fixture_file: Path, run_ccx_analyze, ccx_cli_available):
    """Test that fixture can be analyzed (parsed) successfully.

    Args:
        fixture_file: Path to the .inp fixture file
        run_ccx_analyze: Fixture function to run analyze command
        ccx_cli_available: Boolean indicating if ccx-cli is available
    """
    if not ccx_cli_available:
        pytest.skip("ccx-cli binary not found (run: cargo build --release)")

    if not fixture_file.exists():
        pytest.skip(f"Fixture not found: {fixture_file}")

    # Run analyze command
    result = run_ccx_analyze(fixture_file)

    # Check results
    assert result["success"], (
        f"Failed to analyze {fixture_file.name}\n"
        f"Exit code: {result['returncode']}\n"
        f"STDERR: {result['stderr']}\n"
        f"STDOUT: {result['stdout']}"
    )

    # Verify some output was produced
    assert result["stdout"], f"No output from analyze command for {fixture_file.name}"

    # Report duration
    print(f"✓ {fixture_file.name} analyzed in {result['duration']:.3f}s")


def test_analyze_all_fixtures_summary(all_fixtures, run_ccx_analyze, ccx_cli_available):
    """Run analyze on all fixtures and report summary statistics.

    This test runs all fixtures and reports comprehensive statistics.
    """
    if not ccx_cli_available:
        pytest.skip("ccx-cli binary not found")

    if not all_fixtures:
        pytest.skip("No fixtures found")

    results = {
        "total": len(all_fixtures),
        "passed": 0,
        "failed": 0,
        "total_time": 0.0,
        "failures": [],
    }

    for fixture_file in all_fixtures:
        result = run_ccx_analyze(fixture_file)
        results["total_time"] += result["duration"]

        if result["success"]:
            results["passed"] += 1
        else:
            results["failed"] += 1
            results["failures"].append({
                "name": fixture_file.name,
                "error": result["stderr"],
            })

    # Report summary
    print("\n" + "=" * 60)
    print("ANALYZE SUMMARY")
    print("=" * 60)
    print(f"Total fixtures:   {results['total']}")
    print(f"Passed:           {results['passed']} ({results['passed']/results['total']*100:.1f}%)")
    print(f"Failed:           {results['failed']}")
    print(f"Total time:       {results['total_time']:.2f}s")
    print(f"Average time:     {results['total_time']/results['total']:.3f}s")

    if results["failures"]:
        print("\nFAILURES:")
        for failure in results["failures"]:
            print(f"  ✗ {failure['name']}: {failure['error'][:100]}")

    # Test passes if > 95% success rate
    success_rate = results["passed"] / results["total"]
    assert success_rate >= 0.95, (
        f"Analyze success rate too low: {success_rate*100:.1f}% "
        f"(expected >= 95%)"
    )
