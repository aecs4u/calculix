//! C3D20 - 20-node quadratic hexahedral solid element
//!
//! Port of CalculiX C3D20 element with complete isoparametric formulation.
//!
//! # Element Description
//!
//! - **Type**: 3D solid, isoparametric
//! - **Nodes**: 20 (8 corner + 12 mid-edge)
//! - **DOFs**: 3 per node (ux, uy, uz)
//! - **Integration**: 27-point Gauss quadrature (3x3x3)
//! - **Element family**: Serendipity family
//!
//! # Node Numbering
//!
//! ```text
//!        7-------18------6
//!       /|              /|
//!     19 |            17 |
//!     /  15          /   14
//!    4-------16-----5    |
//!    |   |          |    |
//!    |   3------10--|----2
//!   12  /          13   /
//!    | 11            |  9
//!    |/              | /
//!    0-------8-------1
//! ```
//!
//! Corner nodes: 0-7
//! Mid-edge nodes: 8-19

use nalgebra::{DMatrix, SMatrix, Vector3};
use crate::mesh::Node;
use crate::materials::Material;
use super::Element;

/// 20-node quadratic hexahedral element
#[derive(Debug, Clone)]
pub struct C3D20 {
    /// Element ID
    pub id: i32,
    /// Node IDs [20 nodes]
    pub nodes: [i32; 20],
    /// Use reduced integration (C3D20R)
    pub reduced_integration: bool,
}

impl C3D20 {
    /// Create a new C3D20 element with full integration (27 points)
    pub fn new(id: i32, nodes: [i32; 20]) -> Self {
        Self {
            id,
            nodes,
            reduced_integration: false,
        }
    }

    /// Create a new C3D20R element with reduced integration (8 points)
    pub fn new_reduced(id: i32, nodes: [i32; 20]) -> Self {
        Self {
            id,
            nodes,
            reduced_integration: true,
        }
    }

    /// Serendipity shape functions at natural coordinates (ξ, η, ζ)
    ///
    /// Natural coordinate system: ξ, η, ζ ∈ [-1, 1]
    ///
    /// Returns array of 20 shape function values
    fn shape_functions(xi: f64, eta: f64, zeta: f64) -> [f64; 20] {
        // Corner nodes (1 ± ξ)(1 ± η)(1 ± ζ)(ξ + η + ζ - 2)/8
        let n0 = 0.125 * (1.0 - xi) * (1.0 - eta) * (1.0 - zeta) * (-xi - eta - zeta - 2.0);
        let n1 = 0.125 * (1.0 + xi) * (1.0 - eta) * (1.0 - zeta) * (xi - eta - zeta - 2.0);
        let n2 = 0.125 * (1.0 + xi) * (1.0 + eta) * (1.0 - zeta) * (xi + eta - zeta - 2.0);
        let n3 = 0.125 * (1.0 - xi) * (1.0 + eta) * (1.0 - zeta) * (-xi + eta - zeta - 2.0);
        let n4 = 0.125 * (1.0 - xi) * (1.0 - eta) * (1.0 + zeta) * (-xi - eta + zeta - 2.0);
        let n5 = 0.125 * (1.0 + xi) * (1.0 - eta) * (1.0 + zeta) * (xi - eta + zeta - 2.0);
        let n6 = 0.125 * (1.0 + xi) * (1.0 + eta) * (1.0 + zeta) * (xi + eta + zeta - 2.0);
        let n7 = 0.125 * (1.0 - xi) * (1.0 + eta) * (1.0 + zeta) * (-xi + eta + zeta - 2.0);

        // Mid-edge nodes (1 - ξ²)(1 ± η)(1 ± ζ)/4, etc.
        let n8  = 0.25 * (1.0 - xi * xi) * (1.0 - eta) * (1.0 - zeta);  // Edge 0-1
        let n9  = 0.25 * (1.0 + xi) * (1.0 - eta * eta) * (1.0 - zeta);  // Edge 1-2
        let n10 = 0.25 * (1.0 - xi * xi) * (1.0 + eta) * (1.0 - zeta);  // Edge 2-3
        let n11 = 0.25 * (1.0 - xi) * (1.0 - eta * eta) * (1.0 - zeta);  // Edge 3-0
        let n12 = 0.25 * (1.0 - xi) * (1.0 - eta) * (1.0 - zeta * zeta);  // Edge 0-4
        let n13 = 0.25 * (1.0 + xi) * (1.0 - eta) * (1.0 - zeta * zeta);  // Edge 1-5
        let n14 = 0.25 * (1.0 + xi) * (1.0 + eta) * (1.0 - zeta * zeta);  // Edge 2-6
        let n15 = 0.25 * (1.0 - xi) * (1.0 + eta) * (1.0 - zeta * zeta);  // Edge 3-7
        let n16 = 0.25 * (1.0 - xi * xi) * (1.0 - eta) * (1.0 + zeta);  // Edge 4-5
        let n17 = 0.25 * (1.0 + xi) * (1.0 - eta * eta) * (1.0 + zeta);  // Edge 5-6
        let n18 = 0.25 * (1.0 - xi * xi) * (1.0 + eta) * (1.0 + zeta);  // Edge 6-7
        let n19 = 0.25 * (1.0 - xi) * (1.0 - eta * eta) * (1.0 + zeta);  // Edge 7-4

        [n0, n1, n2, n3, n4, n5, n6, n7, n8, n9, n10, n11, n12, n13, n14, n15, n16, n17, n18, n19]
    }

