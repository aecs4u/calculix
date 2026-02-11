//! Distributed loads and conversion to equivalent nodal forces.
//!
//! This module provides functionality to convert distributed loads (pressure, traction, body forces)
//! into equivalent nodal forces through numerical integration.
//!
//! # Workflow
//! 1. User defines DistributedLoad (element ID or set name + load type + magnitude)
//! 2. DistributedLoadConverter resolves which elements are affected
//! 3. Each element computes equivalent nodal forces via numerical integration
//! 4. Assembly system accumulates nodal forces into global force vector

use crate::boundary_conditions::{DistributedLoad, DistributedLoadType};
use crate::elements::factory::DynamicElement;
use crate::materials::MaterialLibrary;
use crate::mesh::{Element as MeshElement, ElementType, Mesh, Node};
use nalgebra::SVector;
use std::collections::HashMap;

/// Type alias for 6-DOF force/moment vector [Fx, Fy, Fz, Mx, My, Mz]
pub type Vector6 = SVector<f64, 6>;

/// Converter for transforming distributed loads into equivalent nodal forces
pub struct DistributedLoadConverter<'a> {
    mesh: &'a Mesh,
    materials: &'a MaterialLibrary,
}

impl<'a> DistributedLoadConverter<'a> {
    /// Create a new distributed load converter
    ///
    /// # Arguments
    /// * `mesh` - The finite element mesh
    /// * `materials` - Material library for property lookup
    pub fn new(mesh: &'a Mesh, materials: &'a MaterialLibrary) -> Self {
        Self { mesh, materials }
    }

    /// Convert distributed load to equivalent nodal forces
    ///
    /// # Arguments
    /// * `load` - The distributed load specification
    ///
    /// # Returns
    /// HashMap mapping node IDs to force/moment vectors (6 DOFs per node)
    ///
    /// # Errors
    /// Returns error if:
    /// - Element/set not found
    /// - Element type doesn't support the load type
    /// - Load parameters are invalid
    pub fn convert_to_nodal_forces(
        &self,
        load: &DistributedLoad,
    ) -> Result<HashMap<i32, Vector6>, String> {
        // Step 1: Resolve which elements are affected
        let element_ids = self.resolve_elements(&load.element)?;

        // Step 2: Accumulate nodal forces from all elements
        let mut nodal_forces: HashMap<i32, Vector6> = HashMap::new();

        for elem_id in element_ids {
            // Get element from mesh
            let mesh_elem = self
                .mesh
                .elements
                .get(&elem_id)
                .ok_or_else(|| format!("Element {} not found in mesh", elem_id))?;

            // Compute nodal forces for this element
            let elem_nodal_forces = self.element_nodal_forces(mesh_elem, load)?;

            // Accumulate into global nodal forces map
            for (node_id, force) in elem_nodal_forces {
                nodal_forces
                    .entry(node_id)
                    .and_modify(|f| *f += force)
                    .or_insert(force);
            }
        }

        Ok(nodal_forces)
    }

    /// Resolve element IDs from element specification string
    ///
    /// # Arguments
    /// * `element_spec` - Numeric element ID (e.g., "123")
    ///
    /// # Returns
    /// Vector of element IDs
    ///
    /// # Errors
    /// Returns error if element not found
    ///
    /// # Note
    /// Currently only supports single element IDs. Element set support will be added later.
    fn resolve_elements(&self, element_spec: &str) -> Result<Vec<i32>, String> {
        // Parse as numeric element ID
        let elem_id = element_spec.parse::<i32>().map_err(|_| {
            format!(
                "Element specification '{}' is not a valid numeric element ID",
                element_spec
            )
        })?;

        // Check if element exists
        if self.mesh.elements.contains_key(&elem_id) {
            Ok(vec![elem_id])
        } else {
            Err(format!("Element {} not found in mesh", elem_id))
        }
    }

    /// Compute nodal forces for a single element
    ///
    /// # Arguments
    /// * `elem` - Mesh element
    /// * `load` - Distributed load specification
    ///
    /// # Returns
    /// HashMap mapping node IDs to force/moment vectors
    fn element_nodal_forces(
        &self,
        elem: &MeshElement,
        load: &DistributedLoad,
    ) -> Result<HashMap<i32, Vector6>, String> {
        // Dispatch based on element type and load type
        match (elem.element_type, load.load_type) {
            (ElementType::S4, DistributedLoadType::Pressure) => {
                self.shell_pressure_forces(elem, load.magnitude)
            }
            (elem_type, load_type) => Err(format!(
                "Distributed load type {:?} not supported for element type {:?}",
                load_type, elem_type
            )),
        }
    }

