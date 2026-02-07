# CalculiX Modernization Development Plan

## 1. Baseline Study Summary

This repository currently contains an upstream-style source layout:

- `ccx_f/` (solver): Fortran + C mixed codebase
- `cgx_c/` (interactive pre/post GUI): C/C++ + GLUT/OpenGL/X11

Observed scale:

- `ccx_f`: 986 Fortran files, 185 C files, ~339k LOC
- `cgx_c/src`: 126 C files, 2 C++ files, ~139k LOC

Build system characteristics:

- Makefile-based, no modular package manager/workspace structure
- Legacy external dependencies and compile-time feature flags
- Limited in-repo documentation beyond upstream install/changelog files

Important architectural facts:

- Solver orchestration starts in `ccx_f/ccx_2.23.c` and calls many Fortran routines via C/Fortran ABI (`FORTRAN(...)` macros in `ccx_f/CalculiX.h`).
- Input parsing and feature dispatch are spread across `ccx_f/readinput.c`, `ccx_f/calinput.f`, and many specialized routines.
- CGX is a monolithic GLUT/X11 app with command interpreter + geometry/mesh/import/export logic in large files (`cgx.c`, `setFunktions.c`, `meshSet.c`, `readccx.c`, etc.).

Implication: this is not a "rewrite in one step" project. It must be executed as staged migration with compatibility checkpoints.

## 2. Goals

Primary goals:

1. Create high-quality technical documentation for current code and behavior.
2. Port solver (`ccx`) to Rust with numerical/behavioral parity.
3. Replace current CGX-style IDE/UI with a PySide6-based application.

Secondary goals:

- Improve maintainability and build reproducibility.
- Preserve existing input/output compatibility where practical.
- Reduce crash/safety risk from legacy memory-management patterns.

## 3. Non-Goals (for first program iteration)

- Re-implementing every historical/experimental solver backend on day one.
- Full UI parity with every legacy menu path before first release.
- Performance superiority before numerical parity.

## 4. Recommended Target Architecture

### 4.1 Solver (Rust)

Create a Rust workspace:

- `crates/ccx-cli` (CLI and job orchestration)
- `crates/ccx-inp` (Abaqus/CalculiX deck parsing + AST)
- `crates/ccx-model` (mesh/material/bc/load domain model)
- `crates/ccx-assembly` (element assembly and sparse matrix generation)
- `crates/ccx-solver` (analysis pipelines: static, modal, dynamic, thermal)
- `crates/ccx-io` (FRD/DAT/STA writers, restart handling)
- `crates/ccx-compat` (temporary C/Fortran FFI bridges during migration)

Linear solver backend via traits:

- direct sparse (default)
- optional high-performance backends (MUMPS/PARDISO/PaStiX wrappers)

### 4.2 IDE/UI (PySide6)

Create a Python application:

- `ide/app` (PySide6 shell, docking, workflow orchestration)
- `ide/model` (project tree, case configs, datasets)
- `ide/mesh` (import/export adapters, meshing orchestration)
- `ide/visual` (3D viewport, result overlays, scalar/vector plots)
- `ide/runner` (job execution, process control, logs, progress)

Strong recommendation:

- Keep heavy numeric/mesh operations in Rust libraries.
- Use Python only for UI orchestration and scripting ergonomics.

## 5. Documentation Workstream (starts first)

Create `docs/` and deliver these artifacts in order:

1. `docs/architecture/current-system.md`
   - Runtime architecture of `ccx` and `cgx`
   - Component boundaries and call graph overviews

2. `docs/build/build-matrix.md`
   - OS/compiler/dependency matrix
   - Feature flags and solver backend matrix

3. `docs/io/formats.md`
   - INP subset used, FRD outputs, restart/state files

4. `docs/behavior/reference-benchmarks.md`
   - Golden benchmark suite definition and tolerances

5. `docs/migration/feature-coverage.md`
   - Feature inventory with parity status (legacy vs Rust vs PySide6)

6. `docs/adrs/` (Architecture Decision Records)
   - major choices: parser design, sparse backend, viewport tech, plugin model

## 6. Migration Strategy

Use a strangler pattern with hard compatibility gates.

### Phase 0: Program Setup (2-4 weeks)

- Freeze baseline versions and dependencies
- Establish CI for current C/Fortran build (Linux first)
- Create benchmark corpus (small/medium/large + multi-physics representative cases)
- Define numerical acceptance criteria per analysis type