    /// Derivatives of shape functions with respect to natural coordinates
    ///
    /// Returns (dN/dξ, dN/dη, dN/dζ) as 3 arrays of 20 values each
    fn shape_derivatives(xi: f64, eta: f64, zeta: f64) -> ([f64; 20], [f64; 20], [f64; 20]) {
        let mut dn_dxi = [0.0; 20];
        let mut dn_deta = [0.0; 20];
        let mut dn_dzeta = [0.0; 20];

        // Helper variables
        let xp = 1.0 + xi;
        let xm = 1.0 - xi;
        let yp = 1.0 + eta;
        let ym = 1.0 - eta;
        let zp = 1.0 + zeta;
        let zm = 1.0 - zeta;

        // Corner nodes (0-7): N = (1±ξ)(1±η)(1±ζ)(±ξ±η±ζ-2)/8
        // Node 0: (1-ξ)(1-η)(1-ζ)(-ξ-η-ζ-2)/8
        dn_dxi[0]   = 0.125 * ym * zm * (2.0*xi + eta + zeta + 1.0);
        dn_deta[0]  = 0.125 * xm * zm * (xi + 2.0*eta + zeta + 1.0);
        dn_dzeta[0] = 0.125 * xm * ym * (xi + eta + 2.0*zeta + 1.0);

        // Node 1: (1+ξ)(1-η)(1-ζ)(ξ-η-ζ-2)/8
        dn_dxi[1]   = 0.125 * ym * zm * (2.0*xi - eta - zeta - 1.0);
        dn_deta[1]  = 0.125 * xp * zm * (-xi + 2.0*eta + zeta + 1.0);
        dn_dzeta[1] = 0.125 * xp * ym * (-xi + eta + 2.0*zeta + 1.0);

        // Node 2: (1+ξ)(1+η)(1-ζ)(ξ+η-ζ-2)/8
        dn_dxi[2]   = 0.125 * yp * zm * (2.0*xi + eta - zeta - 1.0);
        dn_deta[2]  = 0.125 * xp * zm * (xi + 2.0*eta - zeta - 1.0);
        dn_dzeta[2] = 0.125 * xp * yp * (-xi - eta + 2.0*zeta + 1.0);

        // Node 3: (1-ξ)(1+η)(1-ζ)(-ξ+η-ζ-2)/8
        dn_dxi[3]   = 0.125 * yp * zm * (2.0*xi - eta + zeta + 1.0);
        dn_deta[3]  = 0.125 * xm * zm * (-xi + 2.0*eta - zeta - 1.0);
        dn_dzeta[3] = 0.125 * xm * yp * (xi - eta + 2.0*zeta + 1.0);

        // Node 4: (1-ξ)(1-η)(1+ζ)(-ξ-η+ζ-2)/8
        dn_dxi[4]   = 0.125 * ym * zp * (2.0*xi + eta - zeta + 1.0);
        dn_deta[4]  = 0.125 * xm * zp * (xi + 2.0*eta - zeta + 1.0);
        dn_dzeta[4] = 0.125 * xm * ym * (-xi - eta + 2.0*zeta - 1.0);

        // Node 5: (1+ξ)(1-η)(1+ζ)(ξ-η+ζ-2)/8
        dn_dxi[5]   = 0.125 * ym * zp * (2.0*xi - eta + zeta - 1.0);
        dn_deta[5]  = 0.125 * xp * zp * (-xi + 2.0*eta - zeta + 1.0);
        dn_dzeta[5] = 0.125 * xp * ym * (xi - eta + 2.0*zeta - 1.0);

        // Node 6: (1+ξ)(1+η)(1+ζ)(ξ+η+ζ-2)/8
        dn_dxi[6]   = 0.125 * yp * zp * (2.0*xi + eta + zeta - 1.0);
        dn_deta[6]  = 0.125 * xp * zp * (xi + 2.0*eta + zeta - 1.0);
        dn_dzeta[6] = 0.125 * xp * yp * (xi + eta + 2.0*zeta - 1.0);

        // Node 7: (1-ξ)(1+η)(1+ζ)(-ξ+η+ζ-2)/8
        dn_dxi[7]   = 0.125 * yp * zp * (2.0*xi - eta - zeta + 1.0);
        dn_deta[7]  = 0.125 * xm * zp * (-xi + 2.0*eta + zeta - 1.0);
        dn_dzeta[7] = 0.125 * xm * yp * (-xi + eta + 2.0*zeta - 1.0);

        // Mid-edge nodes (8-19)
        // Node 8: (1-ξ²)(1-η)(1-ζ)/4 (edge 0-1)
        dn_dxi[8]   = -0.5 * xi * ym * zm;
        dn_deta[8]  = -0.25 * (1.0 - xi*xi) * zm;
        dn_dzeta[8] = -0.25 * (1.0 - xi*xi) * ym;

        // Node 9: (1+ξ)(1-η²)(1-ζ)/4 (edge 1-2)
        dn_dxi[9]   = 0.25 * (1.0 - eta*eta) * zm;
        dn_deta[9]  = -0.5 * xp * eta * zm;
        dn_dzeta[9] = -0.25 * xp * (1.0 - eta*eta);

        // Node 10: (1-ξ²)(1+η)(1-ζ)/4 (edge 2-3)
        dn_dxi[10]   = -0.5 * xi * yp * zm;
        dn_deta[10]  = 0.25 * (1.0 - xi*xi) * zm;
        dn_dzeta[10] = -0.25 * (1.0 - xi*xi) * yp;

        // Node 11: (1-ξ)(1-η²)(1-ζ)/4 (edge 3-0)
        dn_dxi[11]   = -0.25 * (1.0 - eta*eta) * zm;
        dn_deta[11]  = -0.5 * xm * eta * zm;
        dn_dzeta[11] = -0.25 * xm * (1.0 - eta*eta);

        // Node 12: (1-ξ)(1-η)(1-ζ²)/4 (edge 0-4)
        dn_dxi[12]   = -0.25 * ym * (1.0 - zeta*zeta);
        dn_deta[12]  = -0.25 * xm * (1.0 - zeta*zeta);
        dn_dzeta[12] = -0.5 * xm * ym * zeta;

        // Node 13: (1+ξ)(1-η)(1-ζ²)/4 (edge 1-5)
        dn_dxi[13]   = 0.25 * ym * (1.0 - zeta*zeta);
        dn_deta[13]  = -0.25 * xp * (1.0 - zeta*zeta);
        dn_dzeta[13] = -0.5 * xp * ym * zeta;

        // Node 14: (1+ξ)(1+η)(1-ζ²)/4 (edge 2-6)
        dn_dxi[14]   = 0.25 * yp * (1.0 - zeta*zeta);
        dn_deta[14]  = 0.25 * xp * (1.0 - zeta*zeta);
        dn_dzeta[14] = -0.5 * xp * yp * zeta;

        // Node 15: (1-ξ)(1+η)(1-ζ²)/4 (edge 3-7)
        dn_dxi[15]   = -0.25 * yp * (1.0 - zeta*zeta);
        dn_deta[15]  = 0.25 * xm * (1.0 - zeta*zeta);
        dn_dzeta[15] = -0.5 * xm * yp * zeta;

        // Node 16: (1-ξ²)(1-η)(1+ζ)/4 (edge 4-5)
        dn_dxi[16]   = -0.5 * xi * ym * zp;
        dn_deta[16]  = -0.25 * (1.0 - xi*xi) * zp;
        dn_dzeta[16] = 0.25 * (1.0 - xi*xi) * ym;

        // Node 17: (1+ξ)(1-η²)(1+ζ)/4 (edge 5-6)
        dn_dxi[17]   = 0.25 * (1.0 - eta*eta) * zp;
        dn_deta[17]  = -0.5 * xp * eta * zp;
        dn_dzeta[17] = 0.25 * xp * (1.0 - eta*eta);

        // Node 18: (1-ξ²)(1+η)(1+ζ)/4 (edge 6-7)
        dn_dxi[18]   = -0.5 * xi * yp * zp;
        dn_deta[18]  = 0.25 * (1.0 - xi*xi) * zp;
        dn_dzeta[18] = 0.25 * (1.0 - xi*xi) * yp;

        // Node 19: (1-ξ)(1-η²)(1+ζ)/4 (edge 7-4)
        dn_dxi[19]   = -0.25 * (1.0 - eta*eta) * zp;
        dn_deta[19]  = -0.5 * xm * eta * zp;
        dn_dzeta[19] = 0.25 * xm * (1.0 - eta*eta);

        (dn_dxi, dn_deta, dn_dzeta)
    }

