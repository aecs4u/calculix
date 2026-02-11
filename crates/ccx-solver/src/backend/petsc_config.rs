//! PETSc solver configuration.
//!
//! This module defines configuration options for PETSc's KSP (Krylov Subspace)
//! linear solvers, preconditioners, and SLEPc eigenvalue solvers.

use serde::{Deserialize, Serialize};

/// KSP (Krylov Subspace) solver types.
///
/// PETSc provides a wide range of iterative solvers. For SPD (Symmetric
/// Positive Definite) systems like standard linear static FEA, CG is optimal.
/// For general systems, GMRES or BiCGSTAB are good choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KspType {
    /// Conjugate Gradient (for SPD systems)
    CG,
    /// Generalized Minimal Residual (general systems)
    GMRES,
    /// Biconjugate Gradient Stabilized (general systems)
    BiCGSTAB,
    /// Transpose-Free Quasi-Minimal Residual
    TFQMR,
    /// Richardson iteration
    Richardson,
    /// Chebyshev iteration
    Chebyshev,
    /// Preonly (preconditioner only - for direct solvers)
    PreOnly,
}

impl Default for KspType {
    fn default() -> Self {
        KspType::GMRES
    }
}

impl KspType {
    /// PETSc string identifier for this KSP type.
    pub fn petsc_name(&self) -> &'static str {
        match self {
            KspType::CG => "cg",
            KspType::GMRES => "gmres",
            KspType::BiCGSTAB => "bcgs",
            KspType::TFQMR => "tfqmr",
            KspType::Richardson => "richardson",
            KspType::Chebyshev => "chebyshev",
            KspType::PreOnly => "preonly",
        }
    }
}

/// PC (Preconditioner) types.
///
/// Preconditioning is critical for iterative solver convergence.
/// For SPD systems, ICC (incomplete Cholesky) is effective.
/// For general systems, ILU (incomplete LU) or Jacobi are common.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PcType {
    /// No preconditioner
    None,
    /// Jacobi (diagonal scaling)
    Jacobi,
    /// Incomplete LU factorization
    ILU,
    /// Incomplete Cholesky factorization (for SPD)
    ICC,
    /// Additive Schwarz Method (domain decomposition)
    ASM,
    /// Block Jacobi
    BJ,
    /// Successive Over-Relaxation
    SOR,
    /// Algebraic Multigrid (via HYPRE BoomerAMG)
    HYPRE,
    /// LU factorization (direct solve via preconditioner)
    LU,
    /// Cholesky factorization (direct solve via preconditioner)
    Cholesky,
}

impl Default for PcType {
    fn default() -> Self {
        PcType::ILU
    }
}

impl PcType {
    /// PETSc string identifier for this PC type.
    pub fn petsc_name(&self) -> &'static str {
        match self {
            PcType::None => "none",
            PcType::Jacobi => "jacobi",
            PcType::ILU => "ilu",
            PcType::ICC => "icc",
            PcType::ASM => "asm",
            PcType::BJ => "bjacobi",
            PcType::SOR => "sor",
            PcType::HYPRE => "hypre",
            PcType::LU => "lu",
            PcType::Cholesky => "cholesky",
        }
    }
}

/// Direct solver libraries available through PETSc.
///
/// When using `KspType::PreOnly` with `PcType::LU` or `PcType::Cholesky`,
/// you can specify which direct solver library to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatSolverType {
    /// MUMPS (MUltifrontal Massively Parallel Sparse direct Solver)
    MUMPS,
    /// SuperLU (sequential)
    SuperLU,
    /// SuperLU_DIST (distributed memory)
    SuperLUDist,
    /// Intel MKL PARDISO
    PARDISO,
    /// PaStiX (Parallel Sparse matrix package)
    PaStiX,
    /// UMFPACK (sequential)
    UMFPACK,
    /// Default PETSc solver
    Default,
}

impl Default for MatSolverType {
    fn default() -> Self {
        MatSolverType::Default
    }
}

impl MatSolverType {
    /// PETSc string identifier for this direct solver.
    pub fn petsc_name(&self) -> &'static str {
        match self {
            MatSolverType::MUMPS => "mumps",
            MatSolverType::SuperLU => "superlu",
            MatSolverType::SuperLUDist => "superlu_dist",
            MatSolverType::PARDISO => "mkl_pardiso",
            MatSolverType::PaStiX => "pastix",
            MatSolverType::UMFPACK => "umfpack",
            MatSolverType::Default => "petsc",
        }
    }
}

