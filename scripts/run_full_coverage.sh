#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ -n "${CALCULIX_PYTHON:-}" ]]; then
  PYTHON_CMD=("${CALCULIX_PYTHON}")
elif command -v uv >/dev/null 2>&1; then
  PYTHON_CMD=(uv run python)
elif [[ -x "$ROOT_DIR/.venv/bin/python" ]] && "$ROOT_DIR/.venv/bin/python" -c "import pytest, gcovr" >/dev/null 2>&1; then
  PYTHON_CMD=("$ROOT_DIR/.venv/bin/python")
else
  PYTHON_CMD=(python3)
fi

if ! "${PYTHON_CMD[@]}" -c "import gcovr" >/dev/null 2>&1; then
  echo "gcovr is required. Install with: python3 -m pip install -r requirements-test.txt" >&2
  exit 1
fi

mkdir -p build/coverage

export CALCULIX_CCX_BIN="${CALCULIX_CCX_BIN:-$ROOT_DIR/ccx_f/ccx_2.23}"
export CALCULIX_CGX_BIN="${CALCULIX_CGX_BIN:-$ROOT_DIR/cgx_c/src/cgx}"
export CALCULIX_REQUIRE_BINARIES=1
export CALCULIX_RUN_FULL=1

"${PYTHON_CMD[@]}" -m pytest --full-matrix --require-binaries

"${PYTHON_CMD[@]}" -m gcovr \
  --root "$ROOT_DIR" \
  --exclude "$ROOT_DIR/tests" \
  --exclude "$ROOT_DIR/glut-3.5" \
  --exclude "$ROOT_DIR/libSNL" \
  --xml-pretty --xml build/coverage/gcovr.xml \
  --html-details build/coverage/gcovr.html \
  --print-summary

echo "Coverage reports:"
echo "  - build/coverage/gcovr.xml"
echo "  - build/coverage/gcovr.html"
