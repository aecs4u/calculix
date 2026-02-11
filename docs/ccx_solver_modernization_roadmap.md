# ccx-solver Libraries Modernization Roadmap (Feature Branch)

**Branch**: `feature/ccx-solver-modernization`  
**Scope**: `ccx-solver`, `ccx-inp`, `ccx-model`, `ccx-io`, `ccx-compat`  
**Primary Goal**: Modernize solver internals for scalability, performance, and interoperability while preserving behavior parity and CLI compatibility.

## 1) Objectives

1. Replace sparse→dense fallback in the solve path with true sparse iterative/direct backends.
2. Introduce backend abstraction for CPU/GPU algebra without destabilizing existing workflows.
3. Modernize I/O and restart state handling for large models and cross-tool interoperability.
4. Keep numerical behavior aligned with existing fixture-based validation.

## 2) Current Gaps (Baseline)

- Sparse assembly exists, but solve currently converts CSR to dense before LU solve.
- Multiple TODOs remain in analysis detection, BC/load coverage, and FRD parsing completeness.
- Restart/state persistence is JSON-only and not optimized for large vector state.
- Parser/postprocessing flow is mostly memory-buffered and not optimized for large files.

## 3) Roadmap Phases

## Phase A — Architecture Hardening (1-2 weeks)

### Deliverables
- Algebra backend trait layer (`LinearOperator`, `LinearSolver`, `Preconditioner`).
- Unified solver entry path (dense and sparse strategies behind one API).
- ADR for backend choices and fallback policy.

### Exit Criteria
- Existing tests pass unchanged.
- No CLI behavior regression for current commands.

## Phase B — Sparse Solver Modernization (2-4 weeks)

### Deliverables
- Native sparse iterative solver path (CG for SPD, BiCGSTAB fallback).
- Basic preconditioning (Jacobi/ILU0 depending on backend support).
- Convergence and residual reporting in solver results.

### Exit Criteria
- Remove sparse→dense conversion from default path.
- Parity on current truss/beam fixture subset within defined tolerances.

## Phase C — Backend Extensibility (CPU/GPU) (2-3 weeks)

### Deliverables
- Pluggable backend selection (`cpu-default`, `cpu-blas`, optional `gpu-cuda`/`gpu-wgpu` features).
- Feature-gated dependency wiring and capability checks.
- Experimental sparse backend adapters:
  - `suitesparse-sys` (or `rusparse`) for SuiteSparse-family direct/iterative capabilities.
  - `petsc-rs` for scalable Krylov + preconditioner workflows and distributed solves.
- Benchmark harness for backend comparison (assembly + solve timing).

### Exit Criteria
- Default build remains lightweight.
- Optional backends compile cleanly when enabled.
- Backend comparison report includes native path vs SuiteSparse/PETSc adapters.

## Phase D — I/O + State Modernization (2-3 weeks)

### Deliverables
- Structured restart schema versioning with migration hooks.
- Optional Arrow IPC restart/result serialization path.
- FRD reader/writer completeness pass for high-frequency datasets used in fixtures.

### Exit Criteria
- Restart round-trip tests pass for JSON and Arrow modes.
- CLI outputs remain backward-compatible by default.

## Phase E — Coverage + Reliability Lift (1-2 weeks)

### Deliverables
- Expanded unit/integration tests for solver branches and IO edge cases.
- Deterministic regression suite for representative fixtures.
- CI checks for clippy, fmt, tests, and optional feature matrix.

### Exit Criteria
- Stable CI across default and selected feature combinations.
- Documented known limitations and follow-up backlog.

## 4) Workstreams and Ownership

- **Numerics**: sparse solve path, preconditioning, tolerances, convergence criteria.
- **Infrastructure**: backend traits, feature flags, CI matrix.
- **External Solvers**: integration layer for `suitesparse-sys`/`rusparse` and `petsc-rs`.
- **I/O**: restart formats, FRD/VTK compatibility, schema migration.
- **QA**: fixture parity, performance regression thresholds, failure triage.
- **Docs**: ADRs, migration notes, backend usage guide.