Exit criteria:

- Reproducible baseline binaries + benchmark outputs archived

### Phase 1: Documentation + Observability (4-8 weeks)

- Deliver docs listed in section 5
- Instrument baseline runtime metrics (parse time, assembly time, solve time, memory)
- Build feature inventory from `calinput`/command handlers

Exit criteria:

- Documented architecture and benchmark harness in CI

### Phase 2: Rust Foundation + Compatibility Shell (8-12 weeks)

- Initialize Rust workspace
- Implement INP lexer/parser for prioritized feature subset
- Build Rust domain model + FRD writer skeleton
- Add temporary FFI path to call selected legacy routines as fallback

Exit criteria:

- Rust CLI can parse benchmark input and run a minimal end-to-end pipeline for linear static subset

### Phase 3: Solver Core Port (incremental, 6-12 months)

Recommended order:

1. Linear static structural
2. Modal/frequency (ARPACK-equivalent behavior)
3. Nonlinear structural/contact subset
4. Thermal + thermo-mechanical coupling
5. Dynamic procedures

For each procedure:

- Implement in Rust
- Run golden diff against legacy outputs
- Track accuracy/performance regressions

Exit criteria (per procedure):

- Numerical parity within predefined tolerance on benchmark suite
- No critical regressions in runtime/memory against baseline target

### Phase 4: PySide6 IDE MVP (3-6 months, parallelizable)

MVP scope:

- Project/case management
- Input editor with validation
- Run/stop job control and log panel
- Result loading and 3D post-processing (at least scalar contour + deformation)
- Scriptable command console

Legacy compatibility:

- Import existing decks and FRD results
- Preserve common CGX workflows first (not every command initially)

Exit criteria:

- Daily engineering workflow can be completed without legacy CGX for MVP feature set

### Phase 5: Convergence and Decommission (3-6 months)

- Expand feature parity matrix to target coverage
- Deprecate legacy execution path by subsystem
- Publish migration guides for users

Exit criteria:

- Rust solver + PySide6 IDE tagged stable for production use

## 7. Test and Quality Strategy

Mandatory test layers:

1. Parser unit tests (card/parameter combinations)
2. Solver kernel tests (matrix assembly, element routines)
3. Golden integration tests (full job outputs vs baseline)
4. Performance regression tests (time + memory ceilings)
5. UI regression tests (headless smoke + workflow automation)

Golden output policy:

- Compare critical numeric fields with relative/absolute tolerances
- Keep curated per-procedure reference outputs in versioned storage

## 8. Risk Register

Top risks:

1. Scope explosion from very large feature surface (`ccx` keywords + CGX command set)
2. Hidden behavior in legacy code paths with sparse documentation
3. Numerical divergence after porting nonlinear/contact procedures
4. UI rewrite underestimation if command-language compatibility is required
5. Dependency/licensing constraints for solver backend choices

Mitigations:

- Strict phase gates + feature coverage matrix
- Benchmark-first development
- Keep compatibility bridge until each subsystem is proven
- Prioritize high-value workflows before long-tail commands

## 9. Team and Timeline Reality Check

Expected effort for full parity is substantial.

Rough planning ranges:

- Small team (2-3 engineers): 24-36 months
- Medium team (5-7 engineers): 12-24 months

Suggested team shape:

- 2 solver engineers (numerics + Rust systems)
- 1-2 geometry/mesh engineers
- 1 UI engineer (PySide6 + visualization)
- 1 QA/automation engineer
- 1 technical writer/developer advocate (part-time acceptable)

## 10. First 90-Day Execution Backlog

1. Stand up CI for legacy build + benchmark replay.
2. Create `docs/` skeleton and complete architecture/build/io docs.
3. Implement Rust workspace scaffold and INP parser MVP.
4. Define golden benchmark dataset and tolerance framework.
5. Build PySide6 shell app with project/run/log panes.
6. Publish feature-coverage dashboard for migration status.

## 11. Success Metrics

- Documentation coverage: >90% of critical workflows and architecture paths
- Rust solver parity: benchmark pass rate target by phase (start at 30%, then 60%, 85%, 95%+)
- IDE adoption: percentage of internal workflows completed without legacy CGX
- Stability: crash-free runs and deterministic outputs for golden suite

