//! Finite element library for structural analysis.

use crate::materials::Material;
use crate::mesh::Node;
use nalgebra::DMatrix;

pub mod beam;
pub mod beam3;
pub mod factory;
pub mod shell;
pub mod solid;
pub mod truss;
pub mod truss3;

pub use beam::{Beam31, BeamSection};
pub use beam3::Beam32;
pub use factory::DynamicElement;
pub use shell::{S4, ShellSection};
pub use solid::C3D8;
pub use truss::Truss2D;
pub use truss3::Truss3D;

/// Element interface for finite element calculations
pub trait Element {
    /// Compute the element stiffness matrix in global coordinates
    ///
    /// # Arguments
    /// * `nodes` - Node coordinates for this element
    /// * `material` - Material properties
    ///
    /// # Returns
    /// Element stiffness matrix k_e (size: num_dofs × num_dofs)
    fn stiffness_matrix(&self, nodes: &[Node], material: &Material)
    -> Result<DMatrix<f64>, String>;

    /// Compute the element mass matrix in global coordinates
    ///
    /// # Arguments
    /// * `nodes` - Node coordinates for this element
    /// * `material` - Material properties (density required)
    ///
    /// # Returns
    /// Element mass matrix m_e (size: num_dofs × num_dofs)
    ///
    /// # Errors
    /// Returns error if material density is not provided
    fn mass_matrix(&self, nodes: &[Node], material: &Material)
    -> Result<DMatrix<f64>, String>;

    /// Get the number of nodes for this element type
    fn num_nodes(&self) -> usize;

    /// Get the number of degrees of freedom per node
    fn dofs_per_node(&self) -> usize;

    /// Get the global DOF indices for this element
    ///
    /// # Arguments
    /// * `connectivity` - Node IDs for this element
    ///
    /// # Returns
    /// Vector of global DOF indices
    fn global_dof_indices(&self, connectivity: &[i32]) -> Vec<usize> {
        let mut indices = Vec::with_capacity(self.num_nodes() * self.dofs_per_node());
        let dofs_per_node = self.dofs_per_node();

        for &node_id in connectivity {
            let base_dof = ((node_id - 1) as usize) * dofs_per_node;
            for local_dof in 0..dofs_per_node {
                indices.push(base_dof + local_dof);
            }
        }

        indices
    }
}

/// Element section properties (for beams, shells, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct SectionProperties {
    /// Cross-sectional area [m²]
    pub area: f64,
    /// Second moment of area about y-axis [m⁴]
    pub i_yy: Option<f64>,
    /// Second moment of area about z-axis [m⁴]
    pub i_zz: Option<f64>,
    /// Torsional constant [m⁴]
    pub i_t: Option<f64>,
}

impl SectionProperties {
    /// Create section properties for a truss element (area only)
    pub fn truss(area: f64) -> Self {
        Self {
            area,
            i_yy: None,
            i_zz: None,
            i_t: None,
        }
    }

    /// Create section properties for a beam element
    pub fn beam(area: f64, i_yy: f64, i_zz: f64, i_t: f64) -> Self {
        Self {
            area,
            i_yy: Some(i_yy),
            i_zz: Some(i_zz),
            i_t: Some(i_t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_dof_indices_simple() {
        struct DummyElement;
        impl Element for DummyElement {
            fn stiffness_matrix(
                &self,
                _nodes: &[Node],
                _material: &Material,
            ) -> Result<DMatrix<f64>, String> {
                Ok(DMatrix::zeros(6, 6))
            }
            fn mass_matrix(
                &self,
                _nodes: &[Node],
                _material: &Material,
            ) -> Result<DMatrix<f64>, String> {
                Ok(DMatrix::zeros(6, 6))
            }
            fn num_nodes(&self) -> usize {
                2
            }
            fn dofs_per_node(&self) -> usize {
                3
            }
        }

        let elem = DummyElement;
        let connectivity = vec![1, 2]; // nodes 1 and 2
        let indices = elem.global_dof_indices(&connectivity);

        // Node 1: DOFs 0, 1, 2
        // Node 2: DOFs 3, 4, 5
        assert_eq!(indices, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn global_dof_indices_offset() {
        struct DummyElement;
        impl Element for DummyElement {
            fn stiffness_matrix(
                &self,
                _nodes: &[Node],
                _material: &Material,
            ) -> Result<DMatrix<f64>, String> {
                Ok(DMatrix::zeros(6, 6))
            }
            fn mass_matrix(
                &self,
                _nodes: &[Node],
                _material: &Material,
            ) -> Result<DMatrix<f64>, String> {
                Ok(DMatrix::zeros(6, 6))
            }
            fn num_nodes(&self) -> usize {
                2
            }
            fn dofs_per_node(&self) -> usize {
                3
            }
        }

        let elem = DummyElement;
        let connectivity = vec![5, 10]; // nodes 5 and 10
        let indices = elem.global_dof_indices(&connectivity);

        // Node 5 (0-indexed: 4): DOFs 12, 13, 14
        // Node 10 (0-indexed: 9): DOFs 27, 28, 29
        assert_eq!(indices, vec![12, 13, 14, 27, 28, 29]);
    }
}