    /// Compute pressure load nodal forces for shell element
    ///
    /// # Arguments
    /// * `elem` - Shell mesh element
    /// * `pressure` - Pressure magnitude (Pa, positive = compression)
    ///
    /// # Returns
    /// HashMap mapping node IDs to force vectors
    fn shell_pressure_forces(
        &self,
        elem: &MeshElement,
        pressure: f64,
    ) -> Result<HashMap<i32, Vector6>, String> {
        // Create DynamicElement for accessing pressure_to_nodal_forces() method
        let dynamic_elem = DynamicElement::from_mesh_element(
            elem.element_type,
            elem.id,
            elem.nodes.clone(),
            0.01, // Default thickness (unused for pressure calculation)
        )
        .ok_or_else(|| {
            format!(
                "Failed to create dynamic element for element {}",
                elem.id
            )
        })?;

        // Get S4 element variant
        let shell = match dynamic_elem {
            DynamicElement::Shell4(s) => s,
            _ => {
                return Err(format!(
                    "Expected Shell4 element, got {:?}",
                    dynamic_elem.element_type()
                ))
            }
        };

        // Get node coordinates
        let nodes: Vec<Node> = elem
            .nodes
            .iter()
            .map(|&node_id| {
                self.mesh
                    .nodes
                    .get(&node_id)
                    .ok_or_else(|| format!("Node {} not found", node_id))
                    .map(|n| n.clone())
            })
            .collect::<Result<Vec<Node>, String>>()?;

        // Compute nodal forces using element method
        let nodal_forces_array = shell.pressure_to_nodal_forces(&nodes, pressure)?;

        // Convert array to HashMap
        let mut nodal_forces = HashMap::new();
        for (i, &node_id) in elem.nodes.iter().enumerate() {
            nodal_forces.insert(node_id, nodal_forces_array[i]);
        }

        Ok(nodal_forces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary_conditions::DistributedLoad;
    use crate::materials::{Material, MaterialModel};

    fn steel_material() -> Material {
        Material {
            name: "Steel".to_string(),
            model: MaterialModel::LinearElastic,
            elastic_modulus: Some(200e9),
            poissons_ratio: Some(0.3),
            density: Some(7850.0),
            thermal_expansion: None,
            conductivity: None,
            specific_heat: None,
        }
    }

    fn make_single_plate_mesh() -> Mesh {
        let mut mesh = Mesh::new();

        // 4 nodes in XY plane (1×1 meter plate)
        mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
        mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));
        mesh.add_node(Node::new(3, 1.0, 1.0, 0.0));
        mesh.add_node(Node::new(4, 0.0, 1.0, 0.0));

        // Single S4 element
        let _ = mesh.add_element(MeshElement::new(1, ElementType::S4, vec![1, 2, 3, 4]));

        mesh
    }

    #[test]
    fn resolves_element_by_id() {
        let mesh = make_single_plate_mesh();
        let materials = MaterialLibrary::new();
        let converter = DistributedLoadConverter::new(&mesh, &materials);

        let result = converter.resolve_elements("1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1]);
    }

    #[test]
    fn error_on_invalid_element() {
        let mesh = make_single_plate_mesh();
        let materials = MaterialLibrary::new();
        let converter = DistributedLoadConverter::new(&mesh, &materials);

        let result = converter.resolve_elements("999");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn error_on_non_numeric_spec() {
        let mesh = make_single_plate_mesh();
        let materials = MaterialLibrary::new();
        let converter = DistributedLoadConverter::new(&mesh, &materials);

        let result = converter.resolve_elements("plate_top");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a valid numeric element ID"));
    }

    #[test]
    fn converts_pressure_to_nodal_forces() {
        let mesh = make_single_plate_mesh();
        let mut materials = MaterialLibrary::new();
        materials.add_material(steel_material());

        let converter = DistributedLoadConverter::new(&mesh, &materials);

        let load = DistributedLoad {
            element: "1".to_string(),
            load_type: DistributedLoadType::Pressure,
            magnitude: 1000.0, // 1000 Pa
            parameters: vec![],
        };

        let result = converter.convert_to_nodal_forces(&load);
        assert!(result.is_ok(), "Conversion should succeed");

        let nodal_forces = result.unwrap();
        assert_eq!(nodal_forces.len(), 4, "Should have forces at 4 nodes");

        // Check that all nodes have forces
        for node_id in [1, 2, 3, 4] {
            assert!(
                nodal_forces.contains_key(&node_id),
                "Node {} should have force",
                node_id
            );
        }
    }
}
