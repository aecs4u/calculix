//! 3-node quadratic truss element (T3D3)
//!
//! This module implements a 3-node truss element with quadratic shape functions.
//! The element has nodes at the two ends and a midpoint node for improved accuracy.
//!
//! ## Element Properties
//!
//! - **Nodes**: 3 (two end nodes + one midpoint)
//! - **DOFs per node**: 3 (ux, uy, uz)
//! - **Total DOFs**: 9
//! - **Shape functions**: Quadratic (N1, N2, N3)
//!
//! ## Shape Functions (in natural coordinates ξ ∈ [-1, 1])
//!
//! ```text
//! N1(ξ) = -0.5 * ξ * (1 - ξ)    (node 1, at ξ = -1)
//! N2(ξ) = 0.5 * ξ * (1 + ξ)     (node 2, at ξ = +1)
//! N3(ξ) = (1 - ξ²)              (node 3, at ξ = 0, midpoint)
//! ```
//!
//! ## Coordinate Mapping
//!
//! ```text
//! Node 1: ξ = -1  (start)
//! Node 3: ξ =  0  (midpoint)
//! Node 2: ξ = +1  (end)
//! ```

use nalgebra::{Matrix3, Vector3, DMatrix, DVector};
use crate::mesh::Node;
use crate::materials::Material;
use super::Element;

/// 3-node quadratic truss element
#[derive(Debug, Clone)]
pub struct Truss3D {
    /// Element ID
    pub id: i32,
    /// Node IDs [node1, node2, node3] where node3 is midpoint
    pub nodes: [i32; 3],
    /// Cross-sectional area (m²)
    pub area: f64,
}

impl Truss3D {
    /// Create a new T3D3 element
    ///
    /// # Arguments
    /// * `id` - Element ID
    /// * `nodes` - Array of 3 node IDs [start, end, midpoint]
    /// * `area` - Cross-sectional area in m²
    pub fn new(id: i32, nodes: [i32; 3], area: f64) -> Self {
        Self { id, nodes, area }
    }

    /// Quadratic shape functions at natural coordinate ξ
    ///
    /// Returns [N1, N2, N3] where:
    /// - N1: Shape function for node 1 (ξ = -1)
    /// - N2: Shape function for node 2 (ξ = +1)
    /// - N3: Shape function for node 3 (ξ = 0, midpoint)
    fn shape_functions(xi: f64) -> [f64; 3] {
        let n1 = -0.5 * xi * (1.0 - xi);  // End node 1
        let n2 = 0.5 * xi * (1.0 + xi);    // End node 2
        let n3 = 1.0 - xi * xi;            // Midpoint node
        [n1, n2, n3]
    }

    /// Derivatives of shape functions with respect to ξ
    ///
    /// Returns [dN1/dξ, dN2/dξ, dN3/dξ]
    fn shape_derivatives(xi: f64) -> [f64; 3] {
        let dn1 = -0.5 + xi;   // d/dξ(-0.5*ξ*(1-ξ)) = -0.5 + ξ
        let dn2 = 0.5 + xi;    // d/dξ(0.5*ξ*(1+ξ)) = 0.5 + ξ
        let dn3 = -2.0 * xi;   // d/dξ(1-ξ²) = -2ξ
        [dn1, dn2, dn3]
    }

    /// Compute element length and direction vector
    ///
    /// Returns (length, unit_direction_vector)
    fn compute_geometry(nodes: &[Node; 3]) -> (f64, Vector3<f64>) {
        // Use end nodes to compute element direction
        let dx = nodes[1].x - nodes[0].x;
        let dy = nodes[1].y - nodes[0].y;
        let dz = nodes[1].z - nodes[0].z;

        let length = (dx * dx + dy * dy + dz * dz).sqrt();
        let dir = Vector3::new(dx / length, dy / length, dz / length);

        (length, dir)
    }

