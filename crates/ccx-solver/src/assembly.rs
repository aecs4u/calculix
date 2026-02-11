//! Global matrix assembly for finite element systems.
//!
//! Assembles element stiffness matrices into the global system:
//! - K: Global stiffness matrix (sparse CSR format)
//! - F: Global force vector
//!
//! ## Assembly Process
//!
//! 1. Allocate sparse global stiffness matrix K (num_dofs × num_dofs)
//! 2. Loop over all elements:
//!    - Compute element stiffness k_e
//!    - Get element DOF indices
//!    - Add k_e contributions to K
//! 3. Build force vector F from boundary conditions
//! 4. Apply displacement boundary conditions
//!
//! ## Sparse Matrix Format
//!
//! Uses Compressed Sparse Row (CSR) format for efficiency:
//! - Only stores non-zero entries
//! - Fast matrix-vector multiplication
//! - Efficient for iterative solvers

use crate::boundary_conditions::BoundaryConditions;
use crate::distributed_loads::DistributedLoadConverter;
use crate::materials::MaterialLibrary;
use crate::mesh::Mesh;
use nalgebra::{DMatrix, DVector};

/// Global finite element system
#[derive(Debug, Clone)]
pub struct GlobalSystem {
    /// Global stiffness matrix (dense for now, sparse later)
    pub stiffness: DMatrix<f64>,
    /// Global mass matrix (optional, only assembled for modal analysis)
    pub mass: Option<DMatrix<f64>>,
    /// Global force vector
    pub force: DVector<f64>,
    /// Number of degrees of freedom
    pub num_dofs: usize,
    /// Constrained DOFs (for boundary conditions)
    pub constrained_dofs: Vec<usize>,
}

impl GlobalSystem {
    /// Create a new empty global system
    pub fn new(num_dofs: usize) -> Self {
        Self {
            stiffness: DMatrix::zeros(num_dofs, num_dofs),
            mass: None,
            force: DVector::zeros(num_dofs),
            num_dofs,
            constrained_dofs: Vec::new(),
        }
    }

    /// Assemble the global system from mesh, materials, and boundary conditions
    ///
    /// # Current Limitations
    /// - Assumes uniform cross-sectional area
    /// - Dense matrix storage (will switch to sparse CSR later)
    ///
    /// # Supported Elements
    /// - T3D2: 2-node truss (3 DOFs/node)
    /// - B31: 2-node beam (6 DOFs/node)
    pub fn assemble(
        mesh: &Mesh,
        materials: &MaterialLibrary,
        bcs: &BoundaryConditions,
        default_area: f64,
    ) -> Result<Self, String> {
        // Determine maximum DOFs per node for mixed meshes
        let max_dofs_per_node = mesh
            .elements
            .values()
            .map(|e| e.element_type.dofs_per_node())
            .max()
            .unwrap_or(3);

        // All nodes get max DOF count to allow mixed element types
        // Use maximum node ID (not count) since node IDs may not be contiguous
        let max_node_id = mesh.nodes.keys().max().copied().unwrap_or(0) as usize;
        let num_dofs = max_node_id * max_dofs_per_node;
        let mut system = Self::new(num_dofs);

        // Assemble stiffness matrix
        system.assemble_stiffness(mesh, materials, default_area, max_dofs_per_node)?;

        // Assemble concentrated forces
        system.assemble_forces(bcs, max_dofs_per_node)?;

        // Assemble distributed loads (pressure, traction, body forces)
        system.assemble_distributed_forces(mesh, materials, bcs, max_dofs_per_node)?;

        // Apply displacement boundary conditions
        system.apply_displacement_bcs(bcs, max_dofs_per_node)?;

        Ok(system)
    }

