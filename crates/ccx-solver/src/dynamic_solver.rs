//! Dynamic analysis solver using Newmark time integration.
//!
//! Solves the transient structural dynamics problem:
//! M*ü + C*u̇ + K*u = F(t)
//!
//! where:
//! - M = global mass matrix
//! - C = damping matrix (Rayleigh damping: C = αM + βK)
//! - K = global stiffness matrix
//! - F(t) = time-varying external force vector
//! - u, u̇, ü = displacement, velocity, acceleration vectors
//!
//! # Newmark Method
//!
//! The Newmark β-method is an implicit time integration scheme:
//!
//! ```text
//! u_{n+1} = u_n + Δt*u̇_n + (Δt²/2)*[(1-2β)*ü_n + 2β*ü_{n+1}]
//! u̇_{n+1} = u̇_n + Δt*[(1-γ)*ü_n + γ*ü_{n+1}]
//! ```
//!
//! Standard parameter choices:
//! - **Average acceleration** (unconditionally stable): γ = 1/2, β = 1/4
//! - **Linear acceleration**: γ = 1/2, β = 1/6
//! - **Fox-Goodwin**: γ = 1/2, β = 1/12
//!
//! # Example
//!
//! ```no_run
//! use ccx_solver::{Mesh, MaterialLibrary, BoundaryConditions, DynamicSolver, NewmarkConfig};
//!
//! # fn example(mesh: Mesh, materials: MaterialLibrary, bcs: BoundaryConditions) {
//! let config = NewmarkConfig::average_acceleration();
//! let solver = DynamicSolver::new(&mesh, &materials, &bcs, 0.01, config);
//!
//! let results = solver.solve(0.0, 1.0, 0.001).expect("Dynamic analysis failed");
//!
//! println!("Computed {} time steps", results.time_steps.len());
//! println!("Final displacement: {:?}", results.displacements.last());
//! # }
//! ```

use crate::assembly::GlobalSystem;
use crate::boundary_conditions::BoundaryConditions;
use crate::materials::MaterialLibrary;
use crate::mesh::Mesh;
use nalgebra::{DMatrix, DVector};

/// Newmark time integration parameters
#[derive(Debug, Clone, Copy)]
pub struct NewmarkConfig {
    /// Newmark β parameter (controls acceleration)
    pub beta: f64,
    /// Newmark γ parameter (controls velocity)
    pub gamma: f64,
    /// Rayleigh damping α (mass-proportional)
    pub alpha_damping: f64,
    /// Rayleigh damping β (stiffness-proportional)
    pub beta_damping: f64,
}

impl NewmarkConfig {
    /// Average acceleration method (unconditionally stable, 2nd order accurate)
    ///
    /// γ = 1/2, β = 1/4
    pub fn average_acceleration() -> Self {
        Self {
            beta: 0.25,
            gamma: 0.5,
            alpha_damping: 0.0,
            beta_damping: 0.0,
        }
    }

    /// Linear acceleration method (conditionally stable)
    ///
    /// γ = 1/2, β = 1/6
    pub fn linear_acceleration() -> Self {
        Self {
            beta: 1.0 / 6.0,
            gamma: 0.5,
            alpha_damping: 0.0,
            beta_damping: 0.0,
        }
    }

    /// Fox-Goodwin method
    ///
    /// γ = 1/2, β = 1/12
    pub fn fox_goodwin() -> Self {
        Self {
            beta: 1.0 / 12.0,
            gamma: 0.5,
            alpha_damping: 0.0,
            beta_damping: 0.0,
        }
    }

    /// Set Rayleigh damping parameters
    ///
    /// C = α*M + β*K
    ///
    /// # Arguments
    /// * `alpha` - Mass-proportional damping coefficient
    /// * `beta` - Stiffness-proportional damping coefficient
    pub fn with_rayleigh_damping(mut self, alpha: f64, beta: f64) -> Self {
        self.alpha_damping = alpha;
        self.beta_damping = beta;
        self
    }