    /// Compute Jacobian (dξ/dx mapping)
    ///
    /// For curved elements, this varies with position. We use numerical integration.
    fn jacobian(nodes: &[Node; 3], xi: f64) -> f64 {
        let dn = Self::shape_derivatives(xi);

        // dx/dξ = Σ(dNi/dξ * xi)
        let dx_dxi = dn[0] * nodes[0].x + dn[1] * nodes[1].x + dn[2] * nodes[2].x;
        let dy_dxi = dn[0] * nodes[0].y + dn[1] * nodes[1].y + dn[2] * nodes[2].y;
        let dz_dxi = dn[0] * nodes[0].z + dn[1] * nodes[1].z + dn[2] * nodes[2].z;

        // |J| = ||dx/dξ||
        (dx_dxi * dx_dxi + dy_dxi * dy_dxi + dz_dxi * dz_dxi).sqrt()
    }
}

impl Element for Truss3D {
    fn dofs_per_node(&self) -> usize {
        3
    }

    fn num_nodes(&self) -> usize {
        3
    }

    fn stiffness_matrix(&self, nodes: &[Node], material: &Material) -> Result<DMatrix<f64>, String> {
        if nodes.len() != 3 {
            return Err(format!("T3D3 element {} requires exactly 3 nodes", self.id));
        }

        let e = material.elastic_modulus
            .ok_or_else(|| format!("Element {}: Material missing elastic_modulus", self.id))?;

        let a = self.area;

        // Stiffness matrix: 9x9 (3 nodes × 3 DOFs)
        let mut k = DMatrix::zeros(9, 9);

        // Use 3-point Gauss quadrature for accurate integration
        let gauss_points = [
            (-0.7745966692414834, 0.5555555555555556),  // (ξ, weight)
            (0.0, 0.8888888888888889),
            (0.7745966692414834, 0.5555555555555556),
        ];

        for (xi, weight) in gauss_points {
            // Shape function derivatives
            let dn = Self::shape_derivatives(xi);
            let jac = Self::jacobian(&[nodes[0].clone(), nodes[1].clone(), nodes[2].clone()], xi);

            // Strain-displacement matrix B (1x9 for axial strain only)
            // ε = B * u where u = [u1x, u1y, u1z, u2x, u2y, u2z, u3x, u3y, u3z]

            // dN/dx = (dN/dξ) * (dξ/dx) = (dN/dξ) / J
            let dn_dx = [dn[0] / jac, dn[1] / jac, dn[2] / jac];

            // Direction cosines at this integration point
            let dx_dxi = dn[0] * nodes[0].x + dn[1] * nodes[1].x + dn[2] * nodes[2].x;
            let dy_dxi = dn[0] * nodes[0].y + dn[1] * nodes[1].y + dn[2] * nodes[2].y;
            let dz_dxi = dn[0] * nodes[0].z + dn[1] * nodes[1].z + dn[2] * nodes[2].z;

            let length_at_point = (dx_dxi * dx_dxi + dy_dxi * dy_dxi + dz_dxi * dz_dxi).sqrt();
            let lx = dx_dxi / length_at_point;
            let ly = dy_dxi / length_at_point;
            let lz = dz_dxi / length_at_point;

            // B matrix (1x9): εxx = Σ(dNi/dx * lx * ui_x + dNi/dx * ly * ui_y + dNi/dx * lz * ui_z)
            let mut b = DMatrix::zeros(1, 9);
            for i in 0..3 {
                b[(0, i * 3 + 0)] = dn_dx[i] * lx;
                b[(0, i * 3 + 1)] = dn_dx[i] * ly;
                b[(0, i * 3 + 2)] = dn_dx[i] * lz;
            }

            // K += B^T * E*A * B * |J| * weight
            let factor = e * a * jac * weight;
            k += b.transpose() * (factor * &b);
        }

        Ok(k)
    }

