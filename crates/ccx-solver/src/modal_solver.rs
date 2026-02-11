//! Modal analysis solver for computing natural frequencies and mode shapes.
//!
//! This module implements eigenvalue analysis for undamped free vibration:
//! (K - λM)φ = 0
//!
//! where:
//! - K = global stiffness matrix
//! - M = global mass matrix
//! - λ = ω² (squared angular frequency)
//! - φ = mode shape (eigenvector)
//!
//! # Workflow
//! 1. Assemble global K and M matrices
//! 2. Extract free DOFs (exclude constrained DOFs from boundary conditions)
//! 3. Reduce matrices to free DOFs only: K_red, M_red
//! 4. Solve generalized eigenvalue problem: K_red * φ = λ * M_red * φ
//! 5. Convert eigenvalues to frequencies: f = √λ / (2π)
//! 6. Expand mode shapes back to full DOF space
//!
//! # Example
//! ```no_run
//! use ccx_solver::{Mesh, MaterialLibrary, BoundaryConditions, ModalSolver};
//!
//! # fn example(mesh: Mesh, materials: MaterialLibrary, bcs: BoundaryConditions) {
//! let solver = ModalSolver::new(&mesh, &materials, &bcs, 0.01);
//! let results = solver.solve(10).expect("Modal analysis failed");
//!
//! println!("Natural frequencies (Hz):");
//! for (i, freq) in results.frequencies_hz.iter().enumerate() {
//!     println!("  Mode {}: {:.2} Hz", i + 1, freq);
//! }
//! # }
//! ```

use crate::assembly::GlobalSystem;
use crate::boundary_conditions::BoundaryConditions;
use crate::materials::MaterialLibrary;
use crate::mesh::Mesh;
use nalgebra::{DMatrix, DVector};
use nalgebra_lapack::SymmetricEigen;

/// Results from modal analysis
#[derive(Debug, Clone)]
pub struct ModalResults {
    /// Natural frequencies in Hz
    pub frequencies_hz: Vec<f64>,
    /// Eigenvalues (λ = ω² = (2πf)²)
    pub eigenvalues: Vec<f64>,
    /// Mode shapes (eigenvectors) - each column is a mode shape
    /// Size: (num_dofs × num_modes)
    pub mode_shapes: DMatrix<f64>,
    /// Number of modes computed
    pub num_modes: usize,
}

impl ModalResults {
    /// Get the i-th mode shape as a vector
    pub fn mode_shape(&self, mode_index: usize) -> Option<DVector<f64>> {
        if mode_index >= self.num_modes {
            return None;
        }
        Some(self.mode_shapes.column(mode_index).into())
    }

    /// Get angular frequency (rad/s) for a given mode
    pub fn angular_frequency(&self, mode_index: usize) -> Option<f64> {
        self.eigenvalues
            .get(mode_index)
            .map(|&lambda| lambda.sqrt())
    }
}

/// Modal analysis solver
pub struct ModalSolver<'a> {
    mesh: &'a Mesh,
    materials: &'a MaterialLibrary,
    bcs: &'a BoundaryConditions,
    default_area: f64,
}

impl<'a> ModalSolver<'a> {
    /// Create a new modal solver
    ///
    /// # Arguments
    /// * `mesh` - Finite element mesh
    /// * `materials` - Material library (must include density)
    /// * `bcs` - Boundary conditions (displacement constraints)
    /// * `default_area` - Default cross-sectional area or thickness
    pub fn new(
        mesh: &'a Mesh,
        materials: &'a MaterialLibrary,
        bcs: &'a BoundaryConditions,
        default_area: f64,
    ) -> Self {
        Self {
            mesh,
            materials,
            bcs,
            default_area,
        }
    }

