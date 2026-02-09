//! 2-node truss element (T3D2) for tension/compression analysis.
//!
//! The truss element is a 1D element that resists only axial forces.
//! It has 2 nodes with 3 DOFs per node (translations in x, y, z).
//!
//! ## Element Formulation
//!
//! Local coordinate system:
//! - x-axis: along element axis (node 1 → node 2)
//! - Element only has axial stiffness
//!
//! Local stiffness matrix (1D):
//! ```text
//! k_local = (A*E/L) * [ 1  -1]
//!                      [-1   1]
//! ```
//!
//! Global stiffness matrix (3D):
//! ```text
//! k_global = T^T * k_local * T
//! ```
//!
//! where T is the transformation matrix from local to global coordinates.

use crate::elements::{Element, SectionProperties};
use crate::materials::Material;
use crate::mesh::Node;
use nalgebra::DMatrix;

/// 2-node truss element (T3D2)
#[derive(Debug, Clone)]
pub struct Truss2D {
    /// Element ID
    pub id: i32,
    /// Node connectivity [node1_id, node2_id]
    pub nodes: Vec<i32>,
    /// Section properties (cross-sectional area)
    pub section: SectionProperties,
}

impl Truss2D {
    /// Create a new truss element
    pub fn new(id: i32, nodes: Vec<i32>, area: f64) -> Self {
        assert_eq!(nodes.len(), 2, "Truss element must have 2 nodes");
        Self {
            id,
            nodes,
            section: SectionProperties::truss(area),
        }
    }

    /// Compute element length
    fn length(&self, nodes: &[Node]) -> Result<f64, String> {
        if nodes.len() != 2 {
            return Err(format!("Expected 2 nodes, got {}", nodes.len()));
        }

        let dx = nodes[1].x - nodes[0].x;
        let dy = nodes[1].y - nodes[0].y;
        let dz = nodes[1].z - nodes[0].z;

        let length = (dx * dx + dy * dy + dz * dz).sqrt();

        if length < 1e-10 {
            return Err(format!(
                "Truss element {} has zero or near-zero length: {}",
                self.id, length
            ));
        }

        Ok(length)
    }

    /// Compute direction cosines (unit vector from node 1 to node 2)
    fn direction_cosines(&self, nodes: &[Node]) -> Result<[f64; 3], String> {
        let length = self.length(nodes)?;

        let dx = nodes[1].x - nodes[0].x;
        let dy = nodes[1].y - nodes[0].y;
        let dz = nodes[1].z - nodes[0].z;

        Ok([dx / length, dy / length, dz / length])
    }

    /// Build transformation matrix from local to global coordinates
    ///
    /// The transformation matrix T relates local DOFs to global DOFs:
    /// ```text
    /// T = [l  m  n  0  0  0]
    ///     [0  0  0  l  m  n]
    /// ```
    /// where (l, m, n) are direction cosines.
    fn transformation_matrix(&self, nodes: &[Node]) -> Result<DMatrix<f64>, String> {
        let [l, m, n] = self.direction_cosines(nodes)?;

        let mut t = DMatrix::zeros(2, 6);
        t[(0, 0)] = l;
        t[(0, 1)] = m;
        t[(0, 2)] = n;
        t[(1, 3)] = l;
        t[(1, 4)] = m;
        t[(1, 5)] = n;

        Ok(t)
    }

    /// Compute local stiffness matrix (2×2)
    fn local_stiffness(&self, length: f64, material: &Material) -> Result<DMatrix<f64>, String> {
        let e = material
            .elastic_modulus
            .ok_or("Material missing elastic modulus")?;
        let a = self.section.area;

        let k = (a * e) / length;

        let mut k_local = DMatrix::zeros(2, 2);
        k_local[(0, 0)] = k;
        k_local[(0, 1)] = -k;
        k_local[(1, 0)] = -k;
        k_local[(1, 1)] = k;

        Ok(k_local)
    }
}

impl Element for Truss2D {
    fn stiffness_matrix(
        &self,
        nodes: &[Node],
        material: &Material,
    ) -> Result<DMatrix<f64>, String> {
        if nodes.len() != 2 {
            return Err(format!(
                "Truss element {} requires 2 nodes, got {}",
                self.id,
                nodes.len()
            ));
        }

        // Compute element length
        let length = self.length(nodes)?;

        // Get local stiffness matrix
        let k_local = self.local_stiffness(length, material)?;

        // Get transformation matrix
        let t = self.transformation_matrix(nodes)?;

        // Transform to global coordinates: k_global = T^T * k_local * T
        let k_global = t.transpose() * k_local * t;

        Ok(k_global)
    }

    fn num_nodes(&self) -> usize {
        2
    }

