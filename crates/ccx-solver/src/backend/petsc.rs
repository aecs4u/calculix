//! PETSc backend (feature-gated).
//!
//! Provides access to PETSc's KSP (Krylov Subspace) solvers, preconditioners,
//! and through PETSc, also to:
//! - MUMPS (parallel direct solver)
//! - SuperLU / SuperLU_DIST (direct solver)
//! - PaStiX (direct solver)
//! - Iterative methods: CG, GMRES, BiCGSTAB, TFQMR, etc.
//! - Preconditioners: ILU, ICC, AMG (HYPRE/BoomerAMG), Jacobi, etc.
//! - Eigenvalue solvers via SLEPc integration
//!
//! # Requirements
//!
//! - PETSc 3.15+ installed on the system
//! - `PETSC_DIR` and `PETSC_ARCH` environment variables set
//! - Build with `cargo build --features petsc`
//!
//! # Architecture
//!
//! ```text
//! Assembly Layer
//!      │
//!      ├─► COO Triplets (SparseTripletsF64)
//!      └─► Force Vector (DVector<f64>)
//!             │
//!             ▼
//!      PetscBackend::solve_linear
//!             │
//!      ┌──────┴──────┐
//!      │             │
//!      ▼             ▼
//!  PetscMat      PetscVec
//!  (from COO)    (from DVector)
//!      │             │
//!      └──────┬──────┘
//!             │
//!             ▼
//!         KSP Solver
//!      ┌──────┴──────┐
//!      │             │
//!      ▼             ▼
//!  Iterative     Direct
//!  (GMRES+ILU)   (MUMPS/SuperLU)
//!      │             │
//!      └──────┬──────┘
//!             │
//!             ▼
//!        Solution Vec
//!             │
//!             ▼
//!      DVector<f64>
//! ```
//!
//! # Example Usage
//!
//! ```ignore
//! use ccx_solver::backend::{PetscBackend, PetscConfig, LinearSolver};
//!
//! // Option 1: Default GMRES + ILU
//! let backend = PetscBackend::new()?;
//!
//! // Option 2: Direct solver (MUMPS)
//! let backend = PetscBackend::with_config(PetscConfig::direct_solver())?;
//!
//! // Option 3: Custom configuration
//! let mut config = PetscConfig::default();
//! config.ksp.solver_type = KspType::CG;
//! config.ksp.precond_type = PcType::ICC;
//! config.ksp.relative_tol = 1e-12;
//! let backend = PetscBackend::with_config(config)?;
//!
//! // Solve
//! let (displacement, info) = backend.solve_linear(&system_data)?;
//! println!("Solved in {} iterations", info.iterations);
//! ```

use super::petsc_config::*;
use super::petsc_wrapper::*;
use super::traits::*;
use nalgebra::{DMatrix, DVector};

/// PETSc solver backend.
///
/// Uses PETSc's KSP for linear solves and SLEPc for eigenvalue problems.
/// Configuration determines solver type, preconditioner, and tolerances.
pub struct PetscBackend {
    config: PetscConfig,
    _context: Option<PetscContext>,
}

impl PetscBackend {
    /// Create a new PETSc backend with default configuration.
    ///
    /// Default: GMRES solver with ILU preconditioner.
    pub fn new() -> Result<Self, BackendError> {
        Self::with_config(PetscConfig::default())
    }

    /// Create a PETSc backend with custom configuration.
    pub fn with_config(config: PetscConfig) -> Result<Self, BackendError> {
        // Initialize PETSc if not already done
        let context = PetscContext::init().ok();

        Ok(Self {
            config,
            _context: context,
        })
    }

    /// Get current configuration.
    pub fn config(&self) -> &PetscConfig {
        &self.config
    }

    /// Update configuration (returns new backend instance).
    pub fn with_updated_config(&self, f: impl FnOnce(&mut PetscConfig)) -> Self {
        let mut config = self.config.clone();
        f(&mut config);
        Self {
            config,
            _context: None, // Reuse existing PETSc context
        }
    }
}

