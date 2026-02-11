//! 3D solid (continuum) element implementations
//!
//! This module provides implementations for solid elements used in 3D continuum mechanics.
//! Currently supports:
//! - C3D8: 8-node hexahedral (brick) element with trilinear shape functions

use crate::elements::Element;
use crate::materials::Material;
use crate::mesh::Node;
use nalgebra::{DMatrix, Matrix3, SMatrix, Vector3};

/// C3D8: 8-node hexahedral (brick) element
///
/// Node ordering (CalculiX convention):
/// ```text
///        7----------8
///       /|         /|
///      / |        / |
///     3----------4  |
///     |  5-------|--6
///     | /        | /
///     |/         |/
///     1----------2
/// ```
///
/// - Bottom face: nodes 1,2,3,4 (z = -1 in local coords)
/// - Top face: nodes 5,6,7,8 (z = +1 in local coords)
/// - Local coordinates: ξ, η, ζ ∈ [-1, 1]³
/// - Integration: 2×2×2 Gauss quadrature (8 integration points)
/// - DOFs: 3 per node (ux, uy, uz)
#[derive(Debug, Clone)]
pub struct C3D8 {
    /// Element ID
    pub id: i32,
    /// Node IDs (8 corner nodes)
    pub nodes: [i32; 8],
}

impl C3D8 {
    /// Create a new C3D8 element
    pub fn new(id: i32, nodes: [i32; 8]) -> Self {
        Self { id, nodes }
    }

    /// Compute shape functions at natural coordinates (ξ, η, ζ)
    ///
    /// Shape functions for trilinear hexahedron:
    /// N_i = (1 + ξξ_i)(1 + ηη_i)(1 + ζζ_i) / 8
    ///
    /// where (ξ_i, η_i, ζ_i) are the natural coordinates of node i
    fn shape_functions(xi: f64, eta: f64, zeta: f64) -> [f64; 8] {
        [
            (1.0 - xi) * (1.0 - eta) * (1.0 - zeta) / 8.0, // N1
            (1.0 + xi) * (1.0 - eta) * (1.0 - zeta) / 8.0, // N2
            (1.0 + xi) * (1.0 + eta) * (1.0 - zeta) / 8.0, // N3
            (1.0 - xi) * (1.0 + eta) * (1.0 - zeta) / 8.0, // N4
            (1.0 - xi) * (1.0 - eta) * (1.0 + zeta) / 8.0, // N5
            (1.0 + xi) * (1.0 - eta) * (1.0 + zeta) / 8.0, // N6
            (1.0 + xi) * (1.0 + eta) * (1.0 + zeta) / 8.0, // N7
            (1.0 - xi) * (1.0 + eta) * (1.0 + zeta) / 8.0, // N8
        ]
    }

    /// Compute derivatives of shape functions with respect to natural coordinates
    ///
    /// Returns [dN/dξ, dN/dη, dN/dζ] for all 8 nodes
    ///
    /// # Returns
    /// Array of shape [3][8] where:
    /// - First row: dN_i/dξ for i=1..8
    /// - Second row: dN_i/dη for i=1..8
    /// - Third row: dN_i/dζ for i=1..8
    fn shape_derivatives(xi: f64, eta: f64, zeta: f64) -> [[f64; 8]; 3] {
        // Natural coordinates of the 8 nodes
        let xi_n = [-1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0];
        let eta_n = [-1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0];
        let zeta_n = [-1.0, -1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0];

        let mut dN_dxi = [0.0; 8];
        let mut dN_deta = [0.0; 8];
        let mut dN_dzeta = [0.0; 8];

        for i in 0..8 {
            // dN_i/dξ = ξ_i(1 + ηη_i)(1 + ζζ_i) / 8
            dN_dxi[i] = xi_n[i] * (1.0 + eta * eta_n[i]) * (1.0 + zeta * zeta_n[i]) / 8.0;

            // dN_i/dη = (1 + ξξ_i)η_i(1 + ζζ_i) / 8
            dN_deta[i] = (1.0 + xi * xi_n[i]) * eta_n[i] * (1.0 + zeta * zeta_n[i]) / 8.0;

            // dN_i/dζ = (1 + ξξ_i)(1 + ηη_i)ζ_i / 8
            dN_dzeta[i] = (1.0 + xi * xi_n[i]) * (1.0 + eta * eta_n[i]) * zeta_n[i] / 8.0;
        }

        [dN_dxi, dN_deta, dN_dzeta]
    }

