from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import os
import shutil
import subprocess
from typing import Iterable


@dataclass(frozen=True)
class SolverCase:
    inp_path: Path


@dataclass(frozen=True)
class ViewerCase:
    path: Path
    mode: str


def is_truthy(value: str | None) -> bool:
    if value is None:
        return False
    return value.strip().lower() in {"1", "true", "yes", "on"}


def locate_binary(env_var: str, candidates: Iterable[Path]) -> Path | None:
    env_value = os.getenv(env_var)
    if env_value:
        candidate = Path(env_value).expanduser().resolve()
        if candidate.is_file():
            return candidate

    for candidate in candidates:
        if candidate and candidate.is_file():
            return candidate.resolve()
    return None


def discover_solver_cases(repo_root: Path) -> list[SolverCase]:
    roots = [
        repo_root / "cgx_c" / "examples",
        repo_root / "tests" / "fixtures" / "solver",
    ]
    cases: list[SolverCase] = []
    for root in roots:
        if not root.exists():
            continue
        for inp in root.rglob("*.inp"):
            cases.append(SolverCase(inp.resolve()))
    return sorted(cases, key=lambda c: str(c.inp_path))


def discover_viewer_cases(repo_root: Path) -> list[ViewerCase]:
    mode_by_suffix = {
        ".fbd": "-b",
        ".fbl": "-b",
        ".frd": "-v",
        ".inp": "-c",
    }
    roots = [
        repo_root / "cgx_c" / "examples",
        repo_root / "tests" / "fixtures" / "viewer",
    ]
    cases: list[ViewerCase] = []
    for root in roots:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if not path.is_file():
                continue
            mode = mode_by_suffix.get(path.suffix.lower())
            if mode is None:
                continue
            cases.append(ViewerCase(path=path.resolve(), mode=mode))
    return sorted(cases, key=lambda c: (c.mode, str(c.path)))


def copy_case_tree(source_file: Path, destination_root: Path) -> Path:
    source_file = source_file.resolve()
    source_parent = source_file.parent
    target_parent = destination_root / source_parent.name
    if target_parent.exists():
        shutil.rmtree(target_parent)
    shutil.copytree(source_parent, target_parent)
    return target_parent / source_file.name


def run_command(
    command: list[str],
    *,
    cwd: Path,
    timeout_s: int,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=str(cwd),
        timeout=timeout_s,
        check=False,
        text=True,
        capture_output=True,
    )