    fn dofs_per_node(&self) -> usize {
        3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_material() -> Material {
        let mut mat = Material::new("STEEL".to_string());
        mat.elastic_modulus = Some(210000.0); // MPa
        mat.poissons_ratio = Some(0.3);
        mat
    }

    #[test]
    fn creates_truss_element() {
        let elem = Truss2D::new(1, vec![1, 2], 0.01);
        assert_eq!(elem.id, 1);
        assert_eq!(elem.nodes, vec![1, 2]);
        assert_eq!(elem.section.area, 0.01);
    }

    #[test]
    #[should_panic(expected = "must have 2 nodes")]
    fn rejects_wrong_node_count() {
        Truss2D::new(1, vec![1, 2, 3], 0.01);
    }

    #[test]
    fn computes_length_horizontal() {
        let elem = Truss2D::new(1, vec![1, 2], 0.01);
        let nodes = vec![Node::new(1, 0.0, 0.0, 0.0), Node::new(2, 1.0, 0.0, 0.0)];

        let length = elem.length(&nodes).unwrap();
        assert!((length - 1.0).abs() < 1e-10);
    }

    #[test]
    fn computes_length_diagonal() {
        let elem = Truss2D::new(1, vec![1, 2], 0.01);
        let nodes = vec![Node::new(1, 0.0, 0.0, 0.0), Node::new(2, 3.0, 4.0, 0.0)];

        let length = elem.length(&nodes).unwrap();
        assert!((length - 5.0).abs() < 1e-10);
    }

    #[test]
    fn computes_length_3d() {
        let elem = Truss2D::new(1, vec![1, 2], 0.01);
        let nodes = vec![Node::new(1, 0.0, 0.0, 0.0), Node::new(2, 2.0, 3.0, 6.0)];

        let length = elem.length(&nodes).unwrap();
        // sqrt(2^2 + 3^2 + 6^2) = sqrt(49) = 7
        assert!((length - 7.0).abs() < 1e-10);
    }

    #[test]
    fn rejects_zero_length() {
        let elem = Truss2D::new(1, vec![1, 2], 0.01);
        let nodes = vec![Node::new(1, 1.0, 2.0, 3.0), Node::new(2, 1.0, 2.0, 3.0)];

        let result = elem.length(&nodes);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("zero"));
    }

    #[test]
    fn computes_direction_cosines_x_axis() {
        let elem = Truss2D::new(1, vec![1, 2], 0.01);
        let nodes = vec![Node::new(1, 0.0, 0.0, 0.0), Node::new(2, 2.0, 0.0, 0.0)];

        let [l, m, n] = elem.direction_cosines(&nodes).unwrap();
        assert!((l - 1.0).abs() < 1e-10);
        assert!(m.abs() < 1e-10);
        assert!(n.abs() < 1e-10);
    }

    #[test]
    fn computes_direction_cosines_y_axis() {
        let elem = Truss2D::new(1, vec![1, 2], 0.01);
        let nodes = vec![Node::new(1, 0.0, 0.0, 0.0), Node::new(2, 0.0, 5.0, 0.0)];

        let [l, m, n] = elem.direction_cosines(&nodes).unwrap();
        assert!(l.abs() < 1e-10);
        assert!((m - 1.0).abs() < 1e-10);
        assert!(n.abs() < 1e-10);
    }

    #[test]
    fn computes_direction_cosines_diagonal() {
        let elem = Truss2D::new(1, vec![1, 2], 0.01);
        let nodes = vec![Node::new(1, 0.0, 0.0, 0.0), Node::new(2, 3.0, 4.0, 0.0)];

        let [l, m, n] = elem.direction_cosines(&nodes).unwrap();
        assert!((l - 0.6).abs() < 1e-10);
        assert!((m - 0.8).abs() < 1e-10);
        assert!(n.abs() < 1e-10);
    }

    #[test]
    fn transformation_matrix_x_axis() {
        let elem = Truss2D::new(1, vec![1, 2], 0.01);
        let nodes = vec![Node::new(1, 0.0, 0.0, 0.0), Node::new(2, 1.0, 0.0, 0.0)];

        let t = elem.transformation_matrix(&nodes).unwrap();
        assert_eq!(t.nrows(), 2);
        assert_eq!(t.ncols(), 6);

        // First row: [1, 0, 0, 0, 0, 0]
        assert!((t[(0, 0)] - 1.0).abs() < 1e-10);
        assert!(t[(0, 1)].abs() < 1e-10);
        assert!(t[(0, 2)].abs() < 1e-10);

        // Second row: [0, 0, 0, 1, 0, 0]
        assert!((t[(1, 3)] - 1.0).abs() < 1e-10);
        assert!(t[(1, 4)].abs() < 1e-10);
        assert!(t[(1, 5)].abs() < 1e-10);
    }

    #[test]
    fn local_stiffness_matrix() {
        let elem = Truss2D::new(1, vec![1, 2], 0.01); // A = 0.01 m²
        let material = make_material(); // E = 210000 MPa

        let length = 2.0; // 2 meters
        let k_local = elem.local_stiffness(length, &material).unwrap();

        // k = A*E/L = 0.01 * 210000 / 2 = 1050
        let expected_k = 1050.0;

        assert_eq!(k_local.nrows(), 2);
        assert_eq!(k_local.ncols(), 2);
        assert!((k_local[(0, 0)] - expected_k).abs() < 1e-6);
        assert!((k_local[(0, 1)] + expected_k).abs() < 1e-6);
        assert!((k_local[(1, 0)] + expected_k).abs() < 1e-6);
        assert!((k_local[(1, 1)] - expected_k).abs() < 1e-6);
    }