impl LinearSolver for PetscBackend {
    fn solve_linear(
        &self,
        system: &LinearSystemData,
    ) -> Result<(DVector<f64>, SolveInfo), BackendError> {
        // PETSc linear solve workflow (implementation when FFI is available):
        //
        // 1. Create sparse matrix from COO triplets
        //    let mat = PetscMat::from_triplets(&system.stiffness)?;
        //
        // 2. Create vectors
        //    let b = PetscVec::from_dvector(&system.force)?;
        //    let x = PetscVec::new(system.num_dofs)?;
        //
        // 3. Configure KSP solver
        //    let ksp = configure_ksp(&mat, &self.config.ksp)?;
        //
        // 4. Solve K * x = b
        //    ksp.solve(&b, &mut x)?;
        //
        // 5. Extract result
        //    let displacement = x.to_dvector()?;
        //    let iterations = ksp.get_iteration_number()?;
        //    let residual = ksp.get_residual_norm()?;
        //
        // See implementation details in configure_ksp() below.

        #[cfg(not(feature = "petsc"))]
        {
            Err(BackendError(
                "PETSc backend not compiled. Rebuild with --features petsc or use native backend."
                    .into(),
            ))
        }

        #[cfg(feature = "petsc")]
        {
            // TODO: Implement with petsc_sys when available
            Err(BackendError(
                "PETSc FFI implementation in progress. Use native backend temporarily.".into(),
            ))
        }
    }
}

impl EigenSolver for PetscBackend {
    fn solve_eigen(
        &self,
        system: &EigenSystemData,
        num_modes: usize,
    ) -> Result<(EigenResult, SolveInfo), BackendError> {
        // SLEPc eigenvalue solve workflow (implementation when FFI is available):
        //
        // 1. Create K and M matrices
        //    let k_mat = PetscMat::from_triplets(&system.stiffness)?;
        //    let m_mat = PetscMat::from_triplets(&system.mass)?;
        //
        // 2. Configure EPS (Eigenvalue Problem Solver)
        //    let eps = configure_eps(&k_mat, &m_mat, &self.config.slepc)?;
        //
        // 3. Solve K * phi = lambda * M * phi
        //    eps.solve()?;
        //    let n_converged = eps.get_converged()?;
        //
        // 4. Extract eigenvalues and eigenvectors
        //    let mut eigenvalues = Vec::new();
        //    let mut eigenvectors_data = Vec::new();
        //
        //    for i in 0..n_converged.min(num_modes) {
        //        let (lambda_real, lambda_imag) = eps.get_eigenvalue(i)?;
        //        eigenvalues.push(lambda_real);
        //
        //        let eigenvec = eps.get_eigenvector(i)?;
        //        eigenvectors_data.extend(eigenvec.to_dvector()?.as_slice());
        //    }
        //
        //    let eigenvectors = DMatrix::from_vec(
        //        system.num_dofs,
        //        eigenvalues.len(),
        //        eigenvectors_data,
        //    );
        //
        // See implementation details in configure_eps() below.

        #[cfg(not(feature = "petsc"))]
        {
            Err(BackendError(
                "PETSc backend not compiled. Rebuild with --features petsc or use native backend."
                    .into(),
            ))
        }

        #[cfg(feature = "petsc")]
        {
            // TODO: Implement with slepc_sys when available
            Err(BackendError(
                "SLEPc FFI implementation in progress. Use native backend temporarily.".into(),
            ))
        }
    }
}

impl SolverBackend for PetscBackend {
    fn name(&self) -> &str {
        "petsc"
    }
}

impl Default for PetscBackend {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback if PETSc initialization fails
            Self {
                config: PetscConfig::default(),
                _context: None,
            }
        })
    }
}

// ============================================================================
// KSP (Krylov Subspace) Solver Configuration
// ============================================================================

