//! Native backend using nalgebra and nalgebra-lapack.
//!
//! This is the default backend when no external solver library is available.
//! It supports:
//! - Dense LU decomposition for linear systems (small-to-medium problems)
//! - Cholesky-transformed SymmetricEigen for generalized eigenvalue problems

use super::traits::*;
use nalgebra::{DMatrix, DVector};
use nalgebra_lapack::SymmetricEigen;

/// Native solver backend using nalgebra for all numerical operations.
///
/// Suitable for small-to-medium problems (up to ~10,000 DOFs).
/// For larger problems, consider the PETSc backend.
pub struct NativeBackend;

impl LinearSolver for NativeBackend {
    fn solve_linear(
        &self,
        system: &LinearSystemData,
    ) -> Result<(DVector<f64>, SolveInfo), BackendError> {
        let n = system.num_dofs;

        // Reconstruct dense matrix from COO triplets
        let mut k = DMatrix::zeros(n, n);
        for i in 0..system.stiffness.nnz() {
            let r = system.stiffness.row_indices[i];
            let c = system.stiffness.col_indices[i];
            k[(r, c)] += system.stiffness.values[i];
        }

        // LU decomposition and solve
        let u = k
            .lu()
            .solve(&system.force)
            .ok_or(BackendError("Singular matrix in LU decomposition".into()))?;

        Ok((
            u,
            SolveInfo {
                iterations: 1,
                residual_norm: None,
                solver_name: "nalgebra-LU".to_string(),
            },
        ))
    }
}

impl EigenSolver for NativeBackend {
    fn solve_eigen(
        &self,
        system: &EigenSystemData,
        num_modes: usize,
    ) -> Result<(EigenResult, SolveInfo), BackendError> {
        let n_full = system.num_dofs;
        let free = &system.free_dofs;
        let n = free.len();

        if n == 0 {
            return Err("No free DOFs for eigenvalue problem".into());
        }

        // Reconstruct dense K and M from COO triplets
        let mut k_full = DMatrix::zeros(n_full, n_full);
        for i in 0..system.stiffness.nnz() {
            let r = system.stiffness.row_indices[i];
            let c = system.stiffness.col_indices[i];
            k_full[(r, c)] += system.stiffness.values[i];
        }

        let mut m_full = DMatrix::zeros(n_full, n_full);
        for i in 0..system.mass.nnz() {
            let r = system.mass.row_indices[i];
            let c = system.mass.col_indices[i];
            m_full[(r, c)] += system.mass.values[i];
        }

        // Reduce to free DOFs
        let mut k_red = DMatrix::zeros(n, n);
        let mut m_red = DMatrix::zeros(n, n);
        for (i_red, &i_full) in free.iter().enumerate() {
            for (j_red, &j_full) in free.iter().enumerate() {
                k_red[(i_red, j_red)] = k_full[(i_full, j_full)];
                m_red[(i_red, j_red)] = m_full[(i_full, j_full)];
            }
        }

        // Cholesky decomposition: M = L * L^T
        use nalgebra::linalg::Cholesky;
        let chol_m = Cholesky::new(m_red.clone())
            .ok_or(BackendError("Mass matrix not positive definite".into()))?;

        let l = chol_m.l();
        let l_inv = l
            .clone()
            .try_inverse()
            .ok_or(BackendError("Failed to invert Cholesky factor L".into()))?;

        // K* = L^-1 * K * L^-T
        let k_star = &l_inv * &k_red * &l_inv.transpose();

        // Solve standard symmetric eigenvalue problem: K* * psi = lambda * psi
        let eigen = SymmetricEigen::new(k_star.into());
        let eigenvalues_vec = eigen.eigenvalues.as_slice();
        let eigenvectors_psi = &eigen.eigenvectors;

        // Transform eigenvectors back: phi = L^-T * psi
        let l_inv_t = l_inv.transpose();

        // Collect positive eigenvalues and eigenvectors
        let mut pairs: Vec<(f64, DVector<f64>)> = Vec::new();
        for i in 0..n {
            let lambda = eigenvalues_vec[i];
            if lambda > 1e-10 {
                let psi: DVector<f64> = eigenvectors_psi.column(i).into_owned();
                let phi_red = &l_inv_t * psi;
                pairs.push((lambda, phi_red));
            }
        }

        // Sort by eigenvalue ascending
        pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let num_available = pairs.len().min(num_modes);
        if num_available == 0 {
            return Err("No positive eigenvalues found".into());
        }

        let eigenvalues: Vec<f64> = pairs[..num_available].iter().map(|(l, _)| *l).collect();

        // Expand eigenvectors from reduced to full DOF space
        let mut eigenvectors = DMatrix::zeros(n_full, num_available);
        for (mode, (_, phi_red)) in pairs[..num_available].iter().enumerate() {
            for (i_red, &i_full) in free.iter().enumerate() {
                eigenvectors[(i_full, mode)] = phi_red[i_red];
            }
        }

        Ok((
            EigenResult {
                eigenvalues,
                eigenvectors,
            },
            SolveInfo {
                iterations: 1,
                residual_norm: None,
                solver_name: "nalgebra-Cholesky+SymmetricEigen".to_string(),
            },
        ))
    }
}

impl SolverBackend for NativeBackend {
    fn name(&self) -> &str {
        "native-nalgebra"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_linear_solve_trivial() {
        // Solve: [2 0; 0 3] * [x; y] = [4; 9]
        // Solution: x=2, y=3
        let backend = NativeBackend;
        let system = LinearSystemData {
            stiffness: SparseTripletsF64 {
                nrows: 2,
                ncols: 2,
                row_indices: vec![0, 1],
                col_indices: vec![0, 1],
                values: vec![2.0, 3.0],
            },
            force: DVector::from_vec(vec![4.0, 9.0]),
            num_dofs: 2,
            constrained_dofs: vec![],
        };

        let (u, info) = backend.solve_linear(&system).unwrap();
        assert!((u[0] - 2.0).abs() < 1e-12);
        assert!((u[1] - 3.0).abs() < 1e-12);
        assert_eq!(info.solver_name, "nalgebra-LU");
    }

    #[test]
    fn native_linear_solve_3x3() {
        // Solve a symmetric positive definite 3×3 system
        // K = [4 -1 0; -1 4 -1; 0 -1 4], F = [1; 2; 1]
        let backend = NativeBackend;
        let system = LinearSystemData {
            stiffness: SparseTripletsF64 {
                nrows: 3,
                ncols: 3,
                row_indices: vec![0, 0, 1, 1, 1, 2, 2],
                col_indices: vec![0, 1, 0, 1, 2, 1, 2],
                values: vec![4.0, -1.0, -1.0, 4.0, -1.0, -1.0, 4.0],
            },
            force: DVector::from_vec(vec![1.0, 2.0, 1.0]),
            num_dofs: 3,
            constrained_dofs: vec![],
        };

        let (u, _) = backend.solve_linear(&system).unwrap();

        // Verify K*u = F
        let k = DMatrix::from_row_slice(3, 3, &[4.0, -1.0, 0.0, -1.0, 4.0, -1.0, 0.0, -1.0, 4.0]);
        let f_check = &k * &u;
        for i in 0..3 {
            assert!(
                (f_check[i] - system.force[i]).abs() < 1e-10,
                "Residual too large at DOF {}",
                i
            );
        }
    }
}
