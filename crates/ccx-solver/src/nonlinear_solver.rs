//! Nonlinear static analysis solver using Newton-Raphson iteration.
//!
//! Solves the nonlinear equilibrium equation:
//! R(u) = F_ext - F_int(u) = 0
//!
//! where:
//! - R = residual force vector
//! - F_ext = external applied forces
//! - F_int = internal forces (function of displacement u)
//! - u = displacement vector
//!
//! # Newton-Raphson Method
//!
//! Iterative solution:
//! 1. Compute residual: R_i = F_ext - F_int(u_i)
//! 2. Compute tangent stiffness: K_T = ∂F_int/∂u
//! 3. Solve: K_T * Δu = R_i
//! 4. Update: u_{i+1} = u_i + Δu
//! 5. Check convergence: ||R_i|| < tol
//!
//! # Convergence Criteria
//!
//! - **Force residual**: ||R|| / ||F_ext|| < tol_force
//! - **Displacement increment**: ||Δu|| / ||u|| < tol_disp
//! - **Energy**: |Δu·R| / |u·F_ext| < tol_energy
//!
//! # Example
//!
//! ```no_run
//! use ccx_solver::{Mesh, MaterialLibrary, BoundaryConditions, NonlinearSolver, NonlinearConfig};
//!
//! # fn example(mesh: Mesh, materials: MaterialLibrary, bcs: BoundaryConditions) {
//! let config = NonlinearConfig::default();
//! let solver = NonlinearSolver::new(&mesh, &materials, &bcs, 0.01, config);
//!
//! let results = solver.solve().expect("Nonlinear analysis failed");
//!
//! println!("Converged in {} iterations", results.num_iterations);
//! println!("Final displacement norm: {:.6}", results.displacement.norm());
//! # }
//! ```

use crate::assembly::GlobalSystem;
use crate::boundary_conditions::BoundaryConditions;
use crate::materials::MaterialLibrary;
use crate::mesh::Mesh;
use nalgebra::DVector;

/// Nonlinear solver configuration
#[derive(Debug, Clone, Copy)]
pub struct NonlinearConfig {
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Force residual tolerance
    pub tol_force: f64,
    /// Displacement tolerance
    pub tol_displacement: f64,
    /// Energy tolerance
    pub tol_energy: f64,
    /// Line search flag
    pub use_line_search: bool,
    /// Maximum line search steps
    pub max_line_search: usize,
}

impl Default for NonlinearConfig {
    fn default() -> Self {
        Self {
            max_iterations: 50,
            tol_force: 1e-6,
            tol_displacement: 1e-8,
            tol_energy: 1e-10,
            use_line_search: true,
            max_line_search: 10,
        }
    }
}

/// Convergence status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConvergenceStatus {
    /// Converged (met all criteria)
    Converged,
    /// Not converged yet
    NotConverged,
    /// Diverged (residual increasing)
    Diverged,
}

/// Nonlinear analysis results
#[derive(Debug, Clone)]
pub struct NonlinearResults {
    /// Final displacement solution
    pub displacement: DVector<f64>,
    /// Number of iterations to convergence
    pub num_iterations: usize,
    /// Final residual norm
    pub residual_norm: f64,
    /// Convergence status
    pub status: ConvergenceStatus,
    /// Iteration history (residual norms)
    pub iteration_history: Vec<f64>,
}

/// Nonlinear static analysis solver
pub struct NonlinearSolver<'a> {
    mesh: &'a Mesh,
    materials: &'a MaterialLibrary,
    bcs: &'a BoundaryConditions,
    default_area: f64,
    config: NonlinearConfig,
}

impl<'a> NonlinearSolver<'a> {
    /// Create a new nonlinear solver
    ///
    /// # Arguments
    /// * `mesh` - Finite element mesh
    /// * `materials` - Material library
    /// * `bcs` - Boundary conditions (displacement constraints and loads)
    /// * `default_area` - Default cross-sectional area or thickness
    /// * `config` - Nonlinear solver configuration
    pub fn new(
        mesh: &'a Mesh,
        materials: &'a MaterialLibrary,
        bcs: &'a BoundaryConditions,
        default_area: f64,
        config: NonlinearConfig,
    ) -> Self {
        Self {
            mesh,
            materials,
            bcs,
            default_area,
            config,
        }
    }