    /// Compute damping ratios from modal frequencies
    ///
    /// Given two modal frequencies and desired damping ratios, compute α and β
    ///
    /// # Arguments
    /// * `freq1` - First modal frequency (Hz)
    /// * `zeta1` - Damping ratio for freq1 (e.g., 0.05 for 5%)
    /// * `freq2` - Second modal frequency (Hz)
    /// * `zeta2` - Damping ratio for freq2
    pub fn from_modal_damping(mut self, freq1: f64, freq2: f64, zeta1: f64, zeta2: f64) -> Self {
        let omega1 = 2.0 * std::f64::consts::PI * freq1;
        let omega2 = 2.0 * std::f64::consts::PI * freq2;

        // Solve: ζ1 = (α/(2*ω1)) + (β*ω1/2)
        //        ζ2 = (α/(2*ω2)) + (β*ω2/2)
        let alpha = 2.0 * (zeta1 * omega1 - zeta2 * omega2) / (1.0 / omega1 - 1.0 / omega2);
        let beta = 2.0 * (zeta2 - zeta1) / (omega2 - omega1);

        self.alpha_damping = alpha;
        self.beta_damping = beta;
        self
    }
}

impl Default for NewmarkConfig {
    fn default() -> Self {
        Self::average_acceleration()
    }
}

/// Dynamic analysis results
#[derive(Debug, Clone)]
pub struct DynamicResults {
    /// Time steps
    pub time_steps: Vec<f64>,
    /// Displacements at each time step (num_time_steps × num_dofs)
    pub displacements: Vec<DVector<f64>>,
    /// Velocities at each time step
    pub velocities: Vec<DVector<f64>>,
    /// Accelerations at each time step
    pub accelerations: Vec<DVector<f64>>,
}

impl DynamicResults {
    /// Get displacement at a specific time step
    pub fn displacement_at(&self, step: usize) -> Option<&DVector<f64>> {
        self.displacements.get(step)
    }

    /// Get velocity at a specific time step
    pub fn velocity_at(&self, step: usize) -> Option<&DVector<f64>> {
        self.velocities.get(step)
    }

    /// Get acceleration at a specific time step
    pub fn acceleration_at(&self, step: usize) -> Option<&DVector<f64>> {
        self.accelerations.get(step)
    }

    /// Get number of time steps
    pub fn num_steps(&self) -> usize {
        self.time_steps.len()
    }
}

/// Dynamic analysis solver
pub struct DynamicSolver<'a> {
    mesh: &'a Mesh,
    materials: &'a MaterialLibrary,
    bcs: &'a BoundaryConditions,
    default_area: f64,
    config: NewmarkConfig,
}

impl<'a> DynamicSolver<'a> {
    /// Create a new dynamic solver
    ///
    /// # Arguments
    /// * `mesh` - Finite element mesh
    /// * `materials` - Material library (must include density)
    /// * `bcs` - Boundary conditions (displacement constraints and loads)
    /// * `default_area` - Default cross-sectional area or thickness
    /// * `config` - Newmark integration parameters
    pub fn new(
        mesh: &'a Mesh,
        materials: &'a MaterialLibrary,
        bcs: &'a BoundaryConditions,
        default_area: f64,
        config: NewmarkConfig,
    ) -> Self {
        Self {
            mesh,
            materials,
            bcs,
            default_area,
            config,
        }
    }

    /// Solve the dynamic analysis problem
    ///
    /// # Arguments
    /// * `t_start` - Start time
    /// * `t_end` - End time
    /// * `dt` - Time step size
    ///
    /// # Returns
    /// Dynamic analysis results with displacements, velocities, and accelerations at each time step
    ///
    /// # Errors
    /// Returns error if:
    /// - Mass matrix assembly fails (e.g., missing density)
    /// - Time step is non-positive
    /// - Effective stiffness matrix is singular
    pub fn solve(&self, t_start: f64, t_end: f64, dt: f64) -> Result<DynamicResults, String> {
        if dt <= 0.0 {
            return Err("Time step must be positive".to_string());
        }

        if t_end <= t_start {
            return Err("End time must be greater than start time".to_string());
        }

        // Step 1: Assemble global matrices (K, M, C)
        let system = self.assemble_system()?;

        // Step 2: Compute damping matrix C = αM + βK
        let c = self.compute_damping_matrix(&system)?;

        // Step 3: Initialize state (u0, v0, a0)
        let (u, v, a) = self.initialize_state(&system)?;

        // Step 4: Compute effective stiffness matrix for Newmark
        let k_eff = self.compute_effective_stiffness(&system, &c, dt)?;

        // Step 5: Time integration loop
        let num_steps = ((t_end - t_start) / dt).ceil() as usize + 1;
        let mut results = DynamicResults {
            time_steps: Vec::with_capacity(num_steps),
            displacements: Vec::with_capacity(num_steps),
            velocities: Vec::with_capacity(num_steps),
            accelerations: Vec::with_capacity(num_steps),
        };

        // Store initial state
        results.time_steps.push(t_start);
        results.displacements.push(u.clone());
        results.velocities.push(v.clone());
        results.accelerations.push(a.clone());

        // Time integration
        let mut u_n = u;
        let mut v_n = v;
        let mut a_n = a;

        for step in 1..num_steps {
            let t = t_start + (step as f64) * dt;

            // Newmark step
            let (u_next, v_next, a_next) = self.newmark_step(
                &system,
                &c,
                &k_eff,
                &u_n,
                &v_n,
                &a_n,
                t,
                dt,
            )?;

            // Store results
            results.time_steps.push(t);
            results.displacements.push(u_next.clone());
            results.velocities.push(v_next.clone());
            results.accelerations.push(a_next.clone());

            // Update state
            u_n = u_next;
            v_n = v_next;
            a_n = a_next;
        }

        Ok(results)
    }

