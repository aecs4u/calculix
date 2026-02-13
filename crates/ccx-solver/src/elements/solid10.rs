//! C3D10: 10-node quadratic tetrahedral solid element
//!
//! This module implements the 10-node tetrahedral element with:
//! - 4 corner nodes + 6 mid-edge nodes
//! - Quadratic shape functions
//! - 3 DOFs per node (ux, uy, uz)
//! - 4-point Gauss integration for stiffness
//!
//! Node numbering (CalculiX convention):
//! ```text
//!        v
//!        ^
//!        |
//!        3
//!       /|\
//!      / | \
//!     /  |  \
//!    9   |   8
//!   /    |    \
//!  /     7     \
//! 0------6------1 -> u
//!  \     |     /
//!   \    |    /
//!    4   |   5
//!     \  |  /
//!      \ | /
//!       \|/
//!        2
//!        |
//!        v w
//!
//! Nodes 0-3: corners
//! Nodes 4-9: mid-edge (4: 0-2, 5: 1-2, 6: 0-1, 7: 0-3, 8: 1-3, 9: 2-3)
//! ```

use nalgebra::{DMatrix, SMatrix, Vector3};

use crate::materials::Material;
use crate::mesh::Node;
use super::Element;

/// C3D10: 10-node quadratic tetrahedral element
#[derive(Debug, Clone)]
pub struct C3D10 {
    pub id: i32,
    pub nodes: [i32; 10],
}

impl C3D10 {
    /// Create a new C3D10 element
    pub fn new(id: i32, nodes: [i32; 10]) -> Self {
        Self { id, nodes }
    }

    /// Quadratic tetrahedral shape functions in natural coordinates (ξ, η, ζ)
    ///
    /// Natural coordinates:
    /// - ξ, η, ζ ≥ 0
    /// - ξ + η + ζ ≤ 1
    /// - λ = 1 - ξ - η - ζ (fourth coordinate)
    ///
    /// # Arguments
    /// * `xi` - Natural coordinate ξ
    /// * `eta` - Natural coordinate η
    /// * `zeta` - Natural coordinate ζ
    ///
    /// # Returns
    /// Vector of 10 shape function values
    fn shape_functions(xi: f64, eta: f64, zeta: f64) -> [f64; 10] {
        let lambda = 1.0 - xi - eta - zeta;

        [
            // Corner nodes (0-3)
            lambda * (2.0 * lambda - 1.0),  // N0
            xi * (2.0 * xi - 1.0),          // N1
            eta * (2.0 * eta - 1.0),        // N2
            zeta * (2.0 * zeta - 1.0),      // N3
            
            // Mid-edge nodes (4-9)
            4.0 * lambda * eta,              // N4 (0-2)
            4.0 * xi * eta,                  // N5 (1-2)
            4.0 * lambda * xi,               // N6 (0-1)
            4.0 * lambda * zeta,             // N7 (0-3)
            4.0 * xi * zeta,                 // N8 (1-3)
            4.0 * eta * zeta,                // N9 (2-3)
        ]
    }

    /// Shape function derivatives with respect to natural coordinates
    ///
    /// # Returns
    /// (dN/dξ, dN/dη, dN/dζ) for all 10 nodes
    fn shape_function_derivatives(xi: f64, eta: f64, zeta: f64) -> ([f64; 10], [f64; 10], [f64; 10]) {
        let lambda = 1.0 - xi - eta - zeta;

        // dN/dξ
        let dN_dxi = [
            -(4.0 * lambda - 1.0),       // N0
            4.0 * xi - 1.0,              // N1
            0.0,                         // N2
            0.0,                         // N3
            -4.0 * eta,                  // N4
            4.0 * eta,                   // N5
            4.0 * lambda - 4.0 * xi,     // N6
            -4.0 * zeta,                 // N7
            4.0 * zeta,                  // N8
            0.0,                         // N9
        ];

        // dN/dη
        let dN_deta = [
            -(4.0 * lambda - 1.0),       // N0
            0.0,                         // N1
            4.0 * eta - 1.0,             // N2
            0.0,                         // N3
            4.0 * lambda - 4.0 * eta,    // N4
            4.0 * xi,                    // N5
            -4.0 * xi,                   // N6
            -4.0 * zeta,                 // N7
            0.0,                         // N8
            4.0 * zeta,                  // N9
        ];

        // dN/dζ
        let dN_dzeta = [
            -(4.0 * lambda - 1.0),       // N0
            0.0,                         // N1
            0.0,                         // N2
            4.0 * zeta - 1.0,            // N3
            -4.0 * eta,                  // N4
            0.0,                         // N5
            -4.0 * xi,                   // N6
            4.0 * lambda - 4.0 * zeta,   // N7
            4.0 * xi,                    // N8
            4.0 * eta,                   // N9
        ];

        (dN_dxi, dN_deta, dN_dzeta)
    }