    /// Assemble element stiffness contributions into global matrix
    fn assemble_stiffness(
        &mut self,
        mesh: &Mesh,
        materials: &MaterialLibrary,
        default_area: f64,
        max_dofs_per_node: usize,
    ) -> Result<(), String> {
        use crate::elements::DynamicElement;

        for (elem_id, element) in &mesh.elements {
            // Get element nodes
            let nodes: Vec<_> = element
                .nodes
                .iter()
                .map(|&node_id| {
                    mesh.nodes
                        .get(&node_id)
                        .cloned()
                        .ok_or(format!("Node {} not found", node_id))
                })
                .collect::<Result<Vec<_>, String>>()?;

            // Get material for this element
            let material = materials
                .get_element_material(*elem_id)
                .ok_or(format!("No material assigned to element {}", elem_id))?;

            // Create element using factory
            let dyn_elem = DynamicElement::from_mesh_element(
                element.element_type,
                *elem_id,
                element.nodes.clone(),
                default_area,
            );

            let dyn_elem = match dyn_elem {
                Some(e) => e,
                None => {
                    eprintln!(
                        "Warning: Unsupported element type {:?}, skipping element {}",
                        element.element_type, elem_id
                    );
                    continue;
                }
            };

            // Compute element stiffness matrix
            let k_e = dyn_elem.stiffness_matrix(&nodes, material)?;

            // Get global DOF indices with correct stride
            let dof_indices = dyn_elem.global_dof_indices(&element.nodes, max_dofs_per_node);

            // Add element contribution to global matrix
            for (i_local, &i_global) in dof_indices.iter().enumerate() {
                for (j_local, &j_global) in dof_indices.iter().enumerate() {
                    self.stiffness[(i_global, j_global)] += k_e[(i_local, j_local)];
                }
            }
        }

        Ok(())
    }

    /// Assemble element mass contributions into global matrix
    ///
    /// # Arguments
    /// * `mesh` - Finite element mesh
    /// * `materials` - Material library
    /// * `default_area` - Default cross-sectional area or thickness
    /// * `max_dofs_per_node` - Maximum DOFs per node (for mixed element types)
    ///
    /// # Notes
    /// This method assembles the global mass matrix using the same scatter-add pattern
    /// as stiffness assembly. The mass matrix is only needed for dynamic analysis
    /// (modal, transient, etc.) and is stored as an Option to avoid unnecessary computation
    /// for static analysis.
    pub fn assemble_mass(
        &mut self,
        mesh: &Mesh,
        materials: &MaterialLibrary,
        default_area: f64,
        max_dofs_per_node: usize,
    ) -> Result<(), String> {
        use crate::elements::DynamicElement;

        // Initialize mass matrix
        self.mass = Some(DMatrix::zeros(self.num_dofs, self.num_dofs));

        for (elem_id, element) in &mesh.elements {
            // Get element nodes
            let nodes: Vec<_> = element
                .nodes
                .iter()
                .map(|&node_id| {
                    mesh.nodes
                        .get(&node_id)
                        .cloned()
                        .ok_or(format!("Node {} not found", node_id))
                })
                .collect::<Result<Vec<_>, String>>()?;

            // Get material for this element
            let material = materials
                .get_element_material(*elem_id)
                .ok_or(format!("No material assigned to element {}", elem_id))?;

            // Create element using factory
            let dyn_elem = DynamicElement::from_mesh_element(
                element.element_type,
                *elem_id,
                element.nodes.clone(),
                default_area,
            );

            let dyn_elem = match dyn_elem {
                Some(e) => e,
                None => {
                    eprintln!(
                        "Warning: Unsupported element type {:?}, skipping element {}",
                        element.element_type, elem_id
                    );
                    continue;
                }
            };

            // Compute element mass matrix
            let m_e = dyn_elem.mass_matrix(&nodes, material)?;

            // Get global DOF indices with correct stride
            let dof_indices = dyn_elem.global_dof_indices(&element.nodes, max_dofs_per_node);

            // Add element contribution to global mass matrix
            if let Some(ref mut mass_matrix) = self.mass {
                for (i_local, &i_global) in dof_indices.iter().enumerate() {
                    for (j_local, &j_global) in dof_indices.iter().enumerate() {
                        mass_matrix[(i_global, j_global)] += m_e[(i_local, j_local)];
                    }
                }
            }
        }

        Ok(())
    }