    /// Compute Jacobian matrix at natural coordinates
    ///
    /// J = [dx/dξ   dy/dξ   dz/dξ  ]
    ///     [dx/dη   dy/dη   dz/dη  ]
    ///     [dx/dζ   dy/dζ   dz/dζ  ]
    fn jacobian(nodes: &[Node; 20], xi: f64, eta: f64, zeta: f64) -> SMatrix<f64, 3, 3> {
        let (dn_dxi, dn_deta, dn_dzeta) = Self::shape_derivatives(xi, eta, zeta);

        let mut jac = SMatrix::<f64, 3, 3>::zeros();

        for i in 0..20 {
            jac[(0, 0)] += dn_dxi[i] * nodes[i].x;
            jac[(0, 1)] += dn_dxi[i] * nodes[i].y;
            jac[(0, 2)] += dn_dxi[i] * nodes[i].z;

            jac[(1, 0)] += dn_deta[i] * nodes[i].x;
            jac[(1, 1)] += dn_deta[i] * nodes[i].y;
            jac[(1, 2)] += dn_deta[i] * nodes[i].z;

            jac[(2, 0)] += dn_dzeta[i] * nodes[i].x;
            jac[(2, 1)] += dn_dzeta[i] * nodes[i].y;
            jac[(2, 2)] += dn_dzeta[i] * nodes[i].z;
        }

        jac
    }