    /// Compute Jacobian matrix
    ///
    /// J = [dx/dξ   dy/dξ   dz/dξ  ]
    ///     [dx/dη   dy/dη   dz/dη  ]
    ///     [dx/dζ   dy/dζ   dz/dζ  ]
    fn jacobian(&self, nodes: &[Node; 10], xi: f64, eta: f64, zeta: f64) -> Result<SMatrix<f64, 3, 3>, String> {
        let (dN_dxi, dN_deta, dN_dzeta) = Self::shape_function_derivatives(xi, eta, zeta);

        let mut j = SMatrix::<f64, 3, 3>::zeros();

        for i in 0..10 {
            j[(0, 0)] += dN_dxi[i] * nodes[i].x;
            j[(0, 1)] += dN_dxi[i] * nodes[i].y;
            j[(0, 2)] += dN_dxi[i] * nodes[i].z;

            j[(1, 0)] += dN_deta[i] * nodes[i].x;
            j[(1, 1)] += dN_deta[i] * nodes[i].y;
            j[(1, 2)] += dN_deta[i] * nodes[i].z;

            j[(2, 0)] += dN_dzeta[i] * nodes[i].x;
            j[(2, 1)] += dN_dzeta[i] * nodes[i].y;
            j[(2, 2)] += dN_dzeta[i] * nodes[i].z;
        }

        Ok(j)
    }

    /// Compute B-matrix (strain-displacement matrix)
    ///
    /// B relates nodal displacements to element strains
    /// ε = B * u
    ///
    /// Size: 6 × 30 (6 strain components, 30 DOFs)
    fn b_matrix(&self, nodes: &[Node; 10], xi: f64, eta: f64, zeta: f64) -> Result<DMatrix<f64>, String> {
        let j = self.jacobian(nodes, xi, eta, zeta)?;
        let j_inv = j.try_inverse()
            .ok_or_else(|| "Singular Jacobian matrix".to_string())?;

        let (dN_dxi, dN_deta, dN_dzeta) = Self::shape_function_derivatives(xi, eta, zeta);

        // Compute shape function derivatives in physical coordinates
        let mut dN_dx = [0.0; 10];
        let mut dN_dy = [0.0; 10];
        let mut dN_dz = [0.0; 10];

        for i in 0..10 {
            dN_dx[i] = j_inv[(0, 0)] * dN_dxi[i] + j_inv[(0, 1)] * dN_deta[i] + j_inv[(0, 2)] * dN_dzeta[i];
            dN_dy[i] = j_inv[(1, 0)] * dN_dxi[i] + j_inv[(1, 1)] * dN_deta[i] + j_inv[(1, 2)] * dN_dzeta[i];
            dN_dz[i] = j_inv[(2, 0)] * dN_dxi[i] + j_inv[(2, 1)] * dN_deta[i] + j_inv[(2, 2)] * dN_dzeta[i];
        }

        // Assemble B matrix (6 × 30)
        let mut b = DMatrix::<f64>::zeros(6, 30);

        for i in 0..10 {
            let col = i * 3;
            
            // ε_xx = ∂u/∂x
            b[(0, col)] = dN_dx[i];
            
            // ε_yy = ∂v/∂y
            b[(1, col + 1)] = dN_dy[i];
            
            // ε_zz = ∂w/∂z
            b[(2, col + 2)] = dN_dz[i];
            
            // γ_xy = ∂u/∂y + ∂v/∂x
            b[(3, col)] = dN_dy[i];
            b[(3, col + 1)] = dN_dx[i];
            
            // γ_yz = ∂v/∂z + ∂w/∂y
            b[(4, col + 1)] = dN_dz[i];
            b[(4, col + 2)] = dN_dy[i];
            
            // γ_xz = ∂u/∂z + ∂w/∂x
            b[(5, col)] = dN_dz[i];
            b[(5, col + 2)] = dN_dx[i];
        }

        Ok(b)
    }

    /// 4-point Gauss quadrature for tetrahedron
    ///
    /// Returns (ξ, η, ζ, weight) for 4 integration points
    fn gauss_points() -> [(f64, f64, f64, f64); 4] {
        let a = 0.585410196624968;
        let b = 0.138196601125011;
        let w = 0.25; // weight for each point (1/4 for tetrahedron)

        [
            (a, b, b, w),
            (b, a, b, w),
            (b, b, a, w),
            (b, b, b, w),
        ]
    }
}