## 5) Milestones

1. **M1**: Backend trait layer merged, no regressions.
2. **M2**: Sparse-native solve path enabled by default.
3. **M3**: Optional backend feature flags validated in CI.
   - Includes compile validation for `suitesparse-sys`/`rusparse` and `petsc-rs` profiles.
4. **M4**: Restart/IO modernization complete with compatibility mode.
5. **M5**: Release-candidate branch with parity + benchmark report.

## 6) Legacy HPC Solver Policy

| Library / Toolkit | Policy | Integration Path | Target Phase |
|---|---|---|---|
| MUMPS | Adopt (high priority) | Through PETSc/Trilinos first, direct adapter optional | C-E |
| SuperLU / SuperLU_DIST | Adopt | Through PETSc/Trilinos first, direct adapter optional | C-E |
| UMFPACK (SuiteSparse) | Adopt | `suitesparse-sys` / `rusparse` adapter | C-D |
| PaStiX | Adopt (high priority, optional profile) | PETSc/Trilinos path first, direct adapter optional | C-E |
| PARDISO (MKL/Panua) | Adopt (optional/commercial profile) | Feature-gated backend adapter | C-E |
| PETSc | Adopt (strategic toolkit) | `petsc-rs` backend profile | C-E |
| Trilinos | Adopt (secondary strategic toolkit) | C FFI/backend profile (after PETSc profile) | D-E |
| HSL | Optional/commercial | Feature-gated enterprise profile only | E+ |
| pARMS | Conditional | Via PETSc preconditioner path where available | E+ |
| TAUCS | Deprioritize | No direct integration planned | Deferred |
| WSMP | Deprioritize | No direct integration planned | Deferred |
| PSPASES | Deprioritize | No direct integration planned | Deferred |
| BCSLib | Deprioritize | No direct integration planned | Deferred |
| HIPS | Deprioritize | No direct integration planned | Deferred |
| MaPHYS | Investigate | Track ecosystem maturity before adoption | E+ |
| PDSLin | Deprioritize | No direct integration planned | Deferred |

### Phase Gates for HPC Backends

- **Gate C1 (compile gate)**: backend feature flags compile in CI (`native`, `suitesparse`, `pastix`, `petsc`).
- **Gate C2 (smoke gate)**: each enabled backend solves a shared small benchmark set.
- **Gate D1 (parity gate)**: backend results match baseline tolerances for selected fixtures.
- **Gate D2 (performance gate)**: backend profile includes memory/time report vs native path.
- **Gate E1 (release gate)**: only backends meeting reliability + docs criteria are promoted from experimental.

## 7) Success Metrics

- **Correctness**: fixture pass rate and numerical tolerance compliance.
- **Performance**: reduced memory footprint and solve-time improvements on medium models.
- **Stability**: zero critical regressions in CLI flows and parse/output workflows.
- **Maintainability**: clear abstraction boundaries and reduced solver-path duplication.

## 8) Risk Register

1. **Numerical divergence** across backends.  
   **Mitigation**: strict golden comparisons and backend-specific tolerances.
2. **Feature-flag complexity growth**.  
   **Mitigation**: narrow supported matrix and explicit default backend policy.
3. **I/O compatibility drift** with legacy workflows.  
   **Mitigation**: backward-compatible default formats and migration docs/tools.
4. **Over-scoping modernization** before parity.  
   **Mitigation**: phase gates and milestone-based merge policy.

## 9) Branch Policy

- Keep PRs small and phase-aligned.
- Merge only with passing CI + fixture parity checks.
- Avoid simultaneous deep refactors in numerics and I/O in a single PR.
- Tag milestone checkpoints (`m1`…`m5`) for reproducible progress tracking.

## 10) Immediate Next Actions

1. Create ADR: algebra backend abstraction and default backend policy.
2. Implement solver trait interfaces and adapt current sparse assembly path.
3. Add convergence/residual reporting to solver output structs.
4. Stand up a benchmark command/script for repeatable performance baselines.