    /// Compute Jacobian matrix at natural coordinates (ξ, η, ζ)
    ///
    /// J = [dx/dξ  dy/dξ  dz/dξ]
    ///     [dx/dη  dy/dη  dz/dη]
    ///     [dx/dζ  dy/dζ  dz/dζ]
    ///
    /// where:
    /// dx/dξ = Σ (dN_i/dξ) * x_i
    fn jacobian(&self, nodes: &[Node], xi: f64, eta: f64, zeta: f64) -> Result<Matrix3<f64>, String> {
        let dN = Self::shape_derivatives(xi, eta, zeta);

        let mut J = Matrix3::zeros();

        for i in 0..8 {
            let node_id = self.nodes[i];
            let node = nodes
                .iter()
                .find(|n| n.id == node_id)
                .ok_or_else(|| format!("Node {} not found", node_id))?;

            // Row 0: dx/dξ, dy/dξ, dz/dξ
            J[(0, 0)] += dN[0][i] * node.x;
            J[(0, 1)] += dN[0][i] * node.y;
            J[(0, 2)] += dN[0][i] * node.z;

            // Row 1: dx/dη, dy/dη, dz/dη
            J[(1, 0)] += dN[1][i] * node.x;
            J[(1, 1)] += dN[1][i] * node.y;
            J[(1, 2)] += dN[1][i] * node.z;

            // Row 2: dx/dζ, dy/dζ, dz/dζ
            J[(2, 0)] += dN[2][i] * node.x;
            J[(2, 1)] += dN[2][i] * node.y;
            J[(2, 2)] += dN[2][i] * node.z;
        }

        Ok(J)
    }

    /// Compute strain-displacement matrix (B-matrix) at natural coordinates
    ///
    /// B matrix relates nodal displacements to strains: {ε} = [B]{u}
    ///
    /// Size: 6 rows (strain components) × 24 columns (8 nodes × 3 DOFs)
    ///
    /// Strain components (Voigt notation):
    /// {ε} = [εxx, εyy, εzz, γxy, γyz, γzx]ᵀ
    ///
    /// For each node i:
    /// B = [dN_i/dx    0         0     ]
    ///     [0          dN_i/dy   0     ]
    ///     [0          0         dN_i/dz]
    ///     [dN_i/dy    dN_i/dx   0     ]
    ///     [0          dN_i/dz   dN_i/dy]
    ///     [dN_i/dz    0         dN_i/dx]
    fn strain_displacement_matrix(
        &self,
        nodes: &[Node],
        xi: f64,
        eta: f64,
        zeta: f64,
    ) -> Result<SMatrix<f64, 6, 24>, String> {
        let dN_natural = Self::shape_derivatives(xi, eta, zeta);
        let J = self.jacobian(nodes, xi, eta, zeta)?;
        let J_inv = J
            .try_inverse()
            .ok_or_else(|| "Singular Jacobian matrix".to_string())?;

        let mut B = SMatrix::<f64, 6, 24>::zeros();

        // For each node, compute dN/dx, dN/dy, dN/dz
        for i in 0..8 {
            // dN/dx = J⁻¹ * dN/dξ (matrix-vector product)
            let dN_natural_i = Vector3::new(dN_natural[0][i], dN_natural[1][i], dN_natural[2][i]);
            let dN_global = J_inv * dN_natural_i;

            let dN_dx = dN_global[0];
            let dN_dy = dN_global[1];
            let dN_dz = dN_global[2];

            let col_offset = i * 3;

            // εxx = du/dx
            B[(0, col_offset)] = dN_dx;

            // εyy = dv/dy
            B[(1, col_offset + 1)] = dN_dy;

            // εzz = dw/dz
            B[(2, col_offset + 2)] = dN_dz;

            // γxy = du/dy + dv/dx
            B[(3, col_offset)] = dN_dy;
            B[(3, col_offset + 1)] = dN_dx;

            // γyz = dv/dz + dw/dy
            B[(4, col_offset + 1)] = dN_dz;
            B[(4, col_offset + 2)] = dN_dy;

            // γzx = dw/dx + du/dz
            B[(5, col_offset + 2)] = dN_dx;
            B[(5, col_offset)] = dN_dz;
        }

        Ok(B)
    }

