# CalculiX 2.23 C Source Migration - Status Report

**Date**: 2026-02-11
**Plan**: `/home/emanuele/.claude/plans/sequential-meandering-pony.md`

---

## Executive Summary

Migration of 185 C source files from CalculiX 2.23 to Rust `ccx-solver` is underway following a 5-phase plan with PETSc backend priority. Current focus: Phase 1 (Critical Solver Infrastructure).

---

## Phase 1: Critical Solver Infrastructure (Weeks 1-4)

### 1.1 Core Utilities & Helpers ✅ **COMPLETE**

**Status**: All string utility functions have been ported and tested.

| C File | Rust Module | Lines | Tests | Status |
|--------|-------------|-------|-------|--------|
| `strcmp2.c` | `ported/string_utils.rs` | ~50 | 7 | ✅ Done |
| `strcpy1.c` | `ported/string_utils.rs` | ~50 | 6 | ✅ Done |
| `strcpy2.c` | `ported/string_utils.rs` | ~50 | 7 | ✅ Done |
| `stos.c` | `ported/string_utils.rs` | ~50 | 4 | ✅ Done |
| `strsplt.c` | `ported/string_utils.rs` | ~70 | 6 | ✅ Done |

**Total**: 5 files, 270 lines, 30 tests
**File**: [crates/ccx-solver/src/ported/string_utils.rs](crates/ccx-solver/src/ported/string_utils.rs)

**Key Functions**:
```rust
pub fn strcmp2(s1: &str, s2: &str, length: usize) -> i32
pub fn strcpy1(src: &str, length: usize) -> String
pub fn strcpy2(src: &str, length: usize) -> String
pub fn stos(string: &str, a: usize, b: usize) -> String
pub fn stos_inv(source: &str, a: usize, b: usize, target_len: usize) -> String
pub fn strsplt(input: &str, delimiter: char) -> Vec<String>
```

---

### 1.2 Matrix Operations Core ⏳ **IN PROGRESS**

**Goal**: Port low-level matrix manipulation routines

#### Analysis of C Files

| C File | Purpose | Rust Equivalent? | Priority |
|--------|---------|------------------|----------|
| `matrixsort.c` | COO→CSR conversion, column sorting | ✅ `nalgebra-sparse` handles this | LOW |
| `transpose.c` | Sparse matrix transpose | ✅ `nalgebra-sparse::CsrMatrix::transpose()` | LOW |
| `convert2rowbyrow.c` | Format conversion for PARDISO | ⚠️ Needed if using PARDISO directly | MEDIUM |
| `insert.c` | Insert entry into sparse matrix | ✅ `CooMatrix::push()` | LOW |
| `insertas.c` | Asymmetric matrix insert | ✅ `CooMatrix::push()` | LOW |
| `insertas_ws.c` | Symmetric matrix insert | ✅ `CooMatrix::push()` | LOW |
| `insertcbs.c` | CBS (?) matrix insert | ❓ Unknown use case | LOW |

**Assessment**: Most matrix operations are already handled by `nalgebra-sparse`. The existing `sparse_assembly.rs` uses:
- **COO (Coordinate)** format for assembly
- **CSR (Compressed Sparse Row)** format for solving
- Automatic conversion via `CsrMatrix::from(&CooMatrix)`

**Recommendation**:
- ✅ **Skip direct ports** - Use `nalgebra-sparse` abstractions
- ⚠️ **Port only if needed** for PETSc/PARDISO integration
- Focus on **Phase 2 (PETSc backend)** instead

---

### 1.3 Residual & Force Computation ⏸️ **PENDING**

**Goal**: Port residual and force calculation routines

| C File | Purpose | Priority |
|--------|---------|----------|
| `calcresidual.c` | Main residual assembly | HIGH |
| `calcshapef.c` | Shape function evaluation | HIGH |
| `rhsmain.c` | RHS force vector | HIGH |
| `dfdbj.c` | Jacobian derivatives | MEDIUM |
| `resforccont.c` | Contact forces | MEDIUM |
| `radflowload.c` | Radiation loads | LOW |

**Status**: Not started
**Dependencies**: Requires existing `assembly.rs` and element modules

---

## Phase 2: Assembly & PETSc Integration ⏸️ **PENDING**

