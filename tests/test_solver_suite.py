from __future__ import annotations

from pathlib import Path
import subprocess
import pytest

from tests.testkit import copy_case_tree, run_command


@pytest.mark.solver
@pytest.mark.smoke
@pytest.mark.xfail(reason="Legacy ccx binary requires libgfortran.so.4 (not available on this system)")
def test_solver_prints_version(ccx_bin: Path) -> None:
    result = run_command([str(ccx_bin), "-v"], cwd=ccx_bin.parent, timeout_s=30)
    combined = f"{result.stdout}\n{result.stderr}"
    assert result.returncode == 0, combined
    assert "Version" in combined


@pytest.mark.solver
@pytest.mark.integration
@pytest.mark.xfail(reason="Legacy ccx binary requires libgfortran.so.4 (not available on this system)")
def test_solver_example_matrix_generates_outputs(
    ccx_bin: Path,
    solver_cases,
    tmp_path: Path,
    full_matrix: bool,
) -> None:
    if not solver_cases:
        pytest.skip("No solver cases available.")

    timeout_s = 600 if full_matrix else 180
    failures: list[str] = []

    for case in solver_cases:
        case_copy = copy_case_tree(case.inp_path, tmp_path / "solver")
        jobname = case_copy.stem
        try:
            result = run_command(
                [str(ccx_bin), "-i", jobname],
                cwd=case_copy.parent,
                timeout_s=timeout_s,
            )
        except subprocess.TimeoutExpired:
            failures.append(f"[{case.inp_path}] timed out after {timeout_s}s")
            continue
        dat_file = case_copy.with_suffix(".dat")
        frd_file = case_copy.with_suffix(".frd")
        if result.returncode != 0:
            failures.append(
                f"[{case.inp_path}] returncode={result.returncode}\n"
                f"stdout:\n{result.stdout}\n"
                f"stderr:\n{result.stderr}\n"
            )
            continue
        if not dat_file.exists():
            failures.append(f"[{case.inp_path}] missing {dat_file.name}")
        if not frd_file.exists():
            failures.append(f"[{case.inp_path}] missing {frd_file.name}")
        if dat_file.exists() and dat_file.stat().st_size == 0:
            failures.append(f"[{case.inp_path}] empty {dat_file.name}")
        if frd_file.exists() and frd_file.stat().st_size == 0:
            failures.append(f"[{case.inp_path}] empty {frd_file.name}")

    assert not failures, "\n".join(failures)