    /// Compute constitutive matrix (D-matrix) for 3D isotropic elasticity
    ///
    /// D matrix relates stresses to strains: {σ} = [D]{ε}
    ///
    /// For isotropic linear elastic material:
    ///       [1-ν   ν     ν     0       0       0    ]
    ///       [ν     1-ν   ν     0       0       0    ]
    ///   E   [ν     ν     1-ν   0       0       0    ]
    /// ───── [0     0     0   (1-2ν)/2  0       0    ]
    /// (1+ν)(1-2ν)
    ///       [0     0     0     0     (1-2ν)/2  0    ]
    ///       [0     0     0     0       0     (1-2ν)/2]
    fn constitutive_matrix(material: &Material) -> Result<SMatrix<f64, 6, 6>, String> {
        let E = material
            .elastic_modulus
            .ok_or("Missing elastic modulus")?;
        let nu = material.poissons_ratio.ok_or("Missing Poisson's ratio")?;

        let factor = E / ((1.0 + nu) * (1.0 - 2.0 * nu));
        let diagonal = 1.0 - nu;
        let shear = (1.0 - 2.0 * nu) / 2.0;

        let mut D = SMatrix::<f64, 6, 6>::zeros();

        // Normal stress components
        D[(0, 0)] = diagonal * factor;
        D[(0, 1)] = nu * factor;
        D[(0, 2)] = nu * factor;

        D[(1, 0)] = nu * factor;
        D[(1, 1)] = diagonal * factor;
        D[(1, 2)] = nu * factor;

        D[(2, 0)] = nu * factor;
        D[(2, 1)] = nu * factor;
        D[(2, 2)] = diagonal * factor;

        // Shear stress components
        D[(3, 3)] = shear * factor;
        D[(4, 4)] = shear * factor;
        D[(5, 5)] = shear * factor;

        Ok(D)
    }
}

impl Element for C3D8 {
    fn num_nodes(&self) -> usize {
        8
    }

    fn dofs_per_node(&self) -> usize {
        3 // ux, uy, uz
    }

    fn stiffness_matrix(
        &self,
        nodes: &[Node],
        material: &Material,
    ) -> Result<DMatrix<f64>, String> {
        // K_e = ∫∫∫ B^T D B |J| dξ dη dζ
        //     ≈ Σ w_i B_i^T D B_i |J_i|  (2×2×2 Gauss quadrature)

        let D = Self::constitutive_matrix(material)?;
        let mut K = DMatrix::zeros(24, 24); // 8 nodes × 3 DOFs

        // 2×2×2 Gauss quadrature
        let gp = 1.0 / f64::sqrt(3.0); // ±1/√3
        let w = 1.0; // Weight for each point

        let gauss_points = [
            (-gp, -gp, -gp),
            (gp, -gp, -gp),
            (gp, gp, -gp),
            (-gp, gp, -gp),
            (-gp, -gp, gp),
            (gp, -gp, gp),
            (gp, gp, gp),
            (-gp, gp, gp),
        ];

        for &(xi, eta, zeta) in &gauss_points {
            let B = self.strain_displacement_matrix(nodes, xi, eta, zeta)?;
            let J = self.jacobian(nodes, xi, eta, zeta)?;
            let det_J = J.determinant();

            if det_J <= 0.0 {
                return Err(format!(
                    "Negative or zero Jacobian determinant: {}",
                    det_J
                ));
            }

            // Convert B and D to DMatrix for multiplication
            let B_dyn = DMatrix::from_fn(6, 24, |i, j| B[(i, j)]);
            let D_dyn = DMatrix::from_fn(6, 6, |i, j| D[(i, j)]);

            // K += B^T * D * B * det(J) * w^3
            K += B_dyn.transpose() * D_dyn * B_dyn * det_J * w * w * w;
        }

        Ok(K)
    }

