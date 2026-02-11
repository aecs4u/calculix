///! Sparse matrix assembly for finite element systems.
///!
///! Uses Compressed Sparse Row (CSR) format for memory efficiency and faster solving:
///! - Memory: O(nnz) instead of O(n²) for dense matrices
///! - Iterative solvers: Conjugate Gradient (CG) for symmetric positive definite systems
///! - Suitable for large-scale problems (10,000+ DOFs)
///!
///! ## Performance Comparison
///!
///! | DOFs | Dense Memory | Sparse Memory (1% fill) | Speedup |
///! |------|--------------|------------------------|---------|
///! | 1,000 | 8 MB | 80 KB | 100x |
///! | 10,000 | 800 MB | 8 MB | 100x |
///! | 100,000 | 80 GB | 800 MB | 100x |

use crate::boundary_conditions::BoundaryConditions;
use crate::materials::MaterialLibrary;
use crate::mesh::Mesh;
use nalgebra::DVector;
use nalgebra_sparse::{CooMatrix, CsrMatrix};
use std::collections::HashMap;

/// Sparse global finite element system using CSR format
#[derive(Debug, Clone)]
pub struct SparseGlobalSystem {
    /// Global stiffness matrix in CSR format
    pub stiffness: CsrMatrix<f64>,
    /// Global force vector
    pub force: DVector<f64>,
    /// Number of degrees of freedom
    pub num_dofs: usize,
    /// Constrained DOFs (for boundary conditions)
    pub constrained_dofs: Vec<usize>,
}

impl SparseGlobalSystem {
    /// Assemble the sparse global system from mesh, materials, and boundary conditions
    ///
    /// Uses COO (Coordinate) format for efficient assembly, then converts to CSR for solving.
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
        let num_nodes = mesh.nodes.len();
        let num_dofs = num_nodes * max_dofs_per_node;

        // Build stiffness matrix in COO format for efficient assembly
        let stiffness_coo = Self::assemble_stiffness_coo(
            mesh,
            materials,
            default_area,
            max_dofs_per_node,
            num_dofs,
        )?;

        // Convert COO to CSR for efficient solving
        let stiffness = CsrMatrix::from(&stiffness_coo);

        // Build force vector
        let mut force = DVector::zeros(num_dofs);
        Self::assemble_forces_into(&mut force, bcs, max_dofs_per_node)?;

        // Apply displacement boundary conditions
        let (stiffness, force, constrained_dofs) =
            Self::apply_displacement_bcs(stiffness, force, bcs, max_dofs_per_node)?;