    /// Assemble global matrices (K, M)
    fn assemble_system(&self) -> Result<GlobalSystem, String> {
        // Assemble stiffness and force
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

        // Assemble mass matrix (required for dynamic analysis)
        system.assemble_mass(self.mesh, self.materials, self.default_area, max_dofs_per_node)?;

        Ok(system)
    }

    /// Compute damping matrix C = αM + βK
    fn compute_damping_matrix(&self, system: &GlobalSystem) -> Result<DMatrix<f64>, String> {
        let m = system.mass.as_ref().ok_or("Mass matrix not assembled")?;
        let k = &system.stiffness;

        let alpha = self.config.alpha_damping;
        let beta = self.config.beta_damping;

        let c = alpha * m + beta * k;
        Ok(c)
    }

    /// Initialize displacement, velocity, and acceleration
    fn initialize_state(
        &self,
        system: &GlobalSystem,
    ) -> Result<(DVector<f64>, DVector<f64>, DVector<f64>), String> {
        let n = system.num_dofs;

        // Initial displacement (zero or from boundary conditions)
        let u0 = DVector::zeros(n);

        // Initial velocity (zero)
        let v0 = DVector::zeros(n);

        // Initial acceleration: a0 = M^-1 * (F0 - K*u0 - C*v0)
        // For simplicity, assume a0 = M^-1 * F0
        let f0 = system.force.clone();
        let m = system.mass.as_ref().ok_or("Mass matrix not assembled")?;

        // Solve M*a0 = F0
        let a0 = m
            .clone()
            .lu()
            .solve(&f0)
            .ok_or("Failed to solve for initial acceleration")?;

        Ok((u0, v0, a0))
    }

    /// Compute effective stiffness matrix for Newmark method
    ///
    /// K_eff = K + (γ/(β*Δt))*C + (1/(β*Δt²))*M
    fn compute_effective_stiffness(
        &self,
        system: &GlobalSystem,
        c: &DMatrix<f64>,
        dt: f64,
    ) -> Result<DMatrix<f64>, String> {
        let k = &system.stiffness;
        let m = system.mass.as_ref().ok_or("Mass matrix not assembled")?;

        let beta = self.config.beta;
        let gamma = self.config.gamma;

        let dt2 = dt * dt;
        let coeff_c = gamma / (beta * dt);
        let coeff_m = 1.0 / (beta * dt2);

        let k_eff = k + coeff_c * c + coeff_m * m;

        Ok(k_eff)
    }

    /// Perform one Newmark time step
    #[allow(clippy::too_many_arguments)]
    fn newmark_step(
        &self,
        system: &GlobalSystem,
        c: &DMatrix<f64>,
        k_eff: &DMatrix<f64>,
        u_n: &DVector<f64>,
        v_n: &DVector<f64>,
        a_n: &DVector<f64>,
        t: f64,
        dt: f64,
    ) -> Result<(DVector<f64>, DVector<f64>, DVector<f64>), String> {
        let beta = self.config.beta;
        let gamma = self.config.gamma;

        let m = system.mass.as_ref().ok_or("Mass matrix not assembled")?;

        // Compute effective force at t_{n+1}
        let f_next = self.compute_force_at_time(system, t)?;

        // Newmark predictors
        let dt2 = dt * dt;

        // Effective force: F_eff = F_{n+1} + M*[a_n/(β*Δt²) + v_n/(β*Δt) + ((1-2β)/(2β))*a_n]
        //                               + C*[γ*a_n/(β*Δt) + (γ-β)/β*v_n + Δt*(γ-2β)/(2β)*a_n]
        let m_term = a_n / (beta * dt2) + v_n / (beta * dt) + ((1.0 - 2.0 * beta) / (2.0 * beta)) * a_n;
        let c_term = gamma * a_n / (beta * dt) + ((gamma - beta) / beta) * v_n
            + (dt * (gamma - 2.0 * beta) / (2.0 * beta)) * a_n;

        let f_eff = &f_next + m * m_term + c * c_term;

        // Solve K_eff * u_{n+1} = F_eff
        let u_next = k_eff
            .clone()
            .lu()
            .solve(&f_eff)
            .ok_or("Failed to solve effective system (singular matrix?)")?;

        // Compute acceleration at n+1
        let a_next = (&u_next - u_n) / (beta * dt2) - v_n / (beta * dt) - ((1.0 - 2.0 * beta) / (2.0 * beta)) * a_n;

        // Compute velocity at n+1
        let v_next = v_n + dt * ((1.0 - gamma) * a_n + gamma * &a_next);

        Ok((u_next, v_next, a_next))
    }