    /// Solve the modal analysis problem
    ///
    /// # Arguments
    /// * `num_modes` - Number of modes to compute
    ///
    /// # Returns
    /// Modal analysis results containing frequencies and mode shapes
    ///
    /// # Errors
    /// Returns error if:
    /// - Mass matrix assembly fails (e.g., missing density)
    /// - Eigenvalue solver fails
    /// - No positive eigenvalues found
    pub fn solve(&self, num_modes: usize) -> Result<ModalResults, String> {
        // Step 1: Assemble global K and M matrices
        let system = self.assemble_system()?;

        // Step 2: Determine free DOFs (exclude constrained)
        let free_dofs = self.extract_free_dofs(&system)?;

        if free_dofs.is_empty() {
            return Err("No free DOFs available for modal analysis (all DOFs constrained)".to_string());
        }

        // Step 3: Reduce matrices to free DOFs
        let k_red = self.reduce_matrix(&system.stiffness, &free_dofs);
        let m_red = self.reduce_matrix(
            system.mass.as_ref().ok_or("Mass matrix not assembled")?,
            &free_dofs,
        );

        // Step 4: Solve generalized eigenvalue problem
        let (eigenvalues, eigenvectors) = self.solve_eigenvalue_problem(&k_red, &m_red, num_modes)?;

        // Step 5: Convert eigenvalues to frequencies
        let frequencies_hz: Vec<f64> = eigenvalues
            .iter()
            .map(|&lambda| {
                if lambda > 0.0 {
                    lambda.sqrt() / (2.0 * std::f64::consts::PI)
                } else {
                    0.0
                }
            })
            .collect();

        // Step 6: Expand mode shapes to full DOF space
        let mode_shapes = self.expand_mode_shapes(&eigenvectors, &free_dofs, system.num_dofs);

        Ok(ModalResults {
            frequencies_hz,
            eigenvalues: eigenvalues.clone(),
            mode_shapes,
            num_modes: eigenvalues.len(),
        })
    }

    /// Assemble global stiffness and mass matrices
    fn assemble_system(&self) -> Result<GlobalSystem, String> {
        // Assemble stiffness and force (standard assembly)
        let mut system =
            GlobalSystem::assemble(self.mesh, self.materials, self.bcs, self.default_area)?;

        // Determine max DOFs per node
        let max_dofs_per_node = self
            .mesh
            .elements
            .values()
            .map(|e| e.element_type.dofs_per_node())
            .max()
            .unwrap_or(3);

        // Assemble mass matrix (required for modal analysis)
        system.assemble_mass(self.mesh, self.materials, self.default_area, max_dofs_per_node)?;

        Ok(system)
    }

    /// Extract free DOFs (non-constrained DOFs)
    fn extract_free_dofs(&self, system: &GlobalSystem) -> Result<Vec<usize>, String> {
        let all_dofs: Vec<usize> = (0..system.num_dofs).collect();
        let constrained = &system.constrained_dofs;

        // Free DOFs are all DOFs minus constrained DOFs
        let free_dofs: Vec<usize> = all_dofs
            .into_iter()
            .filter(|dof| !constrained.contains(dof))
            .collect();

        Ok(free_dofs)
    }

    /// Reduce a matrix to include only free DOFs
    ///
    /// Extracts submatrix K_red[i,j] = K[free_dofs[i], free_dofs[j]]
    fn reduce_matrix(&self, matrix: &DMatrix<f64>, free_dofs: &[usize]) -> DMatrix<f64> {
        let n = free_dofs.len();
        let mut reduced = DMatrix::zeros(n, n);

        for (i, &dof_i) in free_dofs.iter().enumerate() {
            for (j, &dof_j) in free_dofs.iter().enumerate() {
                reduced[(i, j)] = matrix[(dof_i, dof_j)];
            }
        }

        reduced
    }