    /// Compute B matrix (strain-displacement matrix) at natural coordinates
    ///
    /// Size: 6 x 60 (6 strain components, 60 DOFs)
    fn b_matrix(nodes: &[Node; 20], xi: f64, eta: f64, zeta: f64) -> DMatrix<f64> {
        let jac = Self::jacobian(nodes, xi, eta, zeta);
        let jac_inv = jac.try_inverse().expect("Singular Jacobian");
        let (dn_dxi, dn_deta, dn_dzeta) = Self::shape_derivatives(xi, eta, zeta);

        let mut b = DMatrix::<f64>::zeros(6, 60);

        for i in 0..20 {
            // Derivatives in physical coordinates
            let dn_dx = jac_inv[(0, 0)] * dn_dxi[i] + jac_inv[(0, 1)] * dn_deta[i] + jac_inv[(0, 2)] * dn_dzeta[i];
            let dn_dy = jac_inv[(1, 0)] * dn_dxi[i] + jac_inv[(1, 1)] * dn_deta[i] + jac_inv[(1, 2)] * dn_dzeta[i];
            let dn_dz = jac_inv[(2, 0)] * dn_dxi[i] + jac_inv[(2, 1)] * dn_deta[i] + jac_inv[(2, 2)] * dn_dzeta[i];

            // εxx = du/dx
            b[(0, 3 * i)] = dn_dx;
            // εyy = dv/dy
            b[(1, 3 * i + 1)] = dn_dy;
            // εzz = dw/dz
            b[(2, 3 * i + 2)] = dn_dz;
            // γxy = du/dy + dv/dx
            b[(3, 3 * i)] = dn_dy;
            b[(3, 3 * i + 1)] = dn_dx;
            // γyz = dv/dz + dw/dy
            b[(4, 3 * i + 1)] = dn_dz;
            b[(4, 3 * i + 2)] = dn_dy;
            // γxz = du/dz + dw/dx
            b[(5, 3 * i)] = dn_dz;
            b[(5, 3 * i + 2)] = dn_dx;
        }

        b
    }

