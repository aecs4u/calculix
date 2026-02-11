/// Element factory for creating element instances from mesh data
///
/// This module provides factory functions to create appropriate element implementations
/// based on element type, handling the conversion from mesh::Element to typed elements.

use crate::elements::{Beam31, Beam32, BeamSection, C3D8, Element, S4, ShellSection, Truss2D, Truss3D};
use crate::materials::Material;
use crate::mesh::{ElementType, Node};
use nalgebra::DMatrix;

/// Dynamic element wrapper that can hold any element type
///
/// This allows us to work with different element types polymorphically
/// during assembly without knowing the concrete type at compile time.
pub enum DynamicElement {
    Truss(Truss2D),
    Truss3(Truss3D),
    Beam(Beam31),
    Beam3(Beam32),
    Shell4(S4),
    Solid8(C3D8),
}

impl DynamicElement {
    /// Create a dynamic element from mesh element data
    ///
    /// # Arguments
    /// * `elem_type` - The element type from the mesh
    /// * `elem_id` - Element ID
    /// * `nodes` - Node connectivity
    /// * `default_area` - Default cross-sectional area (for truss/beam) or thickness (for shell)
    ///
    /// # Returns
    /// A dynamic element wrapper, or None if the element type is not yet supported
    pub fn from_mesh_element(
        elem_type: ElementType,
        elem_id: i32,
        nodes: Vec<i32>,
        default_area: f64,
    ) -> Option<Self> {
        match elem_type {
            ElementType::T3D2 => {
                let truss = Truss2D::new(elem_id, nodes, default_area);
                Some(DynamicElement::Truss(truss))
            }
            ElementType::T3D3 => {
                if nodes.len() != 3 {
                    return None;
                }
                let node_array: [i32; 3] = nodes.try_into().ok()?;
                let truss3 = Truss3D::new(elem_id, node_array, default_area);
                Some(DynamicElement::Truss3(truss3))
            }
            ElementType::B31 => {
                // For now, use circular section with area-equivalent radius
                let radius = (default_area / std::f64::consts::PI).sqrt();
                let section = BeamSection::circular(radius);
                let beam = Beam31::new(elem_id, nodes[0], nodes[1], section);
                Some(DynamicElement::Beam(beam))
            }
            ElementType::B32 => {
                if nodes.len() != 3 {
                    return None;
                }
                let radius = (default_area / std::f64::consts::PI).sqrt();
                let section = BeamSection::circular(radius);
                let node_array: [i32; 3] = nodes.try_into().ok()?;
                let beam3 = Beam32::new(elem_id, node_array, section);
                Some(DynamicElement::Beam3(beam3))
            }
            ElementType::S4 => {
                // For shells, default_area is interpreted as thickness
                let thickness = if default_area < 0.001 { 0.01 } else { default_area };
                let section = ShellSection::new(thickness);
                let shell = S4::new(elem_id, nodes, section);
                Some(DynamicElement::Shell4(shell))
            }
            ElementType::C3D8 => {
                if nodes.len() != 8 {
                    return None;
                }
                let node_array: [i32; 8] = nodes.try_into().ok()?;
                Some(DynamicElement::Solid8(C3D8::new(elem_id, node_array)))
            }
            _ => None, // Unsupported element type
        }
    }

    /// Compute stiffness matrix for this element
    pub fn stiffness_matrix(
        &self,
        nodes: &[Node],
        material: &Material,
    ) -> Result<DMatrix<f64>, String> {
        match self {
            DynamicElement::Truss(truss) => truss.stiffness_matrix(nodes, material),
            DynamicElement::Truss3(truss3) => truss3.stiffness_matrix(nodes, material),
            DynamicElement::Beam(beam) => beam.stiffness_matrix(nodes, material),
            DynamicElement::Beam3(beam3) => beam3.stiffness_matrix(nodes, material),
            DynamicElement::Shell4(shell) => shell.stiffness_matrix(nodes, material),
            DynamicElement::Solid8(solid) => solid.stiffness_matrix(nodes, material),
        }
    }

    /// Compute mass matrix for this element
    pub fn mass_matrix(
        &self,
        nodes: &[Node],
        material: &Material,
    ) -> Result<DMatrix<f64>, String> {
        match self {
            DynamicElement::Truss(truss) => truss.mass_matrix(nodes, material),
            DynamicElement::Truss3(truss3) => truss3.mass_matrix(nodes, material),
            DynamicElement::Beam(beam) => beam.mass_matrix(nodes, material),
            DynamicElement::Beam3(beam3) => beam3.mass_matrix(nodes, material),
            DynamicElement::Shell4(shell) => shell.mass_matrix(nodes, material),
            DynamicElement::Solid8(solid) => solid.mass_matrix(nodes, material),
        }
    }

    /// Get global DOF indices for this element
    ///
    /// # Arguments
    /// * `connectivity` - Node IDs for this element
    /// * `max_dofs_per_node` - Maximum DOFs per node in the global system
    ///
    /// # Returns
    /// Vector of global DOF indices for this element
    pub fn global_dof_indices(&self, connectivity: &[i32], max_dofs_per_node: usize) -> Vec<usize> {
        let dofs_per_node = match self {
            DynamicElement::Truss(t) => t.dofs_per_node(),
            DynamicElement::Truss3(t3) => t3.dofs_per_node(),
            DynamicElement::Beam(b) => b.dofs_per_node(),
            DynamicElement::Beam3(b3) => b3.dofs_per_node(),
            DynamicElement::Shell4(s) => s.dofs_per_node(),
            DynamicElement::Solid8(c) => c.dofs_per_node(),
        };

        let mut indices = Vec::new();
        for &node_id in connectivity {
            let base_dof = ((node_id - 1) as usize) * max_dofs_per_node;
            for local_dof in 0..dofs_per_node {
                indices.push(base_dof + local_dof);
            }
        }
        indices
    }