    /// Solve the generalized eigenvalue problem K*φ = λ*M*φ
    ///
    /// Uses Cholesky decomposition to transform to standard eigenvalue problem:
    /// 1. M = L*L^T (Cholesky decomposition)
    /// 2. K* = L^-1 * K * L^-T (transformed stiffness)
    /// 3. Solve K*ψ = λψ (standard eigenvalue problem)
    /// 4. φ = L^-T * ψ (transform back)
    fn solve_eigenvalue_problem(
        &self,
        k_red: &DMatrix<f64>,
        m_red: &DMatrix<f64>,
        num_modes: usize,
    ) -> Result<(Vec<f64>, DMatrix<f64>), String> {
        // Check matrix dimensions
        if k_red.nrows() != k_red.ncols() || m_red.nrows() != m_red.ncols() {
            return Err("Matrices must be square".to_string());
        }
        if k_red.nrows() != m_red.nrows() {
            return Err("K and M must have same dimensions".to_string());
        }

        let n = k_red.nrows();
        if n == 0 {
            return Err("Cannot solve eigenvalue problem for 0×0 matrices".to_string());
        }

        // For generalized eigenvalue problem K*φ = λ*M*φ, we transform it to
        // a standard eigenvalue problem using Cholesky decomposition of M.
        //
        // However, nalgebra-lapack's SymmetricEigen currently solves K*φ = λ*φ
        // To solve K*φ = λ*M*φ, we would need:
        // 1. Cholesky: M = L*L^T
        // 2. Transform: L^-1 * K * L^-T * ψ = λ * ψ
        // 3. Back transform: φ = L^-T * ψ
        //
        // For now, we use a simplified approach: solve M^-1*K*φ = λ*φ
        // This requires M to be invertible, which should be true for proper FE models.

        // Check if M is positive definite (required for inversion)
        let m_min_diag = (0..n).map(|i| m_red[(i, i)]).fold(f64::INFINITY, f64::min);
        if m_min_diag <= 1e-14 {
            return Err(format!(
                "Mass matrix is singular or near-singular (min diagonal = {:.2e})",
                m_min_diag
            ));
        }

        // Use Cholesky decomposition to transform generalized eigenvalue problem
        // K*φ = λ*M*φ into standard eigenvalue problem
        //
        // 1. Decompose M = L*L^T (Cholesky)
        // 2. Solve L * y = K*φ for y, giving y = L^-1 * K * φ
        // 3. Substitute into K*φ = λ*M*φ to get (L^T)^-1 * y = λ * φ
        // 4. This transforms to: (L^-1 * K * L^-T) * ψ = λ * ψ where φ = L^-T * ψ

        use nalgebra::linalg::Cholesky;

        // Cholesky decomposition: M = L*L^T
        let chol_m = Cholesky::new(m_red.clone())
            .ok_or("Mass matrix is not positive definite (Cholesky decomposition failed)")?;

        let l = chol_m.l();

        // Compute L^-1 (we need this for the transformation)
        let l_inv = l.clone().try_inverse()
            .ok_or("Failed to invert L")?;

        // Compute K_star = L^-1 * K * (L^-1)^T
        let l_inv_k = &l_inv * k_red;
        let k_star = &l_inv_k * &l_inv.transpose();

        // Solve standard symmetric eigenvalue problem: K_star * ψ = λ * ψ
        let eigen = SymmetricEigen::new(k_star.into());
        let eigenvalues_vec = eigen.eigenvalues.as_slice();
        let eigenvectors_psi = &eigen.eigenvectors;

        // Transform eigenvectors back: φ = L^-T * ψ = (L^-1)^T * ψ
        let l_inv_t = l_inv.transpose();

        // Extract positive eigenvalues and corresponding eigenvectors
        let mut lambda_phi_pairs: Vec<(f64, DVector<f64>)> = Vec::new();
        for i in 0..n {
            let lambda = eigenvalues_vec[i];
            if lambda > 1e-10 {
                // Only positive eigenvalues (non-rigid body modes)
                let psi: DVector<f64> = eigenvectors_psi.column(i).into_owned();
                let phi = &l_inv_t * psi; // Transform back to original space
                lambda_phi_pairs.push((lambda, phi));
            }
        }

        // Sort by eigenvalue (ascending frequency)
        lambda_phi_pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Take first num_modes
        let num_available = lambda_phi_pairs.len().min(num_modes);
        if num_available == 0 {
            return Err("No positive eigenvalues found (only rigid body modes?)".to_string());
        }

        let eigenvalues: Vec<f64> = lambda_phi_pairs[..num_available]
            .iter()
            .map(|(lambda, _)| *lambda)
            .collect();

        let mut eigenvectors_matrix = DMatrix::zeros(n, num_available);
        for (i, (_, phi)) in lambda_phi_pairs[..num_available].iter().enumerate() {
            eigenvectors_matrix.set_column(i, phi);
        }

        Ok((eigenvalues, eigenvectors_matrix))
    }