/// Configure PETSc KSP solver from KspConfig.
///
/// # Implementation (pseudo-code for when FFI is available)
///
/// ```ignore
/// use petsc_sys::{KSP, KSPCreate, KSPSetOperators, KSPSetType, KSPGetPC,
///                 PCSetType, KSPSetTolerances, KSPSetFromOptions};
///
/// fn configure_ksp(mat: &PetscMat, config: &KspConfig) -> Result<KSP, BackendError> {
///     // 1. Create KSP context
///     let mut ksp: KSP = std::ptr::null_mut();
///     KSPCreate(PETSC_COMM_SELF, &mut ksp)?;
///
///     // 2. Set operators (A and preconditioning matrix, same for now)
///     KSPSetOperators(ksp, mat.handle(), mat.handle())?;
///
///     // 3. Set solver type
///     let ksp_type = config.solver_type.petsc_name();
///     KSPSetType(ksp, ksp_type.as_ptr() as *const i8)?;
///
///     // 4. Configure preconditioner
///     let mut pc: PC = std::ptr::null_mut();
///     KSPGetPC(ksp, &mut pc)?;
///     let pc_type = config.precond_type.petsc_name();
///     PCSetType(pc, pc_type.as_ptr() as *const i8)?;
///
///     // 5. Set direct solver if using PreOnly
///     if config.solver_type == KspType::PreOnly {
///         if let Some(direct) = config.direct_solver {
///             PCFactorSetMatSolverType(pc, direct.petsc_name().as_ptr() as *const i8)?;
///         }
///     }
///
///     // 6. Set ILU fill level if applicable
///     if config.precond_type == PcType::ILU && config.ilu_fill > 0 {
///         PCFactorSetLevels(pc, config.ilu_fill)?;
///     }
///
///     // 7. Set convergence tolerances
///     KSPSetTolerances(
///         ksp,
///         config.relative_tol,
///         config.absolute_tol,
///         config.divergence_tol,
///         config.max_iterations as i32,
///     )?;
///
///     // 8. Set GMRES restart if applicable
///     if config.solver_type == KspType::GMRES && config.gmres_restart > 0 {
///         KSPGMRESSetRestart(ksp, config.gmres_restart as i32)?;
///     }
///
///     // 9. Allow runtime options to override (-ksp_type, -pc_type, etc.)
///     KSPSetFromOptions(ksp)?;
///
///     Ok(ksp)
/// }
/// ```
#[allow(dead_code)]
fn configure_ksp_docs() {
    // This function exists only for documentation purposes
    // Actual implementation will be added when petsc_sys is available
}

// ============================================================================
// SLEPc Eigenvalue Solver Configuration
// ============================================================================

