from __future__ import annotations

from pathlib import Path
import subprocess
import pytest

from tests.testkit import copy_case_tree, run_command


@pytest.mark.viewer
@pytest.mark.smoke
def test_viewer_prints_general_info(cgx_bin: Path) -> None:
    # The binary prints usage text when started without valid args.
    result = run_command([str(cgx_bin)], cwd=cgx_bin.parent, timeout_s=30)
    combined = f"{result.stdout}\n{result.stderr}"
    assert "usage: cgx" in combined.lower()


@pytest.mark.viewer
@pytest.mark.integration
def test_viewer_example_matrix_runs_without_crash(
    cgx_bin: Path,
    viewer_cases,
    tmp_path: Path,
    full_matrix: bool,
) -> None:
    if not viewer_cases:
        pytest.skip("No viewer cases available.")

    timeout_s = 300 if full_matrix else 120
    failures: list[str] = []

    for case in viewer_cases:
        case_copy = copy_case_tree(case.path, tmp_path / "viewer")
        try:
            result = run_command(
                [str(cgx_bin), "-bg", case.mode, case_copy.name],
                cwd=case_copy.parent,
                timeout_s=timeout_s,
            )
        except subprocess.TimeoutExpired:
            failures.append(f"[{case.path}] mode={case.mode} timed out after {timeout_s}s")
            continue
        if result.returncode != 0:
            failures.append(
                f"[{case.path}] mode={case.mode} returncode={result.returncode}\n"
                f"stdout:\n{result.stdout}\n"
                f"stderr:\n{result.stderr}\n"
            )

    assert not failures, "\n".join(failures)