### 2.1 Assembly & Matrix Structure (23 files)

| C File | Purpose | Status |
|--------|---------|--------|
| `mastruct.c` | Structure global system matrices | ⏸️ Pending |
| `mastructmatrix.c` | Stiffness matrix structure | ⏸️ Pending |
| `mafillsmmain.c` | Mass matrix assembly | ⏸️ Pending |
| Others... | | |

### 2.2 PETSc Integration (9 files)

**Priority**: **HIGH** (User-specified focus)

Current state:
- ✅ Basic scaffolding exists in `backend/petsc.rs`
- ❌ No KSP (Krylov Subspace) integration
- ❌ No PC (Preconditioner) integration
- ❌ No SLEPc (Eigenvalue solver) integration

**What needs to be done**:

1. **Mat/Vec Wrappers**:
   ```rust
   pub struct PetscMat { /* wrapper */ }
   pub struct PetscVec { /* wrapper */ }

   impl PetscMat {
       pub fn from_csr(csr: &CsrMatrix<f64>) -> Self { ... }
       pub fn create_aij(rows: usize, cols: usize, nnz_per_row: usize) -> Self { ... }
   }
   ```

2. **KSP (Linear Solver)**:
   ```rust
   pub enum KspType {
       CG,           // Conjugate Gradient
       GMRES,        // Generalized Minimal Residual
       BiCGStab,     // Biconjugate Gradient Stabilized
       Direct(MatSolverType),  // MUMPS, SuperLU, PARDISO
   }

   pub struct KspSolver {
       ksp_type: KspType,
       pc_type: PcType,
       max_iter: usize,
       tolerance: f64,
   }
   ```

3. **PC (Preconditioner)**:
   ```rust
   pub enum PcType {
       None,
       Jacobi,
       ILU(usize),  // ILU(k) incomplete LU
       ASM,         // Additive Schwarz Method
       Hypre,       // Algebraic multigrid
   }
   ```

4. **SLEPc (Eigenvalue)**:
   ```rust
   pub struct EigenSolver {
       num_eigenvalues: usize,
       which: WhichEigenvalues,
       tolerance: f64,
   }

   pub enum WhichEigenvalues {
       SmallestMagnitude,
       LargestMagnitude,
       SmallestReal,
       LargestReal,
       Target(f64),
   }
   ```

**Integration Point**:
- Extend `backend/mod.rs` with PETSc dispatcher
- Add `PetscBackend` alongside `NativeBackend`
- Wire into `analysis.rs` solver selection

**C Files to integrate**:
- `pardiso.c` → PETSc MatSolverType::PARDISO
- `spooles.c` → PETSc MatSolverType::SPOOLES
- `pastix.c` → PETSc MatSolverType::PASTIX
- `arpack.c` → PETSc SLEPc integration
- `pcgsolver.c` → PETSc KSP with CG + ILU

---

## Phases 3-5: Future Work

### Phase 3: Nonlinear & Dynamic Analysis ⏸️
- Nonlinear geometry (10 files)
- Contact mechanics (13 files)
- Status: Pending Phases 1-2

### Phase 4: I/O & Post-Processing ⏸️
- FRD output (20 files)
- Results processing (15 files)
- Status: Pending Phases 1-3

### Phase 5: Specialized Features ⏸️
- Material models (4 files)
- Optimization (9 files)
- Electromagnetics (7 files)
- Status: Pending Phases 1-4

---

## Current Infrastructure (Already Implemented)

### Rust Solver Core (~6,000 LOC)

| Module | Lines | Status |
|--------|-------|--------|
| `assembly.rs` | 876 | ✅ Dense assembly |
| `sparse_assembly.rs` | 459 | ✅ CSR sparse assembly |
| `analysis.rs` | 468 | ✅ Analysis pipeline |
| `boundary_conditions.rs` | 313 | ✅ BCs and loads |
| `distributed_loads.rs` | 300 | ✅ Distributed loads |
| `materials.rs` | 528 | ✅ Material library |
| `mesh.rs` | 408 | ✅ Mesh data structures |
| `postprocess.rs` | 574 | ✅ Stress/strain extraction |
| `modal_solver.rs` | 475 | ✅ Eigenvalue solver (native) |
| `dynamic_solver.rs` | 663 | ✅ Newmark time integration |
| `nonlinear_solver.rs` | 326 | ✅ Newton-Raphson |
| `elements/` | 4,553 | ✅ T3D2, T3D3, B31, B32, S4, C3D8 |
| `ported/` | 1,500 | ✅ String utils, sorting, etc. |