    fn mass_matrix(
        &self,
        nodes: &[Node],
        material: &Material,
    ) -> Result<DMatrix<f64>, String> {
        // M_e = ∫∫∫ ρ N^T N |J| dξ dη dζ
        //     ≈ Σ w_i ρ N_i^T N_i |J_i|

        let rho = material.density.ok_or("Missing density")?;
        let mut M = DMatrix::zeros(24, 24);

        // 2×2×2 Gauss quadrature
        let gp = 1.0 / f64::sqrt(3.0);
        let w = 1.0;

        let gauss_points = [
            (-gp, -gp, -gp),
            (gp, -gp, -gp),
            (gp, gp, -gp),
            (-gp, gp, -gp),
            (-gp, -gp, gp),
            (gp, -gp, gp),
            (gp, gp, gp),
            (-gp, gp, gp),
        ];

        for &(xi, eta, zeta) in &gauss_points {
            let N = Self::shape_functions(xi, eta, zeta);
            let J = self.jacobian(nodes, xi, eta, zeta)?;
            let det_J = J.determinant();

            if det_J <= 0.0 {
                return Err(format!(
                    "Negative or zero Jacobian determinant: {}",
                    det_J
                ));
            }

            // For each pair of nodes
            for i in 0..8 {
                for j in 0..8 {
                    let mass_contrib = N[i] * N[j] * rho * det_J * w * w * w;

                    // Add to mass matrix for each DOF
                    for k in 0..3 {
                        let row = i * 3 + k;
                        let col = j * 3 + k;
                        M[(row, col)] += mass_contrib;
                    }
                }
            }
        }

        Ok(M)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_functions_partition_of_unity() {
        // Sum of shape functions should equal 1.0 at any point
        let test_points = [
            (0.0, 0.0, 0.0),
            (0.5, 0.5, 0.5),
            (-0.5, 0.3, 0.7),
            (1.0, -1.0, 0.0),
        ];

        for &(xi, eta, zeta) in &test_points {
            let N = C3D8::shape_functions(xi, eta, zeta);
            let sum: f64 = N.iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-12,
                "Shape functions sum = {} at ({}, {}, {})",
                sum,
                xi,
                eta,
                zeta
            );
        }
    }

    #[test]
    fn shape_functions_at_nodes() {
        // N_i should be 1 at node i, 0 at all other nodes
        let node_coords = [
            (-1.0, -1.0, -1.0), // Node 1
            (1.0, -1.0, -1.0),  // Node 2
            (1.0, 1.0, -1.0),   // Node 3
            (-1.0, 1.0, -1.0),  // Node 4
            (-1.0, -1.0, 1.0),  // Node 5
            (1.0, -1.0, 1.0),   // Node 6
            (1.0, 1.0, 1.0),    // Node 7
            (-1.0, 1.0, 1.0),   // Node 8
        ];

        for (i, &(xi, eta, zeta)) in node_coords.iter().enumerate() {
            let N = C3D8::shape_functions(xi, eta, zeta);
            for (j, &n_j) in N.iter().enumerate() {
                if i == j {
                    assert!(
                        (n_j - 1.0).abs() < 1e-12,
                        "N[{}] = {} at node {} (expected 1.0)",
                        j,
                        n_j,
                        i + 1
                    );
                } else {
                    assert!(
                        n_j.abs() < 1e-12,
                        "N[{}] = {} at node {} (expected 0.0)",
                        j,
                        n_j,
                        i + 1
                    );
                }
            }
        }
    }

