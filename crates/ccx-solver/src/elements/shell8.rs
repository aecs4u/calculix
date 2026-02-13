//! S8: 8-node quadratic shell element
//!
//! Serendipity shell element with:
//! - 4 corner nodes + 4 mid-edge nodes
//! - Quadratic shape functions
//! - 6 DOFs per node (ux, uy, uz, θx, θy, θz)
//! - Membrane + bending + transverse shear
//!
//! Node numbering:
//! ```text
//! η ^
//!   |
//!   3-----6-----2
//!   |           |
//!   7           5
//!   |           |
//!   0-----4-----1  --> ξ
//! ```

use nalgebra::{DMatrix, SMatrix, Vector3};

use crate::materials::Material;
use crate::mesh::Node;
use super::Element;

/// S8: 8-node quadratic shell element
#[derive(Debug, Clone)]
pub struct S8 {
    pub id: i32,
    pub nodes: [i32; 8],
    pub thickness: f64,
}

impl S8 {
    /// Create a new S8 element
    pub fn new(id: i32, nodes: [i32; 8], thickness: f64) -> Self {
        Self { id, nodes, thickness }
    }

    /// Serendipity quadratic shape functions
    ///
    /// Natural coordinates: ξ, η ∈ [-1, 1]
    fn shape_functions(xi: f64, eta: f64) -> [f64; 8] {
        [
            // Corner nodes
            -0.25 * (1.0 - xi) * (1.0 - eta) * (1.0 + xi + eta),  // N0
            -0.25 * (1.0 + xi) * (1.0 - eta) * (1.0 - xi + eta),  // N1
            -0.25 * (1.0 + xi) * (1.0 + eta) * (1.0 - xi - eta),  // N2
            -0.25 * (1.0 - xi) * (1.0 + eta) * (1.0 + xi - eta),  // N3
            
            // Mid-edge nodes
            0.5 * (1.0 - xi * xi) * (1.0 - eta),                  // N4 (0-1)
            0.5 * (1.0 + xi) * (1.0 - eta * eta),                 // N5 (1-2)
            0.5 * (1.0 - xi * xi) * (1.0 + eta),                  // N6 (2-3)
            0.5 * (1.0 - xi) * (1.0 - eta * eta),                 // N7 (3-0)
        ]
    }

    /// Shape function derivatives
    fn shape_function_derivatives(xi: f64, eta: f64) -> ([f64; 8], [f64; 8]) {
        let dN_dxi = [
            0.25 * (1.0 - eta) * (2.0 * xi + eta),
            0.25 * (1.0 - eta) * (2.0 * xi - eta),
            0.25 * (1.0 + eta) * (2.0 * xi + eta),
            0.25 * (1.0 + eta) * (2.0 * xi - eta),
            -xi * (1.0 - eta),
            0.5 * (1.0 - eta * eta),
            -xi * (1.0 + eta),
            -0.5 * (1.0 - eta * eta),
        ];

        let dN_deta = [
            0.25 * (1.0 - xi) * (xi + 2.0 * eta),
            0.25 * (1.0 + xi) * (-xi + 2.0 * eta),
            0.25 * (1.0 + xi) * (xi + 2.0 * eta),
            0.25 * (1.0 - xi) * (-xi + 2.0 * eta),
            -0.5 * (1.0 - xi * xi),
            -eta * (1.0 + xi),
            0.5 * (1.0 - xi * xi),
            -eta * (1.0 - xi),
        ];

        (dN_dxi, dN_deta)
    }

    /// 3×3 Gauss quadrature for shell
    fn gauss_points() -> [(f64, f64, f64); 9] {
        let a = 0.774596669241483; // √(3/5)
        let w0 = 0.555555555555556; // 5/9
        let w1 = 0.888888888888889; // 8/9

        [
            (-a, -a, w0 * w0), (0.0, -a, w1 * w0), (a, -a, w0 * w0),
            (-a, 0.0, w0 * w1), (0.0, 0.0, w1 * w1), (a, 0.0, w0 * w1),
            (-a, a, w0 * w0), (0.0, a, w1 * w0), (a, a, w0 * w0),
        ]
    }
}

impl Element for S8 {
    fn stiffness_matrix(&self, nodes: &[Node], material: &Material) -> Result<DMatrix<f64>, String> {
        if nodes.len() != 8 {
            return Err(format!("S8 requires 8 nodes, got {}", nodes.len()));
        }

        let e = material.elastic_modulus.ok_or("Missing elastic modulus")?;
        let nu = material.poissons_ratio.ok_or("Missing Poisson's ratio")?;

        // Simplified shell stiffness: membrane + bending
        // Full implementation would include transverse shear and drilling DOFs
        
        // For now, return placeholder (48×48 identity scaled by E)
        // TODO: Implement full shell theory (membrane, bending, shear)
        let k = DMatrix::<f64>::identity(48, 48) * (e * self.thickness);

        Ok(k)
    }

    fn mass_matrix(&self, nodes: &[Node], material: &Material) -> Result<DMatrix<f64>, String> {
        if nodes.len() != 8 {
            return Err(format!("S8 requires 8 nodes, got {}", nodes.len()));
        }

        let rho = material.density.ok_or("Missing material density")?;

        // Simplified consistent mass matrix
        // TODO: Implement proper shell mass matrix with rotational inertia
        let m = DMatrix::<f64>::identity(48, 48) * (rho * self.thickness);

        Ok(m)
    }

    fn num_nodes(&self) -> usize {
        8
    }

    fn dofs_per_node(&self) -> usize {
        6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s8_creation() {
        let elem = S8::new(1, [1, 2, 3, 4, 5, 6, 7, 8], 0.01);
        assert_eq!(elem.id, 1);
        assert_eq!(elem.nodes.len(), 8);
        assert_eq!(elem.thickness, 0.01);
    }

    #[test]
    fn test_shape_functions_partition_of_unity() {
        let xi = 0.5;
        let eta = 0.3;
        let n = S8::shape_functions(xi, eta);
        let sum: f64 = n.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10, "Shape functions don't sum to 1: {}", sum);
    }

    #[test]
    fn test_element_properties() {
        let elem = S8::new(1, [1, 2, 3, 4, 5, 6, 7, 8], 0.01);
        assert_eq!(elem.num_nodes(), 8);
        assert_eq!(elem.dofs_per_node(), 6);
    }
}
