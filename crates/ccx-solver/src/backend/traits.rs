//! Backend trait definitions for numerical solvers.
//!
//! These traits abstract over the concrete numerical library used for
//! global system operations (linear solve, eigenvalue solve). Element-level
//! computations remain in nalgebra (small, dense matrices).

use nalgebra::{DMatrix, DVector};

/// Error type for backend operations.
#[derive(Debug, Clone)]
pub struct BackendError(pub String);

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for BackendError {}

impl From<String> for BackendError {
    fn from(s: String) -> Self {
        BackendError(s)
    }
}

impl From<&str> for BackendError {
    fn from(s: &str) -> Self {
        BackendError(s.to_string())
    }
}

/// Sparse matrix in COO (coordinate/triplet) format.
///
/// This is the backend-agnostic interchange format between the assembly
/// layer and any solver backend. Both nalgebra-sparse and PETSc can
/// efficiently consume COO triplets.
pub struct SparseTripletsF64 {
    pub nrows: usize,
    pub ncols: usize,
    pub row_indices: Vec<usize>,
    pub col_indices: Vec<usize>,
    pub values: Vec<f64>,
}

impl SparseTripletsF64 {
    /// Number of non-zero entries.
    pub fn nnz(&self) -> usize {
        self.values.len()
    }
}

/// A linear system ready for solving: K * u = F.
///
/// Produced by the assembly layer, consumed by any `LinearSolver` backend.
/// Boundary conditions should already be applied to K and F before
/// constructing this struct.
pub struct LinearSystemData {
    /// Stiffness matrix in COO triplet format (BCs already applied)
    pub stiffness: SparseTripletsF64,
    /// Force vector (dense, BCs already applied)
    pub force: DVector<f64>,
    /// Total number of degrees of freedom
    pub num_dofs: usize,
    /// Indices of constrained DOFs (for diagnostics)
    pub constrained_dofs: Vec<usize>,
}

/// A generalized eigenvalue system: K * phi = lambda * M * phi.
pub struct EigenSystemData {
    /// Stiffness matrix in COO triplet format
    pub stiffness: SparseTripletsF64,
    /// Mass matrix in COO triplet format
    pub mass: SparseTripletsF64,
    /// Total number of degrees of freedom
    pub num_dofs: usize,
    /// Indices of free (unconstrained) DOFs
    pub free_dofs: Vec<usize>,
}

/// Results from an eigenvalue solve.
pub struct EigenResult {
    /// Eigenvalues (lambda = omega^2), sorted ascending
    pub eigenvalues: Vec<f64>,
    /// Eigenvectors as columns in full DOF space (num_dofs x num_modes)
    pub eigenvectors: DMatrix<f64>,
}

/// Solver convergence and diagnostic info.
pub struct SolveInfo {
    /// Number of iterations (1 for direct solvers)
    pub iterations: usize,
    /// Final residual norm (if available)
    pub residual_norm: Option<f64>,
    /// Human-readable solver name (e.g., "nalgebra-LU", "PETSc-MUMPS")
    pub solver_name: String,
}

/// Trait for a linear solver backend.
///
/// Implementations solve K * u = F given the assembled system data.
pub trait LinearSolver: Send + Sync {
    /// Solve K * u = F and return the displacement vector.
    fn solve_linear(
        &self,
        system: &LinearSystemData,
    ) -> Result<(DVector<f64>, SolveInfo), BackendError>;
}

/// Trait for an eigenvalue solver backend.
///
/// Implementations solve the generalized eigenvalue problem
/// K * phi = lambda * M * phi, returning the first `num_modes`
/// positive eigenvalues and eigenvectors.
pub trait EigenSolver: Send + Sync {
    /// Solve the generalized eigenvalue problem.
    fn solve_eigen(
        &self,
        system: &EigenSystemData,
        num_modes: usize,
    ) -> Result<(EigenResult, SolveInfo), BackendError>;
}

/// Combined backend providing both linear and eigenvalue solvers.
pub trait SolverBackend: LinearSolver + EigenSolver {
    /// Human-readable name of this backend.
    fn name(&self) -> &str;
}
