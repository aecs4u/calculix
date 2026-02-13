# PETSc Backend Integration Strategy

**Date**: 2026-02-11
**Status**: Architecture complete, FFI implementation pending

---

## Executive Summary

The PETSc backend architecture is **fully designed and documented** in the `ccx-solver` crate. All traits, configuration types, and API designs are complete with comprehensive pseudo-code. What remains is implementing the FFI layer when `petsc-sys` bindings become available.

---

## Current State: Architecture Complete ✅

### 1. Trait Layer (`backend/traits.rs`) ✅

**Purpose**: Backend-agnostic interface for linear and eigenvalue solvers

```rust
pub trait LinearSolver {
    fn solve_linear(&self, system: &LinearSystemData)
        -> Result<(DVector<f64>, SolveInfo), BackendError>;
}

pub trait EigenSolver {
    fn solve_eigen(&self, system: &EigenSystemData, num_modes: usize)
        -> Result<(EigenResult, SolveInfo), BackendError>;
}

pub trait SolverBackend: LinearSolver + EigenSolver {
    fn name(&self) -> &str;
}
```

**Interchange Format**: `SparseTripletsF64` (COO format)
- Works with both nalgebra-sparse and PETSc
- Efficient assembly, no format lock-in

---

### 2. Configuration System (`backend/petsc_config.rs`) ✅

**Complete enums and presets**:

```rust
// KSP Solver Types
pub enum KspType {
    CG,           // Conjugate Gradient (SPD systems)
    GMRES,        // General systems
    BiCGSTAB,     // Biconjugate Gradient
    TFQMR,        // Transpose-Free QMR
    Richardson,   // Richardson iteration
    Chebyshev,    // Chebyshev iteration
    PreOnly,      // Direct solver via preconditioner
}

// Preconditioner Types
pub enum PcType {
    None, Jacobi, ILU, ICC,    // Basic
    ASM, BJ, SOR,               // Intermediate
    HYPRE,                      // Algebraic multigrid
    LU, Cholesky,               // Direct (via PC)
}

// Direct Solver Libraries
pub enum MatSolverType {
    MUMPS, SuperLU, SuperLUDist,
    PARDISO, PaStiX, UMFPACK, Default,
}
```

**Configuration Presets**:
```rust
PetscConfig::linear_static()    // CG + ICC (SPD systems)
PetscConfig::direct_solver()    // PreOnly + MUMPS
PetscConfig::modal_analysis(10) // SLEPc with 10 modes
PetscConfig::iterative()        // GMRES + ILU
```

---

### 3. Backend Implementation (`backend/petsc.rs`) ✅

**API complete with comprehensive pseudo-code**:

```rust
impl LinearSolver for PetscBackend {
    fn solve_linear(&self, system: &LinearSystemData)
        -> Result<(DVector<f64>, SolveInfo), BackendError> {
        // 1. Create PETSc matrix from COO triplets
        let mat = PetscMat::from_triplets(&system.stiffness)?;

        // 2. Create vectors
        let b = PetscVec::from_dvector(&system.force)?;
        let x = PetscVec::new(system.num_dofs)?;

        // 3. Configure KSP solver
        let ksp = configure_ksp(&mat, &self.config.ksp)?;

        // 4. Solve K * x = b
        ksp.solve(&b, &mut x)?;

        // 5. Extract results
        let displacement = x.to_dvector()?;
        let info = SolveInfo {
            iterations: ksp.get_iteration_number()?,
            residual_norm: Some(ksp.get_residual_norm()?),
            solver_name: format!("PETSc-{:?}", self.config.ksp.solver_type),
        };

        Ok((displacement, info))
    }
}
```

**300+ lines of detailed pseudo-code** documenting:
- KSP configuration with all options
- SLEPc eigenvalue solver setup
- Error handling strategies
- PETSc function call sequences

---

### 4. Wrapper Types (`backend/petsc_wrapper.rs`) ✅

**Designed for RAII-based memory management**:

