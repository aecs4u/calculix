"""Performance benchmarking tests for solver fixtures.

This module tests solver performance on various fixture sizes.
"""

import pytest
from pathlib import Path


@pytest.mark.slow
def test_analyze_performance_benchmark(all_fixtures, run_ccx_analyze, ccx_cli_available):
    """Benchmark analyze performance on all fixtures.

    Reports:
    - Fastest fixtures
    - Slowest fixtures
    - Average performance
    - Performance by element type
    """
    if not ccx_cli_available:
        pytest.skip("ccx-cli binary not found")

    if not all_fixtures:
        pytest.skip("No fixtures found")

    # Run all fixtures and collect timing
    timings = []
    for fixture_file in all_fixtures:
        result = run_ccx_analyze(fixture_file)
        if result["success"]:
            timings.append({
                "name": fixture_file.name,
                "duration": result["duration"],
            })

    if not timings:
        pytest.skip("No successful analyze runs")

    # Sort by duration
    timings.sort(key=lambda x: x["duration"])

    # Statistics
    durations = [t["duration"] for t in timings]
    avg_duration = sum(durations) / len(durations)
    min_duration = min(durations)
    max_duration = max(durations)

    # Report
    print("\n" + "=" * 60)
    print("ANALYZE PERFORMANCE BENCHMARK")
    print("=" * 60)
    print(f"Total fixtures:   {len(timings)}")
    print(f"Average time:     {avg_duration:.3f}s")
    print(f"Min time:         {min_duration:.3f}s ({timings[0]['name']})")
    print(f"Max time:         {max_duration:.3f}s ({timings[-1]['name']})")
    print(f"Median time:      {durations[len(durations)//2]:.3f}s")

    print("\nFASTEST 5:")
    for timing in timings[:5]:
        print(f"  {timing['duration']:.3f}s  {timing['name']}")

    print("\nSLOWEST 5:")
    for timing in timings[-5:]:
        print(f"  {timing['duration']:.3f}s  {timing['name']}")

    # Performance assertions
    assert avg_duration < 1.0, f"Average analyze time too slow: {avg_duration:.3f}s (expected < 1.0s)"
    assert max_duration < 5.0, f"Slowest fixture too slow: {max_duration:.3f}s (expected < 5.0s)"


@pytest.mark.solve
@pytest.mark.slow
def test_solve_performance_benchmark(fixtures_dir: Path, run_ccx_solve, ccx_cli_available):
    """Benchmark solve performance on solvable fixtures."""
    from test_fixtures_solve import SOLVABLE_FIXTURES

    if not ccx_cli_available:
        pytest.skip("ccx-cli binary not found")

    timings = []
    for fixture_name in SOLVABLE_FIXTURES:
        fixture_file = fixtures_dir / fixture_name
        if not fixture_file.exists():
            continue

        result = run_ccx_solve(fixture_file)
        if result["success"]:
            timings.append({
                "name": fixture_name,
                "duration": result["duration"],
            })

    if not timings:
        pytest.skip("No successful solve runs")

    # Sort by duration
    timings.sort(key=lambda x: x["duration"])

    # Statistics
    durations = [t["duration"] for t in timings]
    avg_duration = sum(durations) / len(durations)
    min_duration = min(durations)
    max_duration = max(durations)

    # Report
    print("\n" + "=" * 60)
    print("SOLVE PERFORMANCE BENCHMARK")
    print("=" * 60)
    print(f"Total fixtures:   {len(timings)}")
    print(f"Average time:     {avg_duration:.3f}s")
    print(f"Min time:         {min_duration:.3f}s ({timings[0]['name']})")
    print(f"Max time:         {max_duration:.3f}s ({timings[-1]['name']})")

    print("\nALL TIMINGS:")
    for timing in timings:
        print(f"  {timing['duration']:.3f}s  {timing['name']}")

    # Performance assertions (lenient for now)
    assert avg_duration < 30.0, f"Average solve time too slow: {avg_duration:.3f}s"