    /// 27-point Gauss quadrature points and weights for 3D integration (full)
    ///
    /// Returns (points, weights) where points are (ξ, η, ζ) coordinates
    fn gauss_points_27() -> (Vec<(f64, f64, f64)>, Vec<f64>) {
        let r = (3.0 / 5.0_f64).sqrt();  // √(3/5)
        let coords = [-r, 0.0, r];
        let w1 = 5.0 / 9.0;
        let w2 = 8.0 / 9.0;
        let weights_1d = [w1, w2, w1];

        let mut points = Vec::new();
        let mut weights = Vec::new();

        for (i, &xi) in coords.iter().enumerate() {
            for (j, &eta) in coords.iter().enumerate() {
                for (k, &zeta) in coords.iter().enumerate() {
                    points.push((xi, eta, zeta));
                    weights.push(weights_1d[i] * weights_1d[j] * weights_1d[k]);
                }
            }
        }

        (points, weights)
    }

    /// 8-point Gauss quadrature for reduced integration (C3D20R)
    ///
    /// Uses 2×2×2 integration scheme to avoid volumetric locking
    fn gauss_points_8() -> (Vec<(f64, f64, f64)>, Vec<f64>) {
        let r = 1.0 / 3.0_f64.sqrt();  // 1/√3
        let coords = [-r, r];
        let weight = 1.0;  // All weights equal for 2-point rule

        let mut points = Vec::new();
        let mut weights = Vec::new();

        for &xi in &coords {
            for &eta in &coords {
                for &zeta in &coords {
                    points.push((xi, eta, zeta));
                    weights.push(weight);
                }
            }
        }

        (points, weights)
    }

    /// Compute element stiffness matrix via numerical integration
    ///
    /// K = ∫∫∫ B^T * D * B * det(J) dξ dη dζ
    pub fn stiffness_matrix(&self, nodes: &[Node; 20], material: &Material) -> Result<DMatrix<f64>, String> {
        eprintln!("    [C3D20] Computing stiffness matrix for element {}", self.id);

        // Debug: Print node coordinates to verify geometry
        eprintln!("    [C3D20] Node coordinates:");
        for (i, node) in nodes.iter().enumerate() {
            eprintln!("      Node {:2}: ({:8.4}, {:8.4}, {:8.4})", i, node.x, node.y, node.z);
        }

        // Material constitutive matrix (6x6) for 3D elasticity
        let d_matrix = self.constitutive_matrix(material)?;

        let mut k = DMatrix::<f64>::zeros(60, 60);

        // Select integration scheme based on element type
        let (gp, gw) = if self.reduced_integration {
            eprintln!("    [C3D20] Using reduced integration (8 points)");
            Self::gauss_points_8()  // C3D20R: 8-point reduced integration
        } else {
            eprintln!("    [C3D20] Using full integration (27 points)");
            Self::gauss_points_27()  // C3D20: 27-point full integration
        };

        for (i, (point, weight)) in gp.iter().zip(gw.iter()).enumerate() {
            let (xi, eta, zeta) = *point;

            // Compute B matrix at this integration point
            let b = Self::b_matrix(nodes, xi, eta, zeta);

            // Compute Jacobian determinant
            let jac = Self::jacobian(nodes, xi, eta, zeta);
            let det_j = jac.determinant();

            if det_j <= 0.0 {
                return Err(format!("Negative Jacobian determinant at point {}: {}", i, det_j));
            }

            if i == 0 || i == gp.len() - 1 {
                eprintln!("    [C3D20]   Point {}: det(J) = {:.6}", i, det_j);
            }

            // K += B^T * D * B * det(J) * weight
            let btd = b.transpose() * &d_matrix;
            let scale = det_j * weight;
            k += (btd * b) * scale;
        }

        eprintln!("    [C3D20] Stiffness matrix computed successfully");
        Ok(k)
    }