impl Element for C3D10 {
    fn stiffness_matrix(&self, nodes: &[Node], material: &Material) -> Result<DMatrix<f64>, String> {
        if nodes.len() != 10 {
            return Err(format!("C3D10 requires 10 nodes, got {}", nodes.len()));
        }

        let e = material.elastic_modulus.ok_or("Missing elastic modulus")?;
        let nu = material.poissons_ratio.ok_or("Missing Poisson's ratio")?;

        // Convert slice to array
        let nodes_array: [Node; 10] = nodes.iter().cloned().collect::<Vec<_>>()
            .try_into()
            .map_err(|_| "Failed to convert nodes to array")?;

        // Constitutive matrix (6×6) for 3D isotropic elasticity
        let factor = e / ((1.0 + nu) * (1.0 - 2.0 * nu));
        let mut d = DMatrix::<f64>::zeros(6, 6);
        
        d[(0, 0)] = factor * (1.0 - nu);
        d[(1, 1)] = factor * (1.0 - nu);
        d[(2, 2)] = factor * (1.0 - nu);
        d[(0, 1)] = factor * nu;
        d[(0, 2)] = factor * nu;
        d[(1, 0)] = factor * nu;
        d[(1, 2)] = factor * nu;
        d[(2, 0)] = factor * nu;
        d[(2, 1)] = factor * nu;
        d[(3, 3)] = factor * (1.0 - 2.0 * nu) / 2.0;
        d[(4, 4)] = factor * (1.0 - 2.0 * nu) / 2.0;
        d[(5, 5)] = factor * (1.0 - 2.0 * nu) / 2.0;

        // Integrate: K = ∫ B^T * D * B * det(J) dV
        let mut k = DMatrix::<f64>::zeros(30, 30);

        for (xi, eta, zeta, weight) in Self::gauss_points() {
            let b = self.b_matrix(&nodes_array, xi, eta, zeta)?;
            let j = self.jacobian(&nodes_array, xi, eta, zeta)?;
            let det_j = j.determinant();

            if det_j <= 0.0 {
                return Err(format!("Negative Jacobian determinant: {}", det_j));
            }

            // K += B^T * D * B * det(J) * weight
            let btd = b.transpose() * &d;
            let scale = det_j * weight;
            k += (btd * b) * scale;
        }

        Ok(k)
    }

    fn mass_matrix(&self, nodes: &[Node], material: &Material) -> Result<DMatrix<f64>, String> {
        if nodes.len() != 10 {
            return Err(format!("C3D10 requires 10 nodes, got {}", nodes.len()));
        }

        let rho = material.density.ok_or("Missing material density")?;

        let nodes_array: [Node; 10] = nodes.iter().cloned().collect::<Vec<_>>()
            .try_into()
            .map_err(|_| "Failed to convert nodes to array")?;

        // Consistent mass matrix: M = ∫ ρ * N^T * N * det(J) dV
        let mut m = DMatrix::<f64>::zeros(30, 30);

        for (xi, eta, zeta, weight) in Self::gauss_points() {
            let n = Self::shape_functions(xi, eta, zeta);
            let j = self.jacobian(&nodes_array, xi, eta, zeta)?;
            let det_j = j.determinant();

            if det_j <= 0.0 {
                return Err(format!("Negative Jacobian determinant: {}", det_j));
            }

            // Assemble N matrix (3 × 30)
            let mut n_matrix = DMatrix::<f64>::zeros(3, 30);
            for i in 0..10 {
                n_matrix[(0, i * 3)] = n[i];
                n_matrix[(1, i * 3 + 1)] = n[i];
                n_matrix[(2, i * 3 + 2)] = n[i];
            }

            // M += ρ * N^T * N * det(J) * weight
            let scale = rho * det_j * weight;
            m += (n_matrix.transpose() * n_matrix) * scale;
        }

        Ok(m)
    }

    fn num_nodes(&self) -> usize {
        10
    }

    fn dofs_per_node(&self) -> usize {
        3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c3d10_creation() {
        let elem = C3D10::new(1, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(elem.id, 1);
        assert_eq!(elem.nodes.len(), 10);
    }

    #[test]
    fn test_shape_functions_partition_of_unity() {
        // Test at element center
        let xi = 0.25;
        let eta = 0.25;
        let zeta = 0.25;
        let n = C3D10::shape_functions(xi, eta, zeta);
        let sum: f64 = n.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10, "Shape functions don't sum to 1: {}", sum);
    }

    #[test]
    fn test_shape_functions_at_corners() {
        // Test at corner node 0 (λ=1, ξ=η=ζ=0)
        let n = C3D10::shape_functions(0.0, 0.0, 0.0);
        assert!((n[0] - 1.0).abs() < 1e-10);
        for i in 1..10 {
            assert!(n[i].abs() < 1e-10);
        }

        // Test at corner node 1 (ξ=1, η=ζ=0)
        let n = C3D10::shape_functions(1.0, 0.0, 0.0);
        assert!((n[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_element_properties() {
        let elem = C3D10::new(1, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(elem.num_nodes(), 10);
        assert_eq!(elem.dofs_per_node(), 3);
    }
}
