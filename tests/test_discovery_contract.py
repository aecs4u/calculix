from __future__ import annotations

from pathlib import Path

from tests.testkit import discover_solver_cases, discover_viewer_cases


def test_solver_case_discovery(repo_root: Path) -> None:
    cases = discover_solver_cases(repo_root)
    assert cases, "No solver .inp cases discovered."
    assert all(case.inp_path.suffix.lower() == ".inp" for case in cases)


def test_viewer_case_discovery(repo_root: Path) -> None:
    cases = discover_viewer_cases(repo_root)
    assert cases, "No viewer cases discovered."
    assert all(case.mode in {"-b", "-c", "-v"} for case in cases)