    /// Assemble concentrated loads into force vector
    fn assemble_forces(
        &mut self,
        bcs: &BoundaryConditions,
        max_dofs_per_node: usize,
    ) -> Result<(), String> {
        for load in &bcs.concentrated_loads {
            let dof_index = (load.node - 1) as usize * max_dofs_per_node + (load.dof - 1);

            if dof_index >= self.num_dofs {
                return Err(format!(
                    "Load DOF index {} out of range (max {})",
                    dof_index, self.num_dofs
                ));
            }

            self.force[dof_index] += load.magnitude;
        }

        Ok(())
    }

    /// Assemble distributed loads (pressure, traction, body forces) into force vector
    ///
    /// Converts distributed loads to equivalent nodal forces using numerical integration.
    /// Called after concentrated loads but before displacement BCs are applied.
    ///
    /// # Arguments
    /// * `mesh` - The finite element mesh
    /// * `materials` - Material library (unused for pressure loads, reserved for body forces)
    /// * `bcs` - Boundary conditions containing distributed loads
    /// * `max_dofs_per_node` - Maximum DOFs per node for DOF indexing
    ///
    /// # Errors
    /// Returns error if element resolution or force conversion fails
    fn assemble_distributed_forces(
        &mut self,
        mesh: &Mesh,
        materials: &MaterialLibrary,
        bcs: &BoundaryConditions,
        max_dofs_per_node: usize,
    ) -> Result<(), String> {
        // Skip if no distributed loads
        if bcs.distributed_loads.is_empty() {
            return Ok(());
        }

        let converter = DistributedLoadConverter::new(mesh, materials);

        for load in &bcs.distributed_loads {
            // Convert distributed load to nodal forces
            let nodal_forces = converter.convert_to_nodal_forces(load)?;

            // Accumulate nodal forces into global force vector
            for (node_id, force_vec) in nodal_forces {
                let base_dof = ((node_id - 1) as usize) * max_dofs_per_node;

                // Add all 6 DOF components (Fx, Fy, Fz, Mx, My, Mz)
                for dof in 0..6 {
                    let dof_index = base_dof + dof;
                    if dof_index < self.num_dofs {
                        self.force[dof_index] += force_vec[dof];
                    }
                }
            }
        }

        Ok(())
    }

    /// Apply displacement boundary conditions using penalty method
    ///
    /// For each constrained DOF:
    /// - If prescribed displacement = 0: Set large diagonal entry
    /// - If prescribed displacement ≠ 0: Modify force vector
    fn apply_displacement_bcs(
        &mut self,
        bcs: &BoundaryConditions,
        max_dofs_per_node: usize,
    ) -> Result<(), String> {
        let penalty = 1e10; // Large penalty factor

        for bc in &bcs.displacement_bcs {
            for dof in bc.first_dof..=bc.last_dof {
                let dof_index = (bc.node - 1) as usize * max_dofs_per_node + (dof - 1);

                if dof_index >= self.num_dofs {
                    return Err(format!(
                        "BC DOF index {} out of range (max {})",
                        dof_index, self.num_dofs
                    ));
                }

                // Apply penalty method
                self.stiffness[(dof_index, dof_index)] += penalty;
                self.force[dof_index] += penalty * bc.value;

                self.constrained_dofs.push(dof_index);
            }
        }

        Ok(())
    }

    /// Check if the system is ready to solve
    pub fn validate(&self) -> Result<(), String> {
        // Check for zero diagonal entries (excluding constrained DOFs)
        for i in 0..self.num_dofs {
            if !self.constrained_dofs.contains(&i) && self.stiffness[(i, i)].abs() < 1e-10 {
                return Err(format!("Zero diagonal entry at DOF {}", i));
            }
        }

        // Check for symmetry
        for i in 0..self.num_dofs {
            for j in (i + 1)..self.num_dofs {
                let diff = (self.stiffness[(i, j)] - self.stiffness[(j, i)]).abs();
                if diff > 1e-6 {
                    return Err(format!(
                        "Stiffness matrix not symmetric at ({}, {}): diff = {}",
                        i, j, diff
                    ));
                }
            }
        }

        Ok(())
    }