    /// Expand mode shapes from reduced DOF space to full DOF space
    ///
    /// Inserts zeros for constrained DOFs
    fn expand_mode_shapes(
        &self,
        reduced_shapes: &DMatrix<f64>,
        free_dofs: &[usize],
        num_dofs: usize,
    ) -> DMatrix<f64> {
        let num_modes = reduced_shapes.ncols();
        let mut full_shapes = DMatrix::zeros(num_dofs, num_modes);

        for mode_idx in 0..num_modes {
            for (reduced_idx, &dof_idx) in free_dofs.iter().enumerate() {
                full_shapes[(dof_idx, mode_idx)] = reduced_shapes[(reduced_idx, mode_idx)];
            }
        }

        full_shapes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary_conditions::DisplacementBC;
    use crate::materials::{Material, MaterialModel};
    use crate::mesh::{Element, ElementType, Node};

    fn make_simple_cantilever_beam() -> (Mesh, MaterialLibrary, BoundaryConditions) {
        let mut mesh = Mesh::new();

        // Create a simple 2-node beam (cantilever)
        mesh.add_node(Node::new(1, 0.0, 0.0, 0.0)); // Fixed end
        mesh.add_node(Node::new(2, 1.0, 0.0, 0.0)); // Free end

        let elem = Element::new(1, ElementType::B31, vec![1, 2]);
        let _ = mesh.add_element(elem);
        mesh.calculate_dofs();

        // Material with density
        let mut materials = MaterialLibrary::new();
        let steel = Material {
            name: "STEEL".to_string(),
            model: MaterialModel::LinearElastic,
            elastic_modulus: Some(200e9), // Pa
            poissons_ratio: Some(0.3),
            density: Some(7850.0), // kg/m³
            thermal_expansion: None,
            conductivity: None,
            specific_heat: None,
        };
        materials.add_material(steel);
        materials.assign_material(1, "STEEL".to_string());

        // Boundary conditions: Fix node 1 (all 6 DOFs)
        let mut bcs = BoundaryConditions::new();
        bcs.add_displacement_bc(DisplacementBC::new(1, 1, 6, 0.0));

        (mesh, materials, bcs)
    }

    #[test]
    fn creates_modal_solver() {
        let (mesh, materials, bcs) = make_simple_cantilever_beam();
        let solver = ModalSolver::new(&mesh, &materials, &bcs, 0.01);

        // Should create successfully
        assert_eq!(solver.default_area, 0.01);
    }

    #[test]
    fn assembles_system_with_mass() {
        let (mesh, materials, bcs) = make_simple_cantilever_beam();
        let solver = ModalSolver::new(&mesh, &materials, &bcs, 0.01);

        let system = solver.assemble_system();
        assert!(system.is_ok(), "System assembly should succeed");

        let system = system.unwrap();
        assert!(system.mass.is_some(), "Mass matrix should be assembled");
    }

    #[test]
    fn extracts_free_dofs() {
        let (mesh, materials, bcs) = make_simple_cantilever_beam();
        let solver = ModalSolver::new(&mesh, &materials, &bcs, 0.01);

        let system = solver.assemble_system().unwrap();
        let free_dofs = solver.extract_free_dofs(&system).unwrap();

        // Node 1 is fixed (DOFs 0-5), Node 2 is free (DOFs 6-11)
        // So free_dofs should be [6, 7, 8, 9, 10, 11]
        assert_eq!(free_dofs.len(), 6);
        assert_eq!(free_dofs, vec![6, 7, 8, 9, 10, 11]);
    }

    #[test]
    fn reduces_matrix() {
        let (mesh, materials, bcs) = make_simple_cantilever_beam();
        let solver = ModalSolver::new(&mesh, &materials, &bcs, 0.01);

        // Create a simple test matrix
        let matrix = DMatrix::from_row_slice(4, 4, &[
            1.0, 2.0, 3.0, 4.0,
            2.0, 5.0, 6.0, 7.0,
            3.0, 6.0, 8.0, 9.0,
            4.0, 7.0, 9.0, 10.0,
        ]);

        // Extract DOFs [0, 2] (skip 1, 3)
        let free_dofs = vec![0, 2];
        let reduced = solver.reduce_matrix(&matrix, &free_dofs);

        // Should get 2×2 matrix with elements [0,0], [0,2], [2,0], [2,2]
        assert_eq!(reduced.nrows(), 2);
        assert_eq!(reduced.ncols(), 2);
        assert_eq!(reduced[(0, 0)], 1.0);
        assert_eq!(reduced[(0, 1)], 3.0);
        assert_eq!(reduced[(1, 0)], 3.0);
        assert_eq!(reduced[(1, 1)], 8.0);
    }

    #[test]
    fn expands_mode_shapes() {
        let (mesh, materials, bcs) = make_simple_cantilever_beam();
        let solver = ModalSolver::new(&mesh, &materials, &bcs, 0.01);

        // Create reduced mode shapes (2 modes, 3 free DOFs)
        let reduced = DMatrix::from_row_slice(3, 2, &[
            1.0, 2.0,
            3.0, 4.0,
            5.0, 6.0,
        ]);

        // Free DOFs are [1, 3, 4] out of total 6 DOFs
        let free_dofs = vec![1, 3, 4];
        let expanded = solver.expand_mode_shapes(&reduced, &free_dofs, 6);

        // Should be 6×2 with zeros at DOFs 0, 2, 5
        assert_eq!(expanded.nrows(), 6);
        assert_eq!(expanded.ncols(), 2);

        // Check mode 1
        assert_eq!(expanded[(0, 0)], 0.0); // Constrained
        assert_eq!(expanded[(1, 0)], 1.0); // Free
        assert_eq!(expanded[(2, 0)], 0.0); // Constrained
        assert_eq!(expanded[(3, 0)], 3.0); // Free
        assert_eq!(expanded[(4, 0)], 5.0); // Free
        assert_eq!(expanded[(5, 0)], 0.0); // Constrained
    }
}
