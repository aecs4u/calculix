# Test-Driven Development and Coverage

This repository now includes a TDD-oriented test harness for both:

- `ccx` solver (`tests/test_solver_suite.py`)
- `cgx` viewer (`tests/test_viewer_suite.py`)

## Quick Start

Run the fast local TDD loop:

```bash
./scripts/tdd.sh
```

`./scripts/tdd.sh` will prefer `uv run python`, then `.venv/bin/python`, then system `python3`.

## Test Matrix Modes

Smoke mode (default):

- Runs a small subset of discovered solver/viewer cases.
- Intended for red/green/refactor cycles.

Full matrix mode:

```bash
CALCULIX_RUN_FULL=1 python3 -m pytest --full-matrix
```

This executes the complete discovered example corpus.

## Binary Discovery

Tests auto-detect binaries in:

- `ccx_f/ccx_2.23`
- `cgx_c/src/cgx`
- or `PATH` (`ccx`, `cgx`)

You can override with:

```bash
export CALCULIX_CCX_BIN=/path/to/ccx_2.23
export CALCULIX_CGX_BIN=/path/to/cgx
```

Require binaries (do not skip):

```bash
export CALCULIX_REQUIRE_BINARIES=1
python3 -m pytest --require-binaries
```

## Coverage Workflow (C/Fortran with gcov)

1. Build instrumented binaries:

```bash
./scripts/build_coverage_binaries.sh
```

2. Run full integration matrix and collect reports:

```bash
./scripts/run_full_coverage.sh
```

If `gcovr` is missing, install test dependencies with pip:

```bash
python3 -m pip install -r requirements-test.txt
```

You can force a specific interpreter:

```bash
CALCULIX_PYTHON=.venv/bin/python ./scripts/run_full_coverage.sh
```

Reports:

- `build/coverage/gcovr.xml`
- `build/coverage/gcovr.html`

## TDD Process

Use red/green/refactor for every new behavior:

1. Add/adjust failing test first.
2. Implement the smallest change to pass.
3. Refactor while keeping tests green.
4. Extend matrix with at least one integration case.
5. Keep coverage reports above your release threshold.