    /// Constitutive matrix for 3D linear elasticity
    ///
    /// Returns 6x6 matrix relating stress to strain
    fn constitutive_matrix(&self, material: &Material) -> Result<DMatrix<f64>, String> {
        let e = material.elastic_modulus.ok_or("Missing elastic modulus")?;
        let nu = material.poissons_ratio.ok_or("Missing Poisson's ratio")?;

        let lambda = (e * nu) / ((1.0 + nu) * (1.0 - 2.0 * nu));
        let mu = e / (2.0 * (1.0 + nu));

        let mut d = DMatrix::<f64>::zeros(6, 6);

        // Normal strains
        d[(0, 0)] = lambda + 2.0 * mu;
        d[(1, 1)] = lambda + 2.0 * mu;
        d[(2, 2)] = lambda + 2.0 * mu;

        // Coupling between normal strains
        d[(0, 1)] = lambda;
        d[(0, 2)] = lambda;
        d[(1, 0)] = lambda;
        d[(1, 2)] = lambda;
        d[(2, 0)] = lambda;
        d[(2, 1)] = lambda;

        // Shear strains
        d[(3, 3)] = mu;  // γxy
        d[(4, 4)] = mu;  // γyz
        d[(5, 5)] = mu;  // γxz

        Ok(d)
    }

    /// Compute element mass matrix via numerical integration
    ///
    /// M = ∫∫∫ ρ * N^T * N * det(J) dξ dη dζ
    pub fn mass_matrix_array(&self, nodes: &[Node; 20], material: &Material) -> Result<DMatrix<f64>, String> {
        let density = material.density.ok_or("Missing material density")?;

        let mut m = DMatrix::<f64>::zeros(60, 60);

        // Select integration scheme based on element type
        let (gp, gw) = if self.reduced_integration {
            Self::gauss_points_8()  // C3D20R: 8-point reduced integration
        } else {
            Self::gauss_points_27()  // C3D20: 27-point full integration
        };

        for (point, weight) in gp.iter().zip(gw.iter()) {
            let (xi, eta, zeta) = *point;

            // Shape functions at this integration point
            let n_vals = Self::shape_functions(xi, eta, zeta);

            // Compute Jacobian determinant
            let jac = Self::jacobian(nodes, xi, eta, zeta);
            let det_j = jac.determinant();

            if det_j <= 0.0 {
                return Err(format!("Negative Jacobian determinant: {}", det_j));
            }

            // Build N matrix and compute N^T * N
            for i in 0..20 {
                for j in 0..20 {
                    let n_i_n_j = n_vals[i] * n_vals[j] * density * det_j * weight;
                    // u_i * u_j
                    m[(3 * i, 3 * j)] += n_i_n_j;
                    // v_i * v_j
                    m[(3 * i + 1, 3 * j + 1)] += n_i_n_j;
                    // w_i * w_j
                    m[(3 * i + 2, 3 * j + 2)] += n_i_n_j;
                }
            }
        }

        Ok(m)
    }

    /// Compute stresses at specified natural coordinates
    ///
    /// σ = D × B × u (stress = constitutive × strain-displacement × displacements)
    ///
    /// Returns [sxx, syy, szz, sxy, syz, sxz] at each evaluation point
    pub fn compute_stresses(
        &self,
        nodes: &[Node; 20],
        material: &Material,
        element_displacements: &[f64; 60],
        eval_points: &[(f64, f64, f64)],
    ) -> Result<Vec<[f64; 6]>, String> {
        let d_matrix = self.constitutive_matrix(material)?;
        let u = nalgebra::DVector::from_column_slice(element_displacements);
        let mut stresses = Vec::with_capacity(eval_points.len());

        for &(xi, eta, zeta) in eval_points {
            let b = Self::b_matrix(nodes, xi, eta, zeta);
            // strain = B * u
            let strain = &b * &u;
            // stress = D * strain
            let stress = &d_matrix * &strain;
            stresses.push([stress[0], stress[1], stress[2], stress[3], stress[4], stress[5]]);
        }

        Ok(stresses)
    }

    /// Compute element volume via numerical integration
    ///
    /// V = ∫∫∫ det(J) dξ dη dζ
    pub fn compute_volume(&self, nodes: &[Node; 20]) -> Result<f64, String> {
        // Use full integration for accurate volume
        let (gp, gw) = Self::gauss_points_27();
        let mut volume = 0.0;

        for (point, weight) in gp.iter().zip(gw.iter()) {
            let (xi, eta, zeta) = *point;
            let jac = Self::jacobian(nodes, xi, eta, zeta);
            let det_j = jac.determinant();
            if det_j <= 0.0 {
                return Err(format!("Negative Jacobian: {}", det_j));
            }
            volume += det_j * weight;
        }

        Ok(volume)
    }