    /// Solve the nonlinear equilibrium problem
    ///
    /// # Returns
    /// Nonlinear analysis results with displacement and convergence info
    ///
    /// # Errors
    /// Returns error if:
    /// - System assembly fails
    /// - Tangent stiffness is singular
    /// - Maximum iterations exceeded without convergence
    pub fn solve(&self) -> Result<NonlinearResults, String> {
        // Step 1: Assemble initial system (linear)
        let system = GlobalSystem::assemble(
            self.mesh,
            self.materials,
            self.bcs,
            self.default_area,
        )?;

        // Step 2: Initialize displacement
        let mut u = DVector::zeros(system.num_dofs);

        // Step 3: Newton-Raphson iteration
        let mut iteration_history = Vec::new();
        let mut status = ConvergenceStatus::NotConverged;

        for iter in 0..self.config.max_iterations {
            // Compute residual: R = F_ext - F_int(u)
            let r = self.compute_residual(&system, &u)?;
            let r_norm = r.norm();
            iteration_history.push(r_norm);

            // Check convergence
            let f_ext_norm = system.force.norm();
            let converged = self.check_convergence(&u, &r, f_ext_norm);

            if converged {
                status = ConvergenceStatus::Converged;
                return Ok(NonlinearResults {
                    displacement: u,
                    num_iterations: iter + 1,
                    residual_norm: r_norm,
                    status,
                    iteration_history,
                });
            }

            // Check divergence
            if iter > 0 && r_norm > iteration_history[iter - 1] * 10.0 {
                status = ConvergenceStatus::Diverged;
                return Err(format!(
                    "Newton-Raphson diverged at iteration {} (residual = {:.3e})",
                    iter + 1,
                    r_norm
                ));
            }

            // Compute tangent stiffness matrix
            // For now, use linear stiffness (geometric nonlinearity not yet implemented)
            let k_t = system.stiffness.clone();

            // Solve for displacement increment: K_T * Δu = R
            let du = k_t
                .lu()
                .solve(&r)
                .ok_or("Failed to solve tangent system (singular matrix?)")?;

            // Line search (optional)
            let alpha = if self.config.use_line_search {
                self.line_search(&system, &u, &du, &r)?
            } else {
                1.0
            };

            // Update displacement
            u += alpha * du;
        }

        // Maximum iterations reached
        Err(format!(
            "Newton-Raphson failed to converge in {} iterations (final residual = {:.3e})",
            self.config.max_iterations,
            iteration_history.last().unwrap_or(&0.0)
        ))
    }

    /// Compute residual vector R = F_ext - F_int(u)
    ///
    /// For now, assumes F_int = K*u (linear)
    /// TODO: Implement geometric nonlinearity (updated Lagrangian)
    fn compute_residual(
        &self,
        system: &GlobalSystem,
        u: &DVector<f64>,
    ) -> Result<DVector<f64>, String> {
        // F_int = K * u (linear approximation)
        let f_int = &system.stiffness * u;

        // R = F_ext - F_int
        let r = &system.force - f_int;

        Ok(r)
    }

    /// Check convergence based on multiple criteria
    fn check_convergence(&self, u: &DVector<f64>, r: &DVector<f64>, f_ext_norm: f64) -> bool {
        let r_norm = r.norm();
        let u_norm = u.norm();

        // Force convergence
        let force_converged = if f_ext_norm > 1e-12 {
            r_norm / f_ext_norm < self.config.tol_force
        } else {
            r_norm < self.config.tol_force
        };

        // Displacement convergence (if not first iteration)
        let disp_converged = if u_norm > 1e-12 {
            r_norm / u_norm < self.config.tol_displacement
        } else {
            true // Can't check if u ≈ 0
        };

        // Energy convergence
        let energy = (r.dot(u)).abs();
        let energy_ref = (u.norm() * f_ext_norm).max(1.0);
        let energy_converged = energy / energy_ref < self.config.tol_energy;

        force_converged && disp_converged && energy_converged
    }