    #[test]
    fn jacobian_for_unit_cube() {
        // For a unit cube (2×2×2), Jacobian should be identity matrix
        let nodes = vec![
            Node {
                id: 1,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Node {
                id: 2,
                x: 2.0,
                y: 0.0,
                z: 0.0,
            },
            Node {
                id: 3,
                x: 2.0,
                y: 2.0,
                z: 0.0,
            },
            Node {
                id: 4,
                x: 0.0,
                y: 2.0,
                z: 0.0,
            },
            Node {
                id: 5,
                x: 0.0,
                y: 0.0,
                z: 2.0,
            },
            Node {
                id: 6,
                x: 2.0,
                y: 0.0,
                z: 2.0,
            },
            Node {
                id: 7,
                x: 2.0,
                y: 2.0,
                z: 2.0,
            },
            Node {
                id: 8,
                x: 0.0,
                y: 2.0,
                z: 2.0,
            },
        ];

        let elem = C3D8::new(1, [1, 2, 3, 4, 5, 6, 7, 8]);

        let J = elem.jacobian(&nodes, 0.0, 0.0, 0.0).unwrap();

        // For 2×2×2 cube, Jacobian should be identity (all derivatives = 1.0)
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (J[(i, j)] - expected).abs() < 1e-10,
                    "J[{},{}] = {} (expected {})",
                    i,
                    j,
                    J[(i, j)],
                    expected
                );
            }
        }
    }

    #[test]
    fn stiffness_matrix_symmetry() {
        let nodes = vec![
            Node {
                id: 1,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Node {
                id: 2,
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Node {
                id: 3,
                x: 1.0,
                y: 1.0,
                z: 0.0,
            },
            Node {
                id: 4,
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            Node {
                id: 5,
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            Node {
                id: 6,
                x: 1.0,
                y: 0.0,
                z: 1.0,
            },
            Node {
                id: 7,
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Node {
                id: 8,
                x: 0.0,
                y: 1.0,
                z: 1.0,
            },
        ];

        let material = Material {
            name: "steel".to_string(),
            model: crate::materials::MaterialModel::LinearElastic,
            elastic_modulus: Some(200e9),
            poissons_ratio: Some(0.3),
            density: Some(7800.0),
            thermal_expansion: None,
            conductivity: None,
            specific_heat: None,
        };

        let elem = C3D8::new(1, [1, 2, 3, 4, 5, 6, 7, 8]);
        let K = elem.stiffness_matrix(&nodes, &material).unwrap();

        // Check symmetry with relative tolerance
        for i in 0..24 {
            for j in 0..24 {
                let avg = (K[(i, j)].abs() + K[(j, i)].abs()) / 2.0;
                let diff = (K[(i, j)] - K[(j, i)]).abs();
                let rel_diff = if avg > 1e-10 { diff / avg } else { diff };
                assert!(
                    rel_diff < 1e-10,
                    "K not symmetric at ({},{}): K[{},{}]={}, K[{},{}]={}, rel_diff={}",
                    i,
                    j,
                    i,
                    j,
                    K[(i, j)],
                    j,
                    i,
                    K[(j, i)],
                    rel_diff
                );
            }
        }
    }

    #[test]
    fn mass_conservation() {
        let nodes = vec![
            Node {
                id: 1,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Node {
                id: 2,
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Node {
                id: 3,
                x: 1.0,
                y: 1.0,
                z: 0.0,
            },
            Node {
                id: 4,
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            Node {
                id: 5,
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            Node {
                id: 6,
                x: 1.0,
                y: 0.0,
                z: 1.0,
            },
            Node {
                id: 7,
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Node {
                id: 8,
                x: 0.0,
                y: 1.0,
                z: 1.0,
            },
        ];

        let rho = 7800.0;
        let volume = 1.0; // Unit cube
        let physical_mass = rho * volume;

        let material = Material {
            name: "steel".to_string(),
            model: crate::materials::MaterialModel::LinearElastic,
            elastic_modulus: Some(200e9),
            poissons_ratio: Some(0.3),
            density: Some(rho),
            thermal_expansion: None,
            conductivity: None,
            specific_heat: None,
        };

        let elem = C3D8::new(1, [1, 2, 3, 4, 5, 6, 7, 8]);
        let M = elem.mass_matrix(&nodes, &material).unwrap();

        // For 3D element with 3 DOFs per node, sum of all entries = 3 × physical_mass
        // (because each physical mass point contributes to x, y, z directions)
        let total_mass: f64 = M.iter().sum();
        let expected_total = 3.0 * physical_mass;

        let rel_error = (total_mass - expected_total).abs() / expected_total;
        assert!(
            rel_error < 1e-10,
            "Mass conservation failed: total={}, expected={}, rel_error={}",
            total_mass,
            expected_total,
            rel_error
        );
    }
}