/// Configuration for PETSc linear solver (KSP).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KspConfig {
    /// Iterative solver type
    pub solver_type: KspType,
    /// Preconditioner type
    pub precond_type: PcType,
    /// Direct solver (used when KspType::PreOnly)
    pub direct_solver: Option<MatSolverType>,
    /// Relative tolerance for convergence
    pub relative_tol: f64,
    /// Absolute tolerance for convergence
    pub absolute_tol: f64,
    /// Divergence tolerance (solver fails if residual exceeds this)
    pub divergence_tol: f64,
    /// Maximum iterations
    pub max_iterations: usize,
    /// GMRES restart parameter (0 = use PETSc default of 30)
    pub gmres_restart: usize,
    /// ILU fill level (0 = no extra fill, higher = more accuracy but slower)
    pub ilu_fill: i32,
}

impl Default for KspConfig {
    fn default() -> Self {
        Self {
            solver_type: KspType::GMRES,
            precond_type: PcType::ILU,
            direct_solver: None,
            relative_tol: 1e-10,
            absolute_tol: 1e-50,
            divergence_tol: 1e5,
            max_iterations: 1000,
            gmres_restart: 30,
            ilu_fill: 0,
        }
    }
}

impl KspConfig {
    /// Configuration for CG with ICC preconditioner (SPD systems).
    pub fn cg_icc() -> Self {
        Self {
            solver_type: KspType::CG,
            precond_type: PcType::ICC,
            ..Default::default()
        }
    }

    /// Configuration for direct solver using MUMPS.
    pub fn direct_mumps() -> Self {
        Self {
            solver_type: KspType::PreOnly,
            precond_type: PcType::LU,
            direct_solver: Some(MatSolverType::MUMPS),
            max_iterations: 1,
            ..Default::default()
        }
    }

    /// Configuration for direct solver using SuperLU.
    pub fn direct_superlu() -> Self {
        Self {
            solver_type: KspType::PreOnly,
            precond_type: PcType::LU,
            direct_solver: Some(MatSolverType::SuperLU),
            max_iterations: 1,
            ..Default::default()
        }
    }

    /// Configuration for GMRES with ILU(k) preconditioner.
    pub fn gmres_ilu(fill_level: i32) -> Self {
        Self {
            solver_type: KspType::GMRES,
            precond_type: PcType::ILU,
            ilu_fill: fill_level,
            ..Default::default()
        }
    }
}

/// Which eigenvalues to compute in SLEPc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhichEigenvalues {
    /// Smallest magnitude (lowest frequencies for modal analysis)
    SmallestMagnitude,
    /// Largest magnitude (highest frequencies)
    LargestMagnitude,
    /// Smallest real part
    SmallestReal,
    /// Largest real part
    LargestReal,
    /// Target value (requires `target_value` in SlepcConfig)
    Target,
}

impl Default for WhichEigenvalues {
    fn default() -> Self {
        WhichEigenvalues::SmallestMagnitude
    }
}

impl WhichEigenvalues {
    /// SLEPc string identifier.
    pub fn slepc_name(&self) -> &'static str {
        match self {
            WhichEigenvalues::SmallestMagnitude => "smallest_magnitude",
            WhichEigenvalues::LargestMagnitude => "largest_magnitude",
            WhichEigenvalues::SmallestReal => "smallest_real",
            WhichEigenvalues::LargestReal => "largest_real",
            WhichEigenvalues::Target => "target_magnitude",
        }
    }
}

/// Configuration for SLEPc eigenvalue solver (EPS).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlepcConfig {
    /// Number of eigenvalues to compute
    pub num_eigenvalues: usize,
    /// Which eigenvalues to compute
    pub which: WhichEigenvalues,
    /// Target eigenvalue (used when which = Target)
    pub target_value: Option<f64>,
    /// Convergence tolerance
    pub tolerance: f64,
    /// Maximum iterations
    pub max_iterations: usize,
    /// Number of basis vectors (0 = use SLEPc default)
    pub ncv: usize,
    /// Maximum dimension of projected problem (0 = use SLEPc default)
    pub mpd: usize,
}

impl Default for SlepcConfig {
    fn default() -> Self {
        Self {
            num_eigenvalues: 6,
            which: WhichEigenvalues::SmallestMagnitude,
            target_value: None,
            tolerance: 1e-8,
            max_iterations: 1000,
            ncv: 0,  // SLEPc will choose
            mpd: 0,  // SLEPc will choose
        }
    }
}