    /// Solve the linear system K * u = F
    ///
    /// Uses LU decomposition for small systems.
    pub fn solve(&self) -> Result<DVector<f64>, String> {
        // Use LU decomposition
        let lu = self
            .stiffness
            .clone()
            .lu()
            .solve(&self.force)
            .ok_or("Failed to solve linear system (singular matrix?)")?;

        Ok(lu)
    }

    /// Export the assembled system as backend-agnostic `LinearSystemData`.
    ///
    /// Converts the dense stiffness matrix to COO triplet format
    /// suitable for consumption by any solver backend.
    pub fn to_linear_system_data(&self) -> crate::backend::LinearSystemData {
        let n = self.num_dofs;
        let mut rows = Vec::new();
        let mut cols = Vec::new();
        let mut vals = Vec::new();

        for i in 0..n {
            for j in 0..n {
                let v = self.stiffness[(i, j)];
                if v.abs() > 1e-30 {
                    rows.push(i);
                    cols.push(j);
                    vals.push(v);
                }
            }
        }

        crate::backend::LinearSystemData {
            stiffness: crate::backend::SparseTripletsF64 {
                nrows: n,
                ncols: n,
                row_indices: rows,
                col_indices: cols,
                values: vals,
            },
            force: self.force.clone(),
            num_dofs: n,
            constrained_dofs: self.constrained_dofs.clone(),
        }
    }

    /// Solve using a specified solver backend.
    ///
    /// This allows plugging in PETSc, native, or any other backend
    /// that implements `LinearSolver`.
    pub fn solve_with_backend(
        &self,
        backend: &dyn crate::backend::LinearSolver,
    ) -> Result<DVector<f64>, String> {
        let data = self.to_linear_system_data();
        let (u, _info) = backend.solve_linear(&data).map_err(|e| e.0)?;
        Ok(u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary_conditions::ConcentratedLoad;
    use crate::materials::Material;
    use crate::mesh::{Element, ElementType, Node};

    fn make_simple_truss_mesh() -> Mesh {
        let mut mesh = Mesh::new();

        // Two nodes: 1m apart along x-axis
        mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
        mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));

        // One truss element connecting them
        let elem = Element::new(1, ElementType::T3D2, vec![1, 2]);
        let _ = mesh.add_element(elem);

        // Calculate DOFs
        mesh.calculate_dofs();

        mesh
    }

    fn make_material_library() -> MaterialLibrary {
        let mut library = MaterialLibrary::new();

        let mut steel = Material::new("STEEL".to_string());
        steel.elastic_modulus = Some(210000.0); // MPa
        steel.poissons_ratio = Some(0.3);
        library.add_material(steel);

        // Assign material to element 1
        library.assign_material(1, "STEEL".to_string());

        library
    }

    fn make_simple_bcs() -> BoundaryConditions {
        let mut bcs = BoundaryConditions::new();

        // Fix node 1 in all directions
        bcs.add_displacement_bc(crate::boundary_conditions::DisplacementBC::new(
            1, 1, 3, 0.0,
        ));

        // Fix node 2 in y and z directions (truss only resists in x)
        bcs.add_displacement_bc(crate::boundary_conditions::DisplacementBC::new(
            2, 2, 3, 0.0,
        ));

        // Apply 100 N force in x-direction at node 2
        bcs.add_concentrated_load(ConcentratedLoad::new(2, 1, 100.0));

        bcs
    }

    #[test]
    fn creates_empty_system() {
        let system = GlobalSystem::new(6);
        assert_eq!(system.num_dofs, 6);
        assert_eq!(system.stiffness.nrows(), 6);
        assert_eq!(system.stiffness.ncols(), 6);
        assert_eq!(system.force.len(), 6);
    }

    #[test]
    fn assembles_single_truss_element() {
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library();
        let bcs = BoundaryConditions::new(); // No BCs for this test
        let area = 0.01; // 0.01 m²

        let system = GlobalSystem::assemble(&mesh, &materials, &bcs, area).unwrap();

        // k = AE/L = 0.01 * 210000 / 1.0 = 2100
        let expected_k = 2100.0;

        // Check that stiffness matrix has expected structure
        assert!((system.stiffness[(0, 0)] - expected_k).abs() < 1e-6);
        assert!((system.stiffness[(0, 3)] + expected_k).abs() < 1e-6);
        assert!((system.stiffness[(3, 0)] + expected_k).abs() < 1e-6);
        assert!((system.stiffness[(3, 3)] - expected_k).abs() < 1e-6);
    }