    /// Perform line search to find optimal step length
    ///
    /// Minimizes ||R(u + α*Δu)||
    fn line_search(
        &self,
        system: &GlobalSystem,
        u: &DVector<f64>,
        du: &DVector<f64>,
        r0: &DVector<f64>,
    ) -> Result<f64, String> {
        let r0_norm = r0.norm();

        // Try different step lengths
        let alphas = [1.0, 0.5, 0.25, 0.125, 0.0625];

        for &alpha in &alphas {
            let u_trial = u + alpha * du;
            let r_trial = self.compute_residual(system, &u_trial)?;
            let r_trial_norm = r_trial.norm();

            // Accept if residual decreases
            if r_trial_norm < r0_norm {
                return Ok(alpha);
            }
        }

        // No improvement found, use full step
        Ok(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary_conditions::{ConcentratedLoad, DisplacementBC};
    use crate::materials::{Material, MaterialModel};
    use crate::mesh::{Element, ElementType, Node};

    fn make_simple_truss() -> (Mesh, MaterialLibrary, BoundaryConditions) {
        let mut mesh = Mesh::new();

        // Simple 2-node truss
        mesh.add_node(Node::new(1, 0.0, 0.0, 0.0)); // Fixed
        mesh.add_node(Node::new(2, 1.0, 0.0, 0.0)); // Loaded

        let elem = Element::new(1, ElementType::T3D2, vec![1, 2]);
        let _ = mesh.add_element(elem);
        mesh.calculate_dofs();

        // Material
        let mut materials = MaterialLibrary::new();
        let steel = Material {
            name: "STEEL".to_string(),
            model: MaterialModel::LinearElastic,
            elastic_modulus: Some(200e9),
            poissons_ratio: Some(0.3),
            density: None,
            thermal_expansion: None,
            conductivity: None,
            specific_heat: None,
        };
        materials.add_material(steel);
        materials.assign_material(1, "STEEL".to_string());

        // Boundary conditions
        let mut bcs = BoundaryConditions::new();
        bcs.add_displacement_bc(DisplacementBC::new(1, 1, 3, 0.0)); // Fix node 1
        bcs.add_concentrated_load(ConcentratedLoad::new(2, 1, 1000.0)); // Load node 2

        (mesh, materials, bcs)
    }

    #[test]
    fn test_nonlinear_config_default() {
        let config = NonlinearConfig::default();
        assert_eq!(config.max_iterations, 50);
        assert_eq!(config.tol_force, 1e-6);
    }

    #[test]
    fn test_creates_nonlinear_solver() {
        let (mesh, materials, bcs) = make_simple_truss();
        let config = NonlinearConfig::default();
        let solver = NonlinearSolver::new(&mesh, &materials, &bcs, 0.01, config);

        assert_eq!(solver.config.max_iterations, 50);
    }

    #[test]
    fn test_solves_linear_problem() {
        let (mesh, materials, bcs) = make_simple_truss();
        let config = NonlinearConfig::default();
        let solver = NonlinearSolver::new(&mesh, &materials, &bcs, 0.01, config);

        let result = solver.solve();
        assert!(result.is_ok(), "Linear problem should converge");

        let result = result.unwrap();
        assert_eq!(result.status, ConvergenceStatus::Converged);
        assert!(result.num_iterations <= 5, "Should converge quickly for linear problem");
    }

    #[test]
    fn test_convergence_check() {
        let (mesh, materials, bcs) = make_simple_truss();
        let config = NonlinearConfig::default();
        let solver = NonlinearSolver::new(&mesh, &materials, &bcs, 0.01, config);

        // Very small residual should converge
        let u = DVector::from_vec(vec![0.0, 0.0, 0.0, 0.001, 0.0, 0.0]);
        let r = DVector::from_vec(vec![0.0, 0.0, 0.0, 1e-10, 0.0, 0.0]);
        let f_norm = 1000.0;

        let converged = solver.check_convergence(&u, &r, f_norm);
        assert!(converged, "Should converge with small residual");
    }
}