```rust
pub struct PetscMat {
    mat: Mat,  // Will hold petsc_sys::Mat handle
    nrows: usize,
    ncols: usize,
}

impl Drop for PetscMat {
    fn drop(&mut self) {
        // Automatic cleanup via MatDestroy
    }
}

pub struct PetscVec {
    vec: Vec,  // Will hold petsc_sys::Vec handle
    size: usize,
}
```

**Conversion methods designed**:
- `PetscMat::from_triplets()` - COO → PETSc AIJ
- `PetscVec::from_dvector()` - nalgebra → PETSc
- `PetscVec::to_dvector()` - PETSc → nalgebra

---

## What's Missing: FFI Implementation Only

### Option 1: Create `petsc-sys` Bindings ⏸️

**Approach**: Use `bindgen` to generate Rust FFI bindings to PETSc C API

**Effort**: ~2-3 weeks for comprehensive bindings
**Files needed**:
```
crates/petsc-sys/
├── build.rs          # bindgen configuration
├── Cargo.toml        # FFI dependencies
├── src/lib.rs        # Raw FFI declarations
└── wrapper.h         # PETSc headers to bind
```

**Key PETSc functions to bind**:
```rust
// Matrix operations
extern "C" {
    fn MatCreate(comm: MPI_Comm, mat: *mut Mat) -> PetscErrorCode;
    fn MatSetSizes(mat: Mat, m: PetscInt, n: PetscInt, M: PetscInt, N: PetscInt) -> PetscErrorCode;
    fn MatSetType(mat: Mat, mat_type: *const c_char) -> PetscErrorCode;
    fn MatSetValues(mat: Mat, m: PetscInt, idxm: *const PetscInt, n: PetscInt,
                    idxn: *const PetscInt, v: *const PetscScalar,
                    addv: InsertMode) -> PetscErrorCode;
    fn MatAssemblyBegin(mat: Mat, assembly_type: MatAssemblyType) -> PetscErrorCode;
    fn MatAssemblyEnd(mat: Mat, assembly_type: MatAssemblyType) -> PetscErrorCode;
    fn MatDestroy(mat: *mut Mat) -> PetscErrorCode;
}

// KSP operations
extern "C" {
    fn KSPCreate(comm: MPI_Comm, ksp: *mut KSP) -> PetscErrorCode;
    fn KSPSetOperators(ksp: KSP, Amat: Mat, Pmat: Mat) -> PetscErrorCode;
    fn KSPSetType(ksp: KSP, ksp_type: *const c_char) -> PetscErrorCode;
    fn KSPGetPC(ksp: KSP, pc: *mut PC) -> PetscErrorCode;
    fn KSPSetTolerances(ksp: KSP, rtol: PetscReal, atol: PetscReal,
                        dtol: PetscReal, maxits: PetscInt) -> PetscErrorCode;
    fn KSPSolve(ksp: KSP, b: Vec, x: Vec) -> PetscErrorCode;
    fn KSPGetIterationNumber(ksp: KSP, its: *mut PetscInt) -> PetscErrorCode;
    fn KSPGetResidualNorm(ksp: KSP, rnorm: *mut PetscReal) -> PetscErrorCode;
    fn KSPDestroy(ksp: *mut KSP) -> PetscErrorCode;
}

// PC operations
extern "C" {
    fn PCSetType(pc: PC, pc_type: *const c_char) -> PetscErrorCode;
    fn PCFactorSetMatSolverType(pc: PC, stype: *const c_char) -> PetscErrorCode;
    fn PCFactorSetLevels(pc: PC, levels: PetscInt) -> PetscErrorCode;
}

// SLEPc operations (for eigenvalue problems)
extern "C" {
    fn EPSCreate(comm: MPI_Comm, eps: *mut EPS) -> PetscErrorCode;
    fn EPSSetOperators(eps: EPS, A: Mat, B: Mat) -> PetscErrorCode;
    fn EPSSetProblemType(eps: EPS, prob_type: EPSProblemType) -> PetscErrorCode;
    fn EPSSetWhichEigenpairs(eps: EPS, which: EPSWhich) -> PetscErrorCode;
    fn EPSSetDimensions(eps: EPS, nev: PetscInt, ncv: PetscInt, mpd: PetscInt) -> PetscErrorCode;
    fn EPSSolve(eps: EPS) -> PetscErrorCode;
    fn EPSGetConverged(eps: EPS, nconv: *mut PetscInt) -> PetscErrorCode;
    fn EPSGetEigenvalue(eps: EPS, i: PetscInt, eigr: *mut PetscScalar,
                        eigi: *mut PetscScalar) -> PetscErrorCode;
    fn EPSGetEigenvector(eps: EPS, i: PetscInt, Vr: Vec, Vi: Vec) -> PetscErrorCode;
    fn EPSDestroy(eps: *mut EPS) -> PetscErrorCode;
}
```