impl SlepcConfig {
    /// Configuration for modal analysis (first N natural frequencies).
    pub fn modal_analysis(num_modes: usize) -> Self {
        Self {
            num_eigenvalues: num_modes,
            which: WhichEigenvalues::SmallestMagnitude,
            ..Default::default()
        }
    }

    /// Configuration for computing eigenvalues near a target frequency.
    pub fn target_frequency(target_omega_squared: f64, num_modes: usize) -> Self {
        Self {
            num_eigenvalues: num_modes,
            which: WhichEigenvalues::Target,
            target_value: Some(target_omega_squared),
            ..Default::default()
        }
    }
}

/// Complete PETSc backend configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetscConfig {
    /// Linear solver configuration
    pub ksp: KspConfig,
    /// Eigenvalue solver configuration
    pub slepc: SlepcConfig,
    /// Enable verbose output from PETSc/SLEPc
    pub verbose: bool,
}

impl Default for PetscConfig {
    fn default() -> Self {
        Self {
            ksp: KspConfig::default(),
            slepc: SlepcConfig::default(),
            verbose: false,
        }
    }
}

impl PetscConfig {
    /// Create a configuration optimized for linear static analysis (SPD systems).
    pub fn linear_static() -> Self {
        Self {
            ksp: KspConfig::cg_icc(),
            ..Default::default()
        }
    }

    /// Create a configuration using direct solver (MUMPS).
    pub fn direct_solver() -> Self {
        Self {
            ksp: KspConfig::direct_mumps(),
            ..Default::default()
        }
    }

    /// Create a configuration for modal analysis with N modes.
    pub fn modal_analysis(num_modes: usize) -> Self {
        Self {
            ksp: KspConfig::gmres_ilu(1),
            slepc: SlepcConfig::modal_analysis(num_modes),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ksp_type_names() {
        assert_eq!(KspType::CG.petsc_name(), "cg");
        assert_eq!(KspType::GMRES.petsc_name(), "gmres");
        assert_eq!(KspType::BiCGSTAB.petsc_name(), "bcgs");
    }

    #[test]
    fn test_pc_type_names() {
        assert_eq!(PcType::ILU.petsc_name(), "ilu");
        assert_eq!(PcType::Jacobi.petsc_name(), "jacobi");
        assert_eq!(PcType::HYPRE.petsc_name(), "hypre");
    }

    #[test]
    fn test_mat_solver_names() {
        assert_eq!(MatSolverType::MUMPS.petsc_name(), "mumps");
        assert_eq!(MatSolverType::SuperLU.petsc_name(), "superlu");
        assert_eq!(MatSolverType::PARDISO.petsc_name(), "mkl_pardiso");
    }

    #[test]
    fn test_default_config() {
        let config = PetscConfig::default();
        assert_eq!(config.ksp.solver_type, KspType::GMRES);
        assert_eq!(config.ksp.precond_type, PcType::ILU);
        assert_eq!(config.ksp.relative_tol, 1e-10);
        assert_eq!(config.slepc.num_eigenvalues, 6);
    }

    #[test]
    fn test_preset_configs() {
        let linear = PetscConfig::linear_static();
        assert_eq!(linear.ksp.solver_type, KspType::CG);
        assert_eq!(linear.ksp.precond_type, PcType::ICC);

        let direct = PetscConfig::direct_solver();
        assert_eq!(direct.ksp.solver_type, KspType::PreOnly);
        assert_eq!(direct.ksp.direct_solver, Some(MatSolverType::MUMPS));

        let modal = PetscConfig::modal_analysis(10);
        assert_eq!(modal.slepc.num_eigenvalues, 10);
        assert_eq!(modal.slepc.which, WhichEigenvalues::SmallestMagnitude);
    }

    #[test]
    fn test_custom_ksp_configs() {
        let ilu2 = KspConfig::gmres_ilu(2);
        assert_eq!(ilu2.ilu_fill, 2);
        assert_eq!(ilu2.solver_type, KspType::GMRES);
    }

    #[test]
    fn test_target_frequency_config() {
        let target_omega_sq = 1000.0; // Target: 5 Hz → ω² ≈ 986
        let config = SlepcConfig::target_frequency(target_omega_sq, 5);
        assert_eq!(config.which, WhichEigenvalues::Target);
        assert_eq!(config.target_value, Some(target_omega_sq));
        assert_eq!(config.num_eigenvalues, 5);
    }
}