    /// Compute external force vector at given time
    ///
    /// For now, assumes constant force from boundary conditions
    /// TODO: Support time-varying loads
    fn compute_force_at_time(
        &self,
        system: &GlobalSystem,
        _t: f64,
    ) -> Result<DVector<f64>, String> {
        // For now, return constant force from system
        // Future: implement time-varying loads (sine, ramp, impact, etc.)
        Ok(system.force.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary_conditions::ConcentratedLoad;
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

        // Boundary conditions: Fix node 1 (all 6 DOFs), apply load at node 2
        let mut bcs = BoundaryConditions::new();
        use crate::boundary_conditions::DisplacementBC;
        bcs.add_displacement_bc(DisplacementBC::new(1, 1, 6, 0.0));

        // Apply vertical load at free end
        bcs.add_concentrated_load(ConcentratedLoad::new(2, 2, -1000.0)); // -1kN in y

        (mesh, materials, bcs)
    }

    #[test]
    fn test_newmark_config_average_acceleration() {
        let config = NewmarkConfig::average_acceleration();
        assert_eq!(config.beta, 0.25);
        assert_eq!(config.gamma, 0.5);
    }

    #[test]
    fn test_newmark_config_modal_damping() {
        let config = NewmarkConfig::default().from_modal_damping(10.0, 100.0, 0.05, 0.05);

        // Both frequencies have 5% damping, so α and β should be chosen accordingly
        assert!(config.alpha_damping != 0.0 || config.beta_damping != 0.0);
    }

    #[test]
    fn test_creates_dynamic_solver() {
        let (mesh, materials, bcs) = make_simple_cantilever_beam();
        let config = NewmarkConfig::average_acceleration();
        let solver = DynamicSolver::new(&mesh, &materials, &bcs, 0.01, config);

        assert_eq!(solver.config.beta, 0.25);
    }

    #[test]
    fn test_assembles_system_with_mass() {
        let (mesh, materials, bcs) = make_simple_cantilever_beam();
        let config = NewmarkConfig::default();
        let solver = DynamicSolver::new(&mesh, &materials, &bcs, 0.01, config);

        let system = solver.assemble_system();
        assert!(system.is_ok(), "System assembly should succeed");

        let system = system.unwrap();
        assert!(system.mass.is_some(), "Mass matrix should be assembled");
    }

    #[test]
    fn test_computes_damping_matrix() {
        let (mesh, materials, bcs) = make_simple_cantilever_beam();
        let config = NewmarkConfig::default().with_rayleigh_damping(0.1, 0.001);
        let solver = DynamicSolver::new(&mesh, &materials, &bcs, 0.01, config);

        let system = solver.assemble_system().unwrap();
        let c = solver.compute_damping_matrix(&system);

        assert!(c.is_ok(), "Damping matrix computation should succeed");
        let c = c.unwrap();
        assert_eq!(c.nrows(), system.num_dofs);
        assert_eq!(c.ncols(), system.num_dofs);
    }

    #[test]
    fn test_initializes_state() {
        let (mesh, materials, bcs) = make_simple_cantilever_beam();
        let config = NewmarkConfig::default();
        let solver = DynamicSolver::new(&mesh, &materials, &bcs, 0.01, config);

        let system = solver.assemble_system().unwrap();
        let (u0, v0, a0) = solver.initialize_state(&system).unwrap();

        assert_eq!(u0.len(), system.num_dofs);
        assert_eq!(v0.len(), system.num_dofs);
        assert_eq!(a0.len(), system.num_dofs);
    }
}