**Environment requirements**:
```bash
export PETSC_DIR=/path/to/petsc
export PETSC_ARCH=arch-linux-c-opt
export LD_LIBRARY_PATH=$PETSC_DIR/$PETSC_ARCH/lib:$LD_LIBRARY_PATH
```

---

### Option 2: Use `kryst` Crate as Intermediate Solution ⏸️

**Found**: `kryst = "3.2.1"` - Krylov subspace solvers for Rust

**Pros**:
- Pure Rust, no FFI complexity
- CG, GMRES, BiCGSTAB available
- Works with nalgebra-sparse

**Cons**:
- No direct solvers (MUMPS, PARDISO, etc.)
- No SLEPc eigenvalue integration
- Less mature than PETSc

**Integration approach**:
```rust
use kryst::prelude::*;

impl LinearSolver for KrystBackend {
    fn solve_linear(&self, system: &LinearSystemData)
        -> Result<(DVector<f64>, SolveInfo), BackendError> {
        // Convert to CsrMatrix
        let a = CsrMatrix::from(&system.stiffness);

        // Configure solver
        let solver = match self.config.solver_type {
            SolverType::CG => Cg::new(&a).with_precond(ILU::new(&a)),
            SolverType::GMRES => Gmres::new(&a).with_precond(ILU::new(&a)),
            SolverType::BiCGSTAB => BiCgStab::new(&a).with_precond(ILU::new(&a)),
        };

        // Solve
        let x = solver.solve(&system.force)?;

        Ok((x, SolveInfo { ... }))
    }
}
```

---

### Option 3: Mock Implementation (Current) ✅

**Status**: Returns helpful error messages

```rust
#[cfg(not(feature = "petsc"))]
{
    Err(BackendError(
        "PETSc backend not compiled. Rebuild with --features petsc or use native backend."
    ))
}
```

**Benefit**: API is ready, users can prepare code for PETSc

---

## Integration Roadmap

### Phase 1: FFI Bindings (2-3 weeks)

1. Create `crates/petsc-sys/` crate with bindgen
2. Bind core Mat/Vec/KSP functions
3. Add error handling wrapper
4. Test basic matrix creation and solve

### Phase 2: Wrapper Implementation (1-2 weeks)

5. Implement `PetscMat::from_triplets()`
6. Implement `PetscVec` conversions
7. Add RAII drop handlers
8. Comprehensive tests

### Phase 3: KSP Integration (1-2 weeks)

9. Implement `configure_ksp()` function
10. Add all solver types (CG, GMRES, BiCGSTAB, etc.)
11. Add all preconditioners (Jacobi, ILU, ICC, ASM, HYPRE)
12. Add direct solver support (MUMPS, SuperLU, PARDISO)
13. Validate with test fixtures

### Phase 4: SLEPc Integration (1-2 weeks)

14. Bind SLEPc EPS functions
15. Implement `configure_eps()` function
16. Add eigenvalue extraction
17. Validate modal analysis

### Phase 5: Production Hardening (1 week)

18. Comprehensive error handling
19. Performance benchmarking vs native backend
20. Documentation and examples
21. CI/CD integration

**Total Estimated Effort**: 7-10 weeks

---

## Alternative: Use Native Backend Enhanced

While PETSc integration is pending, enhance the native backend:

1. ✅ **Direct sparse solve** - Already uses nalgebra-lapack LU
2. ⏸️ **Iterative CG** - Add conjugate gradient for large SPD systems
3. ⏸️ **Iterative GMRES** - Add via `kryst` crate
4. ⏸️ **Better preconditioners** - ILU, ICC implementations