    /// Get the 50 stress evaluation points for CalculiX-compatible beam output
    ///
    /// Returns natural coordinates (ξ, η, ζ) for:
    /// - 8 reduced integration Gauss points
    /// - 21 section points at ζ = -1/√3 (station 1)
    /// - 21 section points at ζ = +1/√3 (station 2)
    pub fn beam_stress_points() -> Vec<(f64, f64, f64)> {
        let r = 1.0 / 3.0_f64.sqrt();
        let mut points = Vec::with_capacity(50);

        // 8 reduced integration Gauss points (2×2×2)
        for &xi in &[-r, r] {
            for &eta in &[-r, r] {
                for &zeta in &[-r, r] {
                    points.push((xi, eta, zeta));
                }
            }
        }

        // Section points: 21 points at each of 2 ζ-stations
        // Pattern: 5 rows in ξ direction × variable η points
        // Row layout: 5+3+5+3+5 = 21
        let xi_5 = [-1.0, -r, 0.0, r, 1.0];
        let xi_3 = [-1.0, 0.0, 1.0];

        for &zeta in &[-r, r] {
            // Row 1: ξ = +1 (5 η points)
            for &eta in &xi_5 { points.push((1.0, eta, zeta)); }
            // Row 2: ξ = +r (3 η points)
            for &eta in &xi_3 { points.push((r, eta, zeta)); }
            // Row 3: ξ = 0 (5 η points)
            for &eta in &xi_5 { points.push((0.0, eta, zeta)); }
            // Row 4: ξ = -r (3 η points)
            for &eta in &xi_3 { points.push((-r, eta, zeta)); }
            // Row 5: ξ = -1 (5 η points)
            for &eta in &xi_5 { points.push((-1.0, eta, zeta)); }
        }

        points
    }
}

impl Element for C3D20 {
    fn stiffness_matrix(&self, nodes: &[Node], material: &Material) -> Result<DMatrix<f64>, String> {
        if nodes.len() != 20 {
            return Err(format!("C3D20 requires 20 nodes, got {}", nodes.len()));
        }
        // Convert slice to array by collecting into Vec first
        let nodes_vec: Vec<Node> = nodes.iter().cloned().collect();
        let node_array: [Node; 20] = nodes_vec.try_into()
            .map_err(|_| "Failed to convert nodes to array")?;
        self.stiffness_matrix(&node_array, material)
    }

    fn mass_matrix(&self, nodes: &[Node], material: &Material) -> Result<DMatrix<f64>, String> {
        if nodes.len() != 20 {
            return Err(format!("C3D20 requires 20 nodes, got {}", nodes.len()));
        }
        // Convert slice to array by collecting into Vec first
        let nodes_vec: Vec<Node> = nodes.iter().cloned().collect();
        let node_array: [Node; 20] = nodes_vec.try_into()
            .map_err(|_| "Failed to convert nodes to array")?;
        self.mass_matrix_array(&node_array, material)
    }

    fn num_nodes(&self) -> usize {
        20
    }

    fn dofs_per_node(&self) -> usize {
        3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c3d20_creation() {
        let nodes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
        let elem = C3D20::new(1, nodes);
        assert_eq!(elem.id, 1);
        assert_eq!(elem.nodes.len(), 20);
        assert_eq!(elem.num_nodes(), 20);
        assert_eq!(elem.dofs_per_node(), 3);
    }

    #[test]
    fn test_shape_functions() {
        // At origin (0, 0, 0), all mid-edge nodes should have value 0.25
        let n = C3D20::shape_functions(0.0, 0.0, 0.0);

        // Sum of shape functions should equal 1
        let sum: f64 = n.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10, "Shape functions don't sum to 1: {}", sum);
    }

    #[test]
    fn test_gauss_points() {
        let (points, weights) = C3D20::gauss_points_27();
        assert_eq!(points.len(), 27);
        assert_eq!(weights.len(), 27);

        // Sum of weights should equal 8 (volume of reference cube [-1,1]³)
        let sum: f64 = weights.iter().sum();
        assert!((sum - 8.0).abs() < 1e-10, "Gauss weights don't sum to 8: {}", sum);
    }
}