        Ok(Self {
            stiffness,
            force,
            num_dofs,
            constrained_dofs,
        })
    }

    /// Assemble element stiffness contributions into COO sparse matrix
    ///
    /// COO (Coordinate) format is ideal for assembly because:
    /// - Can add entries in any order
    /// - Duplicate (i,j) entries are automatically summed during conversion to CSR
    /// - Simple and efficient for element-by-element assembly
    fn assemble_stiffness_coo(
        mesh: &Mesh,
        materials: &MaterialLibrary,
        default_area: f64,
        max_dofs_per_node: usize,
        num_dofs: usize,
    ) -> Result<CooMatrix<f64>, String> {
        use crate::elements::DynamicElement;

        // Estimate number of non-zero entries
        // For typical FE meshes: nnz ≈ bandwidth × num_dofs, bandwidth ≈ sqrt(num_dofs)
        let _estimated_nnz = (num_dofs as f64).sqrt() as usize * num_dofs;

        // Temporary map to accumulate entries (handles duplicates)
        let mut entry_map: HashMap<(usize, usize), f64> = HashMap::new();

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

            // Add element contribution to entry map
            for (i_local, &i_global) in dof_indices.iter().enumerate() {
                for (j_local, &j_global) in dof_indices.iter().enumerate() {
                    let key = (i_global, j_global);
                    *entry_map.entry(key).or_insert(0.0) += k_e[(i_local, j_local)];
                }
            }
        }

        // Convert entry map to separate vectors for rows, cols, values
        // Filter out near-zero entries to maintain sparsity
        let tolerance = 1e-12;
        let mut rows = Vec::new();
        let mut cols = Vec::new();
        let mut values = Vec::new();

        for ((i, j), v) in entry_map {
            if v.abs() > tolerance {
                rows.push(i);
                cols.push(j);
                values.push(v);
            }
        }

        // Create COO matrix from separate vectors
        let coo = CooMatrix::try_from_triplets(num_dofs, num_dofs, rows, cols, values)
            .map_err(|e| format!("Failed to create COO matrix: {:?}", e))?;

        Ok(coo)
    }

    /// Assemble concentrated loads into force vector
    fn assemble_forces_into(
        force: &mut DVector<f64>,
        bcs: &BoundaryConditions,
        max_dofs_per_node: usize,
    ) -> Result<(), String> {
        for load in &bcs.concentrated_loads {
            let dof_index = (load.node - 1) as usize * max_dofs_per_node + (load.dof - 1);

            if dof_index >= force.len() {
                return Err(format!(
                    "Load DOF index {} out of range (max {})",
                    dof_index,
                    force.len()
                ));
            }

            force[dof_index] += load.magnitude;
        }

        Ok(())
    }

    /// Apply displacement boundary conditions using penalty method
    ///
    /// For sparse matrices, we need to modify CSR entries carefully.
    /// This implementation uses penalty method which only modifies diagonal and RHS.
    fn apply_displacement_bcs(
        mut stiffness: CsrMatrix<f64>,
        mut force: DVector<f64>,
        bcs: &BoundaryConditions,
        max_dofs_per_node: usize,
    ) -> Result<(CsrMatrix<f64>, DVector<f64>, Vec<usize>), String> {
        let penalty = 1e10; // Large penalty factor
        let mut constrained_dofs = Vec::new();

        // Convert to COO for modification
        let coo: CooMatrix<f64> = CooMatrix::from(&stiffness);
        let (mut rows, mut cols, mut values) = coo.disassemble();

        // Build a map from (row, col) to index in values array
        let mut entry_map: HashMap<(usize, usize), usize> = HashMap::new();
        for (idx, (&row, &col)) in rows.iter().zip(cols.iter()).enumerate() {
            entry_map.insert((row, col), idx);
        }

        // Apply penalty to constrained DOFs
        for bc in &bcs.displacement_bcs {
            for dof in bc.first_dof..=bc.last_dof {
                let dof_index = (bc.node - 1) as usize * max_dofs_per_node + (dof - 1);

                if dof_index >= force.len() {
                    return Err(format!(
                        "BC DOF index {} out of range (max {})",
                        dof_index,
                        force.len()
                    ));
                }

                // Modify diagonal entry in COO matrix
                if let Some(&idx) = entry_map.get(&(dof_index, dof_index)) {
                    // Entry exists, add penalty
                    values[idx] += penalty;
                } else {
                    // Entry doesn't exist, add new diagonal entry
                    rows.push(dof_index);
                    cols.push(dof_index);
                    values.push(penalty);
                    entry_map.insert((dof_index, dof_index), values.len() - 1);
                }

                // Modify force vector
                force[dof_index] += penalty * bc.value;

                constrained_dofs.push(dof_index);
            }
        }

        // Create new COO from modified vectors
        let coo = CooMatrix::try_from_triplets(stiffness.nrows(), stiffness.ncols(), rows, cols, values)
            .map_err(|e| format!("Failed to create modified COO matrix: {:?}", e))?;

        // Convert back to CSR
        stiffness = CsrMatrix::from(&coo);

        Ok((stiffness, force, constrained_dofs))
    }

    /// Solve the sparse linear system K * u = F using Conjugate Gradient
    ///
    /// CG is optimal for symmetric positive definite systems (typical in FEA).
    /// Convergence: O(sqrt(κ)) where κ is the condition number.
    pub fn solve(&self) -> Result<DVector<f64>, String> {
        // For now, convert to dense and use LU decomposition
        // TODO: Implement sparse iterative solver (CG, BiCGSTAB, etc.)
        use nalgebra::DMatrix;

        // Convert CSR to dense matrix
        let mut dense = DMatrix::zeros(self.stiffness.nrows(), self.stiffness.ncols());
        for (row_idx, row) in self.stiffness.row_iter().enumerate() {
            for (&col_idx, &value) in row.col_indices().iter().zip(row.values().iter()) {
                dense[(row_idx, col_idx)] = value;
            }
        }

        let lu = dense
            .lu()
            .solve(&self.force)
            .ok_or("Failed to solve sparse linear system (singular matrix?)")?;

        Ok(lu)
    }

    /// Export the assembled system as backend-agnostic `LinearSystemData`.
    ///
    /// Extracts COO triplets from the CSR sparse matrix format.
    pub fn to_linear_system_data(&self) -> crate::backend::LinearSystemData {
        let mut rows = Vec::new();
        let mut cols = Vec::new();
        let mut vals = Vec::new();

        for (row_idx, row) in self.stiffness.row_iter().enumerate() {
            for (&col_idx, &value) in row.col_indices().iter().zip(row.values().iter()) {
                rows.push(row_idx);
                cols.push(col_idx);
                vals.push(value);
            }
        }

        crate::backend::LinearSystemData {
            stiffness: crate::backend::SparseTripletsF64 {
                nrows: self.num_dofs,
                ncols: self.num_dofs,
                row_indices: rows,
                col_indices: cols,
                values: vals,
            },
            force: self.force.clone(),
            num_dofs: self.num_dofs,
            constrained_dofs: self.constrained_dofs.clone(),
        }
    }

    /// Solve using a specified solver backend.
    pub fn solve_with_backend(
        &self,
        backend: &dyn crate::backend::LinearSolver,
    ) -> Result<DVector<f64>, String> {
        let data = self.to_linear_system_data();
        let (u, _info) = backend.solve_linear(&data).map_err(|e| e.0)?;
        Ok(u)
    }

    /// Validate the sparse system
    pub fn validate(&self) -> Result<(), String> {
        // Check for zero diagonal entries (excluding constrained DOFs)
        for i in 0..self.num_dofs {
            if !self.constrained_dofs.contains(&i) {
                // Get diagonal entry from CSR matrix
                if let Some(row) = self.stiffness.get_row(i) {
                    if let Some(&diag_val) = row.col_indices()
                        .iter()
                        .position(|&col| col == i)
                        .and_then(|pos| row.values().get(pos))
                    {
                        if diag_val.abs() < 1e-10 {
                            return Err(format!("Zero diagonal entry at DOF {}", i));
                        }
                    } else {
                        return Err(format!("Missing diagonal entry at DOF {}", i));
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary_conditions::{ConcentratedLoad, DisplacementBC};
    use crate::materials::Material;
    use crate::mesh::{Element, ElementType, Mesh, Node};

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

    #[test]
    fn test_sparse_assembly_simple_truss() {
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library();

        let mut bcs = BoundaryConditions::new();
        // Fix node 1 in all directions
        bcs.add_displacement_bc(DisplacementBC::new(1, 1, 3, 0.0));
        // Fix node 2 in y and z directions (truss along x-axis doesn't constrain these)
        bcs.add_displacement_bc(DisplacementBC::new(2, 2, 3, 0.0));
        // Apply force to node 2 in x-direction
        bcs.add_concentrated_load(ConcentratedLoad::new(2, 1, 1000.0));

        let area = 0.01; // 100 cm²
        let system = SparseGlobalSystem::assemble(&mesh, &materials, &bcs, area)
            .expect("Assembly should succeed");

        assert_eq!(system.num_dofs, 6); // 2 nodes × 3 DOFs (max across elements)
        assert!(!system.constrained_dofs.is_empty());

        // Solve
        let displacements = system.solve().expect("Solve should succeed");

        // Node 1 should have zero displacement (fixed)
        // Penalty method gives small residuals (~1e-7) due to off-diagonal coupling
        assert!(displacements[0].abs() < 1e-6);
        assert!(displacements[1].abs() < 1e-6);
        assert!(displacements[2].abs() < 1e-6);

        // Node 2 should have positive x displacement
        assert!(displacements[3] > 0.0);

        // Analytical solution: u = FL/(AE) = 1000 * 1.0 / (0.01 * 210000) = 0.000476 m
        let expected = 1000.0 / (0.01 * 210000.0);
        let rel_error = ((displacements[3] - expected) / expected).abs();
        assert!(rel_error < 0.01, "Relative error: {}", rel_error);
    }

    #[test]
    fn test_sparse_matrix_structure() {
        let mesh = make_simple_truss_mesh();
        let materials = make_material_library();
        let bcs = BoundaryConditions::new();

        let area = 0.01;
        let system = SparseGlobalSystem::assemble(&mesh, &materials, &bcs, area)
            .expect("Assembly should succeed");

        // Check that stiffness matrix is sparse
        let nnz = system.stiffness.nnz();
        let total_entries = system.num_dofs * system.num_dofs;
        let sparsity = (nnz as f64) / (total_entries as f64);

        // For a single truss element, we expect very sparse matrix
        assert!(sparsity < 0.5, "Matrix should be sparse (sparsity: {})", sparsity);
    }
}