    #[test]
    fn global_stiffness_x_axis() {
        // Element along x-axis should have simple stiffness pattern
        let elem = Truss2D::new(1, vec![1, 2], 0.01);
        let nodes = vec![Node::new(1, 0.0, 0.0, 0.0), Node::new(2, 2.0, 0.0, 0.0)];
        let material = make_material();

        let k = elem.stiffness_matrix(&nodes, &material).unwrap();
        assert_eq!(k.nrows(), 6);
        assert_eq!(k.ncols(), 6);

        // k = A*E/L = 0.01 * 210000 / 2 = 1050
        let expected_k = 1050.0;

        // For element along x-axis, only x-DOFs should be coupled
        assert!((k[(0, 0)] - expected_k).abs() < 1e-6);
        assert!((k[(0, 3)] + expected_k).abs() < 1e-6);
        assert!((k[(3, 0)] + expected_k).abs() < 1e-6);
        assert!((k[(3, 3)] - expected_k).abs() < 1e-6);

        // y and z DOFs should have zero stiffness
        assert!(k[(1, 1)].abs() < 1e-10);
        assert!(k[(2, 2)].abs() < 1e-10);
        assert!(k[(4, 4)].abs() < 1e-10);
        assert!(k[(5, 5)].abs() < 1e-10);
    }

    #[test]
    fn global_stiffness_symmetry() {
        let elem = Truss2D::new(1, vec![1, 2], 0.01);
        let nodes = vec![Node::new(1, 0.0, 0.0, 0.0), Node::new(2, 1.0, 2.0, 3.0)];
        let material = make_material();

        let k = elem.stiffness_matrix(&nodes, &material).unwrap();

        // Stiffness matrix must be symmetric
        for i in 0..6 {
            for j in 0..6 {
                assert!(
                    (k[(i, j)] - k[(j, i)]).abs() < 1e-10,
                    "k[{}, {}] = {} != k[{}, {}] = {}",
                    i,
                    j,
                    k[(i, j)],
                    j,
                    i,
                    k[(j, i)]
                );
            }
        }
    }

    #[test]
    fn global_stiffness_equilibrium() {
        // Sum of each row and column should be zero (equilibrium)
        let elem = Truss2D::new(1, vec![1, 2], 0.01);
        let nodes = vec![Node::new(1, 0.0, 0.0, 0.0), Node::new(2, 1.0, 2.0, 3.0)];
        let material = make_material();

        let k = elem.stiffness_matrix(&nodes, &material).unwrap();

        // Check row sums
        for i in 0..6 {
            let row_sum: f64 = (0..6).map(|j| k[(i, j)]).sum();
            assert!(
                row_sum.abs() < 1e-6,
                "Row {} sum = {} (should be ~0)",
                i,
                row_sum
            );
        }

        // Check column sums
        for j in 0..6 {
            let col_sum: f64 = (0..6).map(|i| k[(i, j)]).sum();
            assert!(
                col_sum.abs() < 1e-6,
                "Column {} sum = {} (should be ~0)",
                j,
                col_sum
            );
        }
    }

    #[test]
    fn validates_material_properties() {
        let elem = Truss2D::new(1, vec![1, 2], 0.01);
        let nodes = vec![Node::new(1, 0.0, 0.0, 0.0), Node::new(2, 1.0, 0.0, 0.0)];
        let material = Material::new("INCOMPLETE".to_string()); // Missing E

        let result = elem.stiffness_matrix(&nodes, &material);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("elastic modulus"));
    }

    #[test]
    fn element_trait_properties() {
        let elem = Truss2D::new(1, vec![1, 2], 0.01);
        assert_eq!(elem.num_nodes(), 2);
        assert_eq!(elem.dofs_per_node(), 3);
    }

    #[test]
    fn analytical_solution_simple_truss() {
        // Simple analytical test: 1m bar, area=1m², E=100 MPa
        // Force F=100 N → displacement = FL/AE = 100*1/(1*100) = 1.0 m
        let elem = Truss2D::new(1, vec![1, 2], 1.0);
        let nodes = vec![Node::new(1, 0.0, 0.0, 0.0), Node::new(2, 1.0, 0.0, 0.0)];
        let mut material = Material::new("TEST".to_string());
        material.elastic_modulus = Some(100.0);

        let k = elem.stiffness_matrix(&nodes, &material).unwrap();

        // k = AE/L = 1*100/1 = 100
        assert!((k[(0, 0)] - 100.0).abs() < 1e-6);
        assert!((k[(0, 3)] + 100.0).abs() < 1e-6);
    }
}
