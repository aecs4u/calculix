#!/usr/bin/env bash
set -euo pipefail

# Fast local TDD loop:
# 1) run pure-Python discovery/contract tests
# 2) run solver/viewer smoke tests if binaries are available

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ -n "${CALCULIX_PYTHON:-}" ]]; then
  PYTHON_CMD=("${CALCULIX_PYTHON}")
elif command -v uv >/dev/null 2>&1; then
  PYTHON_CMD=(uv run python)
elif [[ -x "$ROOT_DIR/.venv/bin/python" ]] && "$ROOT_DIR/.venv/bin/python" -c "import pytest" >/dev/null 2>&1; then
  PYTHON_CMD=("$ROOT_DIR/.venv/bin/python")
else
  PYTHON_CMD=(python3)
fi

"${PYTHON_CMD[@]}" -m pytest -m "not integration"
"${PYTHON_CMD[@]}" -m pytest -m "smoke"