    #[test]
    fn assembles_forces() {
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library();
        let bcs = make_simple_bcs();
        let area = 0.01;

        let system = GlobalSystem::assemble(&mesh, &materials, &bcs, area).unwrap();

        // Force at node 2, DOF 1 (x-direction) should be 100 N
        // DOF index = (2-1)*3 + (1-1) = 3
        assert!((system.force[3] - 100.0).abs() < 1e-10);
    }

    #[test]
    fn applies_displacement_bcs() {
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library();
        let bcs = make_simple_bcs();
        let area = 0.01;

        let system = GlobalSystem::assemble(&mesh, &materials, &bcs, area).unwrap();

        // Node 1 (DOFs 0, 1, 2) and Node 2 (DOFs 4, 5) should be constrained
        assert_eq!(system.constrained_dofs.len(), 5);
        assert!(system.constrained_dofs.contains(&0)); // Node 1 x
        assert!(system.constrained_dofs.contains(&1)); // Node 1 y
        assert!(system.constrained_dofs.contains(&2)); // Node 1 z
        assert!(system.constrained_dofs.contains(&4)); // Node 2 y
        assert!(system.constrained_dofs.contains(&5)); // Node 2 z

        // Penalty method should increase diagonal
        assert!(system.stiffness[(0, 0)] > 1e9);
        assert!(system.stiffness[(1, 1)] > 1e9);
        assert!(system.stiffness[(2, 2)] > 1e9);
        assert!(system.stiffness[(4, 4)] > 1e9);
        assert!(system.stiffness[(5, 5)] > 1e9);
    }

    #[test]
    fn validates_system() {
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library();
        let bcs = make_simple_bcs();
        let area = 0.01;

        let system = GlobalSystem::assemble(&mesh, &materials, &bcs, area).unwrap();
        assert!(system.validate().is_ok());
    }

    #[test]
    fn solves_simple_truss() {
        // Analytical solution:
        // Bar: L=1m, A=0.01m², E=210000 MPa
        // BC: Fixed at node 1, Force=100N at node 2
        // displacement u = FL/AE = 100*1/(0.01*210000) = 0.0476... m
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library();
        let bcs = make_simple_bcs();
        let area = 0.01;

        let system = GlobalSystem::assemble(&mesh, &materials, &bcs, area).unwrap();
        let u = system.solve().unwrap();

        // Node 1 should be fixed (u ≈ 0)
        assert!(u[0].abs() < 1e-6);
        assert!(u[1].abs() < 1e-6);
        assert!(u[2].abs() < 1e-6);

        // Node 2 x-displacement: expected ≈ 0.047619 m
        let expected_u = 100.0 * 1.0 / (0.01 * 210000.0);
        assert!((u[3] - expected_u).abs() < 1e-6);

        // Node 2 y and z displacements should be zero
        assert!(u[4].abs() < 1e-6);
        assert!(u[5].abs() < 1e-6);
    }

    #[test]
    fn rejects_missing_material() {
        let mesh = make_simple_truss_mesh();
        let materials = MaterialLibrary::new(); // Empty library
        let bcs = BoundaryConditions::new();
        let area = 0.01;

        let result = GlobalSystem::assemble(&mesh, &materials, &bcs, area);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No material"));
    }

    #[test]
    fn rejects_invalid_dof_in_load() {
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library();
        let mut bcs = BoundaryConditions::new();

        // Add load with invalid node ID
        bcs.add_concentrated_load(ConcentratedLoad::new(999, 1, 100.0));

        let area = 0.01;
        let result = GlobalSystem::assemble(&mesh, &materials, &bcs, area);
        assert!(result.is_err());
    }

