# Feature Comparison Summary (Legacy vs Rust Migration)

This document summarizes current feature coverage across:

- Legacy `ccx` / `cgx` codebases
- Rust migration crates (`ccx-inp`, `ccx-model`, `ccx-solver`, `calculix_gui`)
- CLI surface (`ccx-cli`)

Snapshot date: **2026-02-08**

## High-level status

| Area | Legacy status | Rust migration status | Notes |
|---|---|---|---|
| INP parsing | Full historical behavior in `ccx` | Partial, bootstrap parser implemented | Good fixture parse coverage; semantic parity still in progress |
| Include expansion | Supported in legacy deck flow | Supported with nested includes and cycle detection | Implemented in `Deck::parse_file_with_includes` |
| Model extraction | Full solver-internal model build | Summary-level extraction only | Counts/flags available; full FE assembly not yet ported |
| Solver routines | Complete legacy solver | Early utility routine ports only | Core analysis pipeline still pending |
| GUI routines (CGX) | Complete legacy GUI stack | Early utility/math ports | Full PySide6 replacement is future phase |
| CLI orchestration | Legacy command/batch workflows | Migration-oriented CLI available | Analysis + migration reports available now |

## Current measurable coverage

### Solver source migration inventory (`ccx-solver`)

From `ccx-cli migration-report`:

- `legacy_units_total`: **1199**
- `superseded_fortran_units`: **986**
- `ported_units`: **9**
- `pending_units`: **213**

Ported units currently listed:

- `compare.c`
- `strcmp1.c`
- `stof.c`
- `stoi.c`
- `superseded/bsort.f`
- `superseded/cident.f`
- `superseded/insertsortd.f`
- `superseded/nident.f`
- `superseded/nident2.f`

### GUI source migration inventory (`calculix_gui`)

From `ccx-cli gui-migration-report`:

- `legacy_gui_units_total`: **136**
- `ported_gui_units`: **11**
- `pending_gui_units`: **125**

Ported GUI units currently listed:

- `compare.c`
- `compareStrings.c`
- `strfind.c`
- `checkIfNumber.c`
- `v_add.c`
- `v_prod.c`
- `v_result.c`
- `v_sprod.c`
- `v_norm.c`
- `v_angle.c`
- `p_angle.c`

### Fixture parse pass status (`ccx-inp`)

From `ccx-cli analyze-fixtures tests/fixtures/solver`:

- `total_inp`: **638**
- `parse_ok`: **638**
- `parse_failed`: **0**

This indicates strong parser robustness on the current solver fixture corpus, but does **not** yet imply numerical solver parity.

## Feature matrix (detailed)

| Capability | Legacy | Rust status | Verification path |
|---|---|---|---|
| Parse basic cards/data lines | ✅ | ✅ | `ccx-inp` unit tests + CLI `analyze` |
| Parse header continuation lines | ✅ | ✅ | `ccx-inp::tests::parses_header_continuation` |
| Handle comment variants (`**`, `>**`) | ✅ | ✅ | parser tests |
| Expand nested `*INCLUDE` trees | ✅ | ✅ | include expansion tests |
| Detect include cycles | implicit/behavioral | ✅ explicit error | include cycle test |
| Parse all regression fixtures | ✅ | ✅ parse-level | CLI `analyze-fixtures` |
| Build full FE model for solving | ✅ | ⚠️ partial | currently summary-level only |
| Run static/dynamic/frequency solve | ✅ | ❌ not yet | future solver phases |
| Emit production-equivalent results | ✅ | ❌ not yet | parity benchmarks pending |
| GUI interactive workflows | ✅ | ❌ (new PySide6 path pending) | planned in migration phases |

## Immediate gaps

- No end-to-end Rust FE solve pipeline yet (assembly/solver/output parity pending).
- `ccx-model` currently provides structural summaries, not full analysis-ready data structures.
- GUI migration currently covers low-level utilities, not user-facing workflows.

## Recommended next comparison checkpoints

1. Add keyword-level support matrix (`keyword -> parse status -> model status -> solve status`).
2. Add benchmark parity table for first target procedure (linear static).
3. Add CLI machine-readable output (`--json`) for automated coverage dashboards.

