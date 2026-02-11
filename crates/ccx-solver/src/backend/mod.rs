//! Numerical backend abstraction layer.
//!
//! This module provides trait-based interfaces for linear and eigenvalue
//! solvers, allowing the assembly layer to be backend-agnostic. The actual
//! numerical work is dispatched to a concrete backend at runtime.
//!
//! # Backends
//!
//! - **Native** (default): Uses nalgebra + nalgebra-lapack. No external
//!   dependencies. Suitable for small-to-medium problems.
//! - **PETSc** (optional, `--features petsc`): Uses PETSc for scalable
//!   Krylov solvers, preconditioners, and access to MUMPS/SuperLU/PaStiX.
//!
//! # Architecture
//!
//! ```text
//! Element Library (nalgebra DMatrix — small, dense)
//!         │
//!         ▼
//! Assembly (produces COO triplets + force vector)
//!         │
//!         ▼
//! Backend Trait Layer (LinearSolver, EigenSolver)
//!    ┌────┴────┐
//!    ▼         ▼
//! Native    PETSc
//! Backend   Backend
//! ```

pub mod native;
pub mod petsc;
pub mod traits;

pub use native::NativeBackend;
pub use petsc::PetscBackend;
pub use traits::*;

/// Returns the default solver backend based on enabled features.
///
/// With `--features petsc`: returns `PetscBackend`.
/// Without: returns `NativeBackend`.
pub fn default_backend() -> Box<dyn SolverBackend> {
    Box::new(NativeBackend)
}
