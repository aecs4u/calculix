from __future__ import annotations

from pathlib import Path
import os
import shutil
import pytest

from tests.testkit import (
    discover_solver_cases,
    discover_viewer_cases,
    is_truthy,
    locate_binary,
)


def pytest_addoption(parser: pytest.Parser) -> None:
    parser.addoption(
        "--full-matrix",
        action="store_true",
        default=False,
        help="Run all discovered integration cases",
    )
    parser.addoption(
        "--require-binaries",
        action="store_true",
        default=False,
        help="Fail instead of skip when ccx/cgx binaries are not found",
    )


@pytest.fixture(scope="session")
def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


@pytest.fixture(scope="session")
def full_matrix(pytestconfig: pytest.Config) -> bool:
    cli_value = bool(pytestconfig.getoption("--full-matrix"))
    env_value = is_truthy(os.getenv("CALCULIX_RUN_FULL"))
    return cli_value or env_value


@pytest.fixture(scope="session")
def require_binaries(pytestconfig: pytest.Config) -> bool:
    cli_value = bool(pytestconfig.getoption("--require-binaries"))
    env_value = is_truthy(os.getenv("CALCULIX_REQUIRE_BINARIES"))
    return cli_value or env_value


@pytest.fixture(scope="session")
def ccx_bin(repo_root: Path, require_binaries: bool) -> Path:
    candidates = [
        repo_root / "dist" / "ccx_2.23",
        repo_root / "ccx_f" / "ccx_2.23",
        Path(shutil.which("ccx") or ""),
    ]
    binary = locate_binary("CALCULIX_CCX_BIN", candidates)
    if binary is None:
        message = (
            "ccx binary not found. Set CALCULIX_CCX_BIN or build ccx_f/ccx_2.23."
        )
        if require_binaries:
            pytest.fail(message)
        pytest.skip(message)
    return binary


@pytest.fixture(scope="session")
def cgx_bin(repo_root: Path, require_binaries: bool) -> Path:
    candidates = [
        repo_root / "dist" / "cgx_2.23",
        repo_root / "cgx_c" / "src" / "cgx",
        Path(shutil.which("cgx") or ""),
    ]
    binary = locate_binary("CALCULIX_CGX_BIN", candidates)
    if binary is None:
        message = (
            "cgx binary not found. Set CALCULIX_CGX_BIN or build cgx_c/src/cgx."
        )
        if require_binaries:
            pytest.fail(message)
        pytest.skip(message)
    return binary


@pytest.fixture(scope="session")
def solver_cases(repo_root: Path, full_matrix: bool):
    cases = discover_solver_cases(repo_root)
    if full_matrix:
        return cases
    return cases[:3]


@pytest.fixture(scope="session")
def viewer_cases(repo_root: Path, full_matrix: bool):
    cases = discover_viewer_cases(repo_root)
    if full_matrix:
        return cases
    return cases[:6]