    fn mass_matrix(&self, nodes: &[Node], material: &Material) -> Result<DMatrix<f64>, String> {
        if nodes.len() != 3 {
            return Err(format!("T3D3 element {} requires exactly 3 nodes", self.id));
        }

        let density = material.density.unwrap_or(0.0);
        let a = self.area;

        // Consistent mass matrix using quadratic shape functions
        let mut m = DMatrix::zeros(9, 9);

        // Use 3-point Gauss quadrature
        let gauss_points = [
            (-0.7745966692414834, 0.5555555555555556),
            (0.0, 0.8888888888888889),
            (0.7745966692414834, 0.5555555555555556),
        ];

        for (xi, weight) in gauss_points {
            let n = Self::shape_functions(xi);
            let jac = Self::jacobian(&[nodes[0].clone(), nodes[1].clone(), nodes[2].clone()], xi);

            // M = ∫ ρ * A * N^T * N dV = ∫ ρ * A * N^T * N * J dξ
            for i in 0..3 {
                for j in 0..3 {
                    let mass_ij = density * a * n[i] * n[j] * jac * weight;
                    // Add to diagonal blocks (same DOF direction)
                    for dof in 0..3 {
                        m[(i * 3 + dof, j * 3 + dof)] += mass_ij;
                    }
                }
            }
        }

        Ok(m)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_functions_at_nodes() {
        // At ξ = -1 (node 1)
        let n = Truss3D::shape_functions(-1.0);
        assert!((n[0] - 1.0).abs() < 1e-10);
        assert!(n[1].abs() < 1e-10);
        assert!(n[2].abs() < 1e-10);

        // At ξ = +1 (node 2)
        let n = Truss3D::shape_functions(1.0);
        assert!(n[0].abs() < 1e-10);
        assert!((n[1] - 1.0).abs() < 1e-10);
        assert!(n[2].abs() < 1e-10);

        // At ξ = 0 (node 3, midpoint)
        let n = Truss3D::shape_functions(0.0);
        assert!(n[0].abs() < 1e-10);
        assert!(n[1].abs() < 1e-10);
        assert!((n[2] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_shape_functions_partition_of_unity() {
        // Sum of shape functions should equal 1 at any point
        for xi in [-0.5, 0.0, 0.5] {
            let n = Truss3D::shape_functions(xi);
            let sum = n[0] + n[1] + n[2];
            assert!((sum - 1.0).abs() < 1e-10, "Partition of unity failed at ξ={}", xi);
        }
    }

    #[test]
    fn test_shape_derivatives() {
        // At midpoint ξ = 0
        let dn = Truss3D::shape_derivatives(0.0);
        assert!((dn[0] - (-0.5)).abs() < 1e-10);
        assert!((dn[1] - 0.5).abs() < 1e-10);
        assert!((dn[2] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_straight_element_stiffness() {
        // Straight horizontal element from (0,0,0) to (2,0,0) with midpoint at (1,0,0)
        let node1 = Node::new(1, 0.0, 0.0, 0.0);
        let node2 = Node::new(2, 2.0, 0.0, 0.0);
        let node3 = Node::new(3, 1.0, 0.0, 0.0);
        let nodes = [&node1, &node2, &node3];

        let element = Truss3D::new(1, [1, 2, 3], 0.01); // 1 cm² area

        let material = Material {
            name: "Steel".to_string(),
            elastic_modulus: Some(200e9),  // 200 GPa
            poissons_ratio: Some(0.3),
            density: Some(7850.0),
            ..Default::default()
        };

        // Convert &[&Node; 3] to [Node; 3] for the API
        let node_array = [nodes[0].clone(), nodes[1].clone(), nodes[2].clone()];
        let k = element.stiffness_matrix(&node_array, &material).unwrap();
        assert_eq!(k.nrows(), 9);
        assert_eq!(k.ncols(), 9);

        // For a straight element, the stiffness should be positive
        // Check symmetry
        for i in 0..9 {
            for j in 0..9 {
                assert!((k[(i, j)] - k[(j, i)]).abs() < 1e-6,
                    "Stiffness matrix not symmetric at ({}, {})", i, j);
            }
        }
    }

    #[test]
    fn test_element_creation() {
        let element = Truss3D::new(1, [1, 2, 3], 0.01);
        assert_eq!(element.id, 1);
        assert_eq!(element.nodes.to_vec(), vec![1, 2, 3]);
        assert_eq!(element.num_nodes(), 3);
        assert_eq!(element.dofs_per_node(), 3);
        assert_eq!(element.area, 0.01);
    }
}