/// Configure SLEPc EPS (Eigenvalue Problem Solver) from SlepcConfig.
///
/// # Implementation (pseudo-code for when FFI is available)
///
/// ```ignore
/// use slepc_sys::{EPS, EPSCreate, EPSSetOperators, EPSSetProblemType,
///                 EPSSetWhichEigenpairs, EPSSetDimensions, EPSSetTolerances,
///                 EPSSetFromOptions};
///
/// fn configure_eps(
///     k_mat: &PetscMat,
///     m_mat: &PetscMat,
///     config: &SlepcConfig,
/// ) -> Result<EPS, BackendError> {
///     // 1. Create EPS context
///     let mut eps: EPS = std::ptr::null_mut();
///     EPSCreate(PETSC_COMM_SELF, &mut eps)?;
///
///     // 2. Set operators (K and M)
///     EPSSetOperators(eps, k_mat.handle(), m_mat.handle())?;
///
///     // 3. Set problem type (generalized Hermitian eigenvalue problem)
///     EPSSetProblemType(eps, EPS_GHEP)?;  // K*phi = lambda*M*phi
///
///     // 4. Set which eigenvalues to compute
///     match config.which {
///         WhichEigenvalues::SmallestMagnitude => {
///             EPSSetWhichEigenpairs(eps, EPS_SMALLEST_MAGNITUDE)?;
///         }
///         WhichEigenvalues::LargestMagnitude => {
///             EPSSetWhichEigenpairs(eps, EPS_LARGEST_MAGNITUDE)?;
///         }
///         WhichEigenvalues::Target => {
///             EPSSetWhichEigenpairs(eps, EPS_TARGET_MAGNITUDE)?;
///             if let Some(target) = config.target_value {
///                 EPSSetTarget(eps, target)?;
///             }
///         }
///         _ => { /* other options */ }
///     }
///
///     // 5. Set dimensions (nev = num eigenvalues, ncv = basis size, mpd = max projected)
///     let ncv = if config.ncv > 0 { config.ncv } else { PETSC_DECIDE };
///     let mpd = if config.mpd > 0 { config.mpd } else { PETSC_DECIDE };
///     EPSSetDimensions(
///         eps,
///         config.num_eigenvalues as i32,
///         ncv as i32,
///         mpd as i32,
///     )?;
///
///     // 6. Set convergence tolerance
///     EPSSetTolerances(eps, config.tolerance, config.max_iterations as i32)?;
///
///     // 7. Use Krylov-Schur method (default, robust for large problems)
///     EPSSetType(eps, EPSKRYLOVSCHUR)?;
///
///     // 8. Allow runtime options to override (-eps_type, -eps_nev, etc.)
///     EPSSetFromOptions(eps)?;
///
///     Ok(eps)
/// }
/// ```
#[allow(dead_code)]
fn configure_eps_docs() {
    // This function exists only for documentation purposes
    // Actual implementation will be added when slepc_sys is available
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_petsc_backend_creation() {
        // Backend creation succeeds (validation of API design)
        let result = PetscBackend::new();
        // Without petsc feature, PetscContext::init() returns Err but backend still created
        assert!(result.is_ok());
        let backend = result.unwrap();
        assert_eq!(backend.name(), "petsc");
    }

    #[test]
    fn test_petsc_backend_with_config() {
        let config = PetscConfig::direct_solver();
        let result = PetscBackend::with_config(config);
        // Backend creation succeeds even without FFI (validates API design)
        assert!(result.is_ok());
        let backend = result.unwrap();
        assert_eq!(backend.config.ksp.solver_type, KspType::PreOnly);
    }

    #[test]
    fn test_config_defaults() {
        let backend = PetscBackend::default();
        assert_eq!(backend.config.ksp.solver_type, KspType::GMRES);
        assert_eq!(backend.config.ksp.precond_type, PcType::ILU);
    }

    #[test]
    fn test_config_presets() {
        let linear_static = PetscConfig::linear_static();
        assert_eq!(linear_static.ksp.solver_type, KspType::CG);
        assert_eq!(linear_static.ksp.precond_type, PcType::ICC);

        let direct = PetscConfig::direct_solver();
        assert_eq!(direct.ksp.solver_type, KspType::PreOnly);
        assert_eq!(direct.ksp.direct_solver, Some(MatSolverType::MUMPS));

        let modal = PetscConfig::modal_analysis(10);
        assert_eq!(modal.slepc.num_eigenvalues, 10);
    }

    #[test]
    fn test_linear_solver_trait() {
        // Validate that PetscBackend implements LinearSolver trait
        let backend = PetscBackend::default();
        assert_eq!(backend.name(), "petsc");
    }

    #[test]
    fn test_solve_without_ffi() {
        let backend = PetscBackend::default();

        // Create minimal system data
        let system = LinearSystemData {
            stiffness: SparseTripletsF64 {
                nrows: 2,
                ncols: 2,
                row_indices: vec![0, 1],
                col_indices: vec![0, 1],
                values: vec![1.0, 1.0],
            },
            force: DVector::from_vec(vec![1.0, 1.0]),
            num_dofs: 2,
            constrained_dofs: vec![],
        };

        // Should fail gracefully without FFI
        let result = backend.solve_linear(&system);
        assert!(result.is_err());
    }
}