    /// Get the element type
    pub fn element_type(&self) -> ElementType {
        match self {
            DynamicElement::Truss(_) => ElementType::T3D2,
            DynamicElement::Truss3(_) => ElementType::T3D3,
            DynamicElement::Beam(_) => ElementType::B31,
            DynamicElement::Beam3(_) => ElementType::B32,
            DynamicElement::Shell4(_) => ElementType::S4,
            DynamicElement::Solid8(_) => ElementType::C3D8,
        }
    }

    /// Get number of DOFs for this element
    pub fn num_dofs(&self) -> usize {
        match self {
            DynamicElement::Truss(truss) => truss.num_nodes() * truss.dofs_per_node(),
            DynamicElement::Truss3(truss3) => truss3.num_nodes() * truss3.dofs_per_node(),
            DynamicElement::Beam(beam) => beam.num_nodes() * beam.dofs_per_node(),
            DynamicElement::Beam3(beam3) => beam3.num_nodes() * beam3.dofs_per_node(),
            DynamicElement::Shell4(shell) => shell.num_nodes() * shell.dofs_per_node(),
            DynamicElement::Solid8(solid) => solid.num_nodes() * solid.dofs_per_node(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_truss_element() {
        let elem = DynamicElement::from_mesh_element(
            ElementType::T3D2,
            1,
            vec![0, 1],
            0.01,
        );

        assert!(elem.is_some());
        let elem = elem.unwrap();
        assert_eq!(elem.element_type(), ElementType::T3D2);
        assert_eq!(elem.num_dofs(), 6); // 2 nodes × 3 DOFs
    }

    #[test]
    fn test_create_beam_element() {
        let elem = DynamicElement::from_mesh_element(
            ElementType::B31,
            1,
            vec![0, 1],
            0.01,
        );

        assert!(elem.is_some());
        let elem = elem.unwrap();
        assert_eq!(elem.element_type(), ElementType::B31);
        assert_eq!(elem.num_dofs(), 12); // 2 nodes × 6 DOFs
    }

    #[test]
    fn test_create_shell_element() {
        let elem = DynamicElement::from_mesh_element(
            ElementType::S4,
            1,
            vec![1, 2, 3, 4],
            0.01, // thickness
        );

        assert!(elem.is_some());
        let elem = elem.unwrap();
        assert_eq!(elem.element_type(), ElementType::S4);
        assert_eq!(elem.num_dofs(), 24); // 4 nodes × 6 DOFs
    }

    #[test]
    fn test_unsupported_element_type() {
        // C3D20 (20-node brick) is not yet supported
        let elem = DynamicElement::from_mesh_element(
            ElementType::C3D20,
            1,
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19],
            0.01,
        );

        assert!(elem.is_none());
    }

    #[test]
    fn test_mass_matrix_dispatch() {
        // Test that mass_matrix() dispatch works for all element types
        use crate::materials::{Material, MaterialModel};

        let material = Material {
            name: "STEEL".to_string(),
            model: MaterialModel::LinearElastic,
            elastic_modulus: Some(200e9),
            poissons_ratio: Some(0.3),
            density: Some(7850.0), // kg/m³
            thermal_expansion: None,
            conductivity: None,
            specific_heat: None,
        };

        // Test Truss
        let truss_elem = DynamicElement::from_mesh_element(
            ElementType::T3D2,
            1,
            vec![1, 2],
            0.01,
        ).unwrap();
        let truss_nodes = vec![
            Node::new(1, 0.0, 0.0, 0.0),
            Node::new(2, 1.0, 0.0, 0.0),
        ];
        let m_truss = truss_elem.mass_matrix(&truss_nodes, &material);
        assert!(m_truss.is_ok(), "Truss mass matrix should succeed");
        let m = m_truss.unwrap();
        assert_eq!(m.nrows(), 6);
        assert_eq!(m.ncols(), 6);

        // Test Beam
        let beam_elem = DynamicElement::from_mesh_element(
            ElementType::B31,
            1,
            vec![1, 2],
            0.01,
        ).unwrap();
        let beam_nodes = vec![
            Node::new(1, 0.0, 0.0, 0.0),
            Node::new(2, 1.0, 0.0, 0.0),
        ];
        let m_beam = beam_elem.mass_matrix(&beam_nodes, &material);
        assert!(m_beam.is_ok(), "Beam mass matrix should succeed");
        let m = m_beam.unwrap();
        assert_eq!(m.nrows(), 12);
        assert_eq!(m.ncols(), 12);

        // Test Shell
        let shell_elem = DynamicElement::from_mesh_element(
            ElementType::S4,
            1,
            vec![1, 2, 3, 4],
            0.01, // thickness
        ).unwrap();
        let shell_nodes = vec![
            Node::new(1, 0.0, 0.0, 0.0),
            Node::new(2, 1.0, 0.0, 0.0),
            Node::new(3, 1.0, 1.0, 0.0),
            Node::new(4, 0.0, 1.0, 0.0),
        ];
        let m_shell = shell_elem.mass_matrix(&shell_nodes, &material);
        assert!(m_shell.is_ok(), "Shell mass matrix should succeed");
        let m = m_shell.unwrap();
        assert_eq!(m.nrows(), 24);
        assert_eq!(m.ncols(), 24);
    }
}