### Test Suite (206+ tests, 75% pass rate)

- Unit tests: 163 tests
- Integration tests: 43 tests
- Validation fixtures: 638 `.inp` files
- Reference outputs: 629 `.dat.ref` files
- Example files: 1,133 INP files (99.6% parse success)

---

## Recommended Next Steps

### Immediate (This Week)

1. ✅ **Complete Phase 1.1** (String utils) - DONE
2. ⚠️ **Skip Phase 1.2** (Matrix ops) - Use `nalgebra-sparse` instead
3. 🔄 **Begin Phase 2.2** (PETSc integration) - HIGH PRIORITY

### Short Term (Next 2 Weeks)

1. Implement PETSc Mat/Vec wrappers
2. Integrate PETSc KSP (linear solver)
3. Add PC (preconditioner) support
4. Integrate SLEPc (eigenvalue solver)
5. Create `PetscBackend` in `backend/mod.rs`
6. Add configuration system for solver selection
7. Validate with test fixtures

### Medium Term (Weeks 3-4)

1. Port Phase 1.3 residual computation routines
2. Port Phase 2.1 assembly structure routines
3. Integrate with PETSc backend
4. Comprehensive validation suite

---

## Key Decisions

### ✅ **Decision 1**: Use `nalgebra-sparse` for matrix operations
- **Rationale**: Already implemented, well-tested, idiomatic Rust
- **Impact**: Skip direct ports of `matrixsort.c`, `transpose.c`, etc.
- **Benefit**: Focus on higher-level solver integration

### ✅ **Decision 2**: PETSc priority over native backend
- **Rationale**: User-specified requirement, access to 20+ solver libraries
- **Impact**: Focus Phase 2 effort on PETSc KSP/PC/SLEPc
- **Benefit**: Production-ready solver ecosystem

### ⏸️ **Decision 3**: Defer specialized features to Phase 5
- **Rationale**: Core solver capabilities more critical
- **Impact**: Electromagnetics, optimization deferred
- **Benefit**: Focused incremental progress

---

## Resources

- **Legacy source**: `calculix_migration_tooling/ccx_2.23/src/` (185 C files)
- **Migration plan**: `/home/emanuele/.claude/plans/sequential-meandering-pony.md`
- **Current solver**: `crates/ccx-solver/` (6,000+ LOC)
- **Test fixtures**: `tests/fixtures/solver/` (638 INP files)
- **Validation**: `validation/solver/` (629 DAT.ref files)
- **Documentation**: See [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)

---

## Progress Metrics

| Metric | Current | Target (Phase 1-2) |
|--------|---------|-------------------|
| **C files ported** | 5 / 185 (3%) | 34 / 185 (18%) |
| **Tests passing** | 206+ (75%) | 250+ (80%) |
| **Fixtures validated** | 5 / 638 (1%) | 50 / 638 (8%) |
| **Element types** | 6 / 40 (15%) | 8 / 40 (20%) |
| **Analysis types** | 4 / 16 (25%) | 6 / 16 (38%) |
| **Lines of code** | ~6,000 | ~8,000 |

---

## Questions for User

1. **PETSc availability**: Is PETSc installed on the system? If not, should we prioritize native solver improvements?
2. **PARDISO license**: Do you have access to Intel MKL PARDISO, or should we use open-source alternatives (MUMPS, SuperLU)?
3. **Performance targets**: What's the acceptable performance regression vs legacy CCX? (Currently suggesting < 10%)
4. **Validation priority**: Should we validate each module incrementally, or complete phases before validation?
5. **Parallel computing**: Should we prioritize MPI/thread parallelism now, or defer to Phase 5?

---

## Contact

**Repository**: `/mnt/developer/git/aecs4u.it/calculix`
**Branch**: `feature/ccx223-build-scripts`
**Plan File**: `/home/emanuele/.claude/plans/sequential-meandering-pony.md`

For questions or to start specific migration tasks, refer to this status document.