This provides a production-ready path without external dependencies.

---

## Testing Strategy

### Unit Tests (When FFI Available)

```rust
#[cfg(feature = "petsc")]
mod petsc_tests {
    #[test]
    fn test_mat_from_triplets() {
        let triplets = SparseTripletsF64 { /* ... */ };
        let mat = PetscMat::from_triplets(&triplets).unwrap();
        assert_eq!(mat.shape(), (3, 3));
    }

    #[test]
    fn test_ksp_cg_solve() {
        // SPD system, should converge with CG
        let config = KspConfig {
            solver_type: KspType::CG,
            precond_type: PcType::ICC,
            ..Default::default()
        };
        // ... solve and validate
    }

    #[test]
    fn test_mumps_direct_solve() {
        let config = PetscConfig::direct_solver();
        // ... solve and compare with reference
    }
}
```

### Integration Tests

```rust
#[test]
fn test_cantilever_beam_petsc_vs_native() {
    let mesh = load_mesh("beamcr4.inp");

    // Solve with native backend
    let native_result = solve_with_backend(&mesh, NativeBackend::new());

    // Solve with PETSc backend
    let petsc_result = solve_with_backend(&mesh, PetscBackend::new());

    // Results should match within tolerance
    assert_relative_eq!(native_result, petsc_result, epsilon = 1e-6);
}
```

---

## Current Capabilities Summary

| Feature | Native Backend | PETSc Backend | Status |
|---------|---------------|---------------|--------|
| **Linear Solve** | ✅ LU factorization | ⏸️ KSP + many solvers | Architecture ready |
| **Iterative CG** | ❌ | ⏸️ Yes | Pending FFI |
| **Iterative GMRES** | ❌ | ⏸️ Yes | Pending FFI |
| **Direct MUMPS** | ❌ | ⏸️ Yes | Pending FFI |
| **Direct PARDISO** | ❌ | ⏸️ Yes | Pending FFI |
| **Preconditioners** | ❌ | ⏸️ Many (ILU, ICC, ASM, HYPRE) | Pending FFI |
| **Eigenvalue (Dense)** | ✅ LAPACK | ⏸️ SLEPc | Architecture ready |
| **Eigenvalue (Sparse)** | ❌ | ⏸️ SLEPc | Pending FFI |
| **Parallel (MPI)** | ❌ | ⏸️ Yes | Pending FFI |
| **GPU Support** | ❌ | ⏸️ Via PETSc | Pending FFI |

---

## Recommendation

**Short Term** (Now):
1. ✅ **Use native backend** for development and testing
2. ⏸️ **Enhance native backend** with iterative solvers (`kryst`)
3. ⏸️ **Document PETSc integration** for when FFI is available

**Medium Term** (3-6 months):
4. ⏸️ **Create `petsc-sys` crate** with comprehensive bindings
5. ⏸️ **Implement PETSc backend** following existing pseudo-code
6. ⏸️ **Validate with test fixtures**

**Long Term** (6-12 months):
7. ⏸️ **SLEPc integration** for large-scale eigenvalue problems
8. ⏸️ **MPI parallelism** for distributed memory systems
9. ⏸️ **GPU acceleration** via PETSc's CUDA/HIP backends

---

## Resources

- **PETSc Documentation**: https://petsc.org/release/docs/
- **SLEPc Documentation**: https://slepc.upv.es/documentation/
- **Current Implementation**: `crates/ccx-solver/src/backend/petsc.rs`
- **Configuration**: `crates/ccx-solver/src/backend/petsc_config.rs`
- **Traits**: `crates/ccx-solver/src/backend/traits.rs`

---

## Questions for User

1. **PETSc availability**: Is PETSc installed on your system? (`echo $PETSC_DIR`)
2. **Priority**: Should we focus on creating FFI bindings, or use native backend for now?
3. **Intermediate solution**: Should we integrate `kryst` crate for iterative solvers?
4. **Timeline**: Is PETSc integration critical for immediate work, or can it be deferred?
5. **Validation**: Can we proceed with native backend for Phase 1-3 of the migration?

---

**Status**: Architecture complete, awaiting decision on FFI implementation approach.