    #[test]
    fn symmetry_check() {
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library();
        let bcs = BoundaryConditions::new();
        let area = 0.01;

        let system = GlobalSystem::assemble(&mesh, &materials, &bcs, area).unwrap();

        // Verify matrix is symmetric
        for i in 0..system.num_dofs {
            for j in 0..system.num_dofs {
                assert!(
                    (system.stiffness[(i, j)] - system.stiffness[(j, i)]).abs() < 1e-10,
                    "Not symmetric at ({}, {})",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn multiple_loads() {
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library();
        let mut bcs = BoundaryConditions::new();

        // Add multiple loads at same node
        bcs.add_concentrated_load(ConcentratedLoad::new(2, 1, 50.0));
        bcs.add_concentrated_load(ConcentratedLoad::new(2, 1, 30.0));
        bcs.add_concentrated_load(ConcentratedLoad::new(2, 2, 20.0));

        let area = 0.01;
        let system = GlobalSystem::assemble(&mesh, &materials, &bcs, area).unwrap();

        // Loads should sum: 50 + 30 = 80 in x, 20 in y
        assert!((system.force[3] - 80.0).abs() < 1e-10);
        assert!((system.force[4] - 20.0).abs() < 1e-10);
    }

    #[test]
    fn assembles_distributed_pressure_load() {
        use crate::boundary_conditions::{DistributedLoad, DistributedLoadType};

        let mut mesh = Mesh::new();

        // 1×1 meter square plate in XY plane (S4 element)
        mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
        mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));
        mesh.add_node(Node::new(3, 1.0, 1.0, 0.0));
        mesh.add_node(Node::new(4, 0.0, 1.0, 0.0));

        let elem = Element::new(1, ElementType::S4, vec![1, 2, 3, 4]);
        let _ = mesh.add_element(elem);
        mesh.calculate_dofs();

        let mut materials = MaterialLibrary::new();
        let mut steel = Material::new("STEEL".to_string());
        steel.elastic_modulus = Some(200e9); // 200 GPa
        steel.poissons_ratio = Some(0.3);
        materials.add_material(steel);
        materials.assign_material(1, "STEEL".to_string());

        let mut bcs = BoundaryConditions::new();

        // Add pressure load on element 1
        let pressure_load = DistributedLoad {
            element: "1".to_string(),
            load_type: DistributedLoadType::Pressure,
            magnitude: 1000.0, // 1000 Pa
            parameters: vec![],
        };
        bcs.add_distributed_load(pressure_load);

        let thickness = 0.01; // 1 cm
        let system = GlobalSystem::assemble(&mesh, &materials, &bcs, thickness).unwrap();

        // Verify that force vector has non-zero entries
        // For a uniform pressure on 1×1 plate, total force = pressure × area = 1000 N
        // Distributed to 4 nodes, should be ~250 N per node in z-direction
        // NOTE: Positive pressure = compression = forces in -Z direction (into surface)

        // Sum all z-forces (DOF 3: node 1 z, DOF 9: node 2 z, etc.)
        // Node DOFs: node N has DOFs [(N-1)*6, (N-1)*6+1, ..., (N-1)*6+5]
        let total_z_force = system.force[2] + system.force[8] + system.force[14] + system.force[20];

        // Total force should equal pressure × area (negative because compression)
        let expected_total = -1000.0 * 1.0; // -1000 Pa × 1 m² (compression)
        assert!(
            (total_z_force - expected_total).abs() < 1.0,
            "Total z-force {} should be ~{} N",
            total_z_force,
            expected_total
        );

        // Each node should have roughly equal force (within 20% due to integration)
        // Negative because pressure pushes into surface
        for node_id in 1..=4 {
            let z_dof = ((node_id - 1) as usize) * 6 + 2;
            let node_force = system.force[z_dof];
            assert!(
                node_force > -300.0 && node_force < -200.0,
                "Node {} z-force {} should be ~-250 N",
                node_id,
                node_force
            );
        }
    }

    #[test]
    fn skips_empty_distributed_loads() {
        // Test that assembly doesn't fail when distributed_loads is empty
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library();
        let bcs = BoundaryConditions::new(); // No distributed loads

        let area = 0.01;
        let result = GlobalSystem::assemble(&mesh, &materials, &bcs, area);
        assert!(result.is_ok(), "Assembly should succeed with no distributed loads");
    }

    // ========== Mass Matrix Assembly Tests ==========

    fn make_material_library_with_density() -> MaterialLibrary {
        let mut materials = MaterialLibrary::new();
        let mut steel = Material::new("STEEL".to_string());
        steel.elastic_modulus = Some(210e9); // 210 GPa
        steel.poissons_ratio = Some(0.3);
        steel.density = Some(7850.0); // kg/m³
        materials.add_material(steel);

        // Assign material to element 1
        materials.assign_material(1, "STEEL".to_string());
        materials
    }

    #[test]
    fn assembles_mass_matrix() {
        // Test that mass matrix assembly completes successfully
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library_with_density();
        let bcs = BoundaryConditions::new();

        let area = 0.01;
        let mut system = GlobalSystem::assemble(&mesh, &materials, &bcs, area).unwrap();

        // Mass matrix should initially be None (not assembled by default)
        assert!(system.mass.is_none(), "Mass should not be assembled by default");

        // Assemble mass matrix
        let result = system.assemble_mass(&mesh, &materials, area, 3);
        assert!(result.is_ok(), "Mass assembly should succeed");
        assert!(system.mass.is_some(), "Mass matrix should now exist");
    }

    #[test]
    fn mass_matrix_is_symmetric() {
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library_with_density();
        let bcs = BoundaryConditions::new();

        let area = 0.01;
        let mut system = GlobalSystem::assemble(&mesh, &materials, &bcs, area).unwrap();
        system.assemble_mass(&mesh, &materials, area, 3).unwrap();

        let mass = system.mass.as_ref().unwrap();

        // Check symmetry
        for i in 0..mass.nrows() {
            for j in 0..mass.ncols() {
                let error = (mass[(i, j)] - mass[(j, i)]).abs();
                assert!(
                    error < 1e-10,
                    "Mass matrix not symmetric at ({}, {}): {} vs {}",
                    i,
                    j,
                    mass[(i, j)],
                    mass[(j, i)]
                );
            }
        }
    }

    #[test]
    fn mass_matrix_positive_semidefinite() {
        // Test that mass matrix has non-negative diagonal entries
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library_with_density();
        let bcs = BoundaryConditions::new();

        let area = 0.01;
        let mut system = GlobalSystem::assemble(&mesh, &materials, &bcs, area).unwrap();
        system.assemble_mass(&mesh, &materials, area, 3).unwrap();

        let mass = system.mass.as_ref().unwrap();

        // Check all diagonal entries are non-negative
        for i in 0..mass.nrows() {
            assert!(
                mass[(i, i)] >= 0.0,
                "Diagonal entry M[{}, {}] = {} should be non-negative",
                i,
                i,
                mass[(i, i)]
            );
        }

        // Check that some diagonal entries are positive (mesh has mass)
        let positive_count = (0..mass.nrows()).filter(|&i| mass[(i, i)] > 1e-12).count();
        assert!(
            positive_count > 0,
            "At least some DOFs should have positive mass"
        );
    }

    #[test]
    fn mass_matrix_dimensions() {
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library_with_density();
        let bcs = BoundaryConditions::new();

        let area = 0.01;
        let mut system = GlobalSystem::assemble(&mesh, &materials, &bcs, area).unwrap();
        system.assemble_mass(&mesh, &materials, area, 3).unwrap();

        let mass = system.mass.as_ref().unwrap();

        // Mass matrix should be square and match system DOFs
        assert_eq!(mass.nrows(), system.num_dofs);
        assert_eq!(mass.ncols(), system.num_dofs);
    }

    #[test]
    fn mass_requires_density() {
        // Test that mass assembly fails gracefully when density is missing
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library(); // No density
        let bcs = BoundaryConditions::new();

        let area = 0.01;
        let mut system = GlobalSystem::assemble(&mesh, &materials, &bcs, area).unwrap();

        // Mass assembly should fail due to missing density
        let result = system.assemble_mass(&mesh, &materials, area, 3);
        assert!(result.is_err(), "Mass assembly should fail without density");
        assert!(result.unwrap_err().contains("density"));
    }
}
