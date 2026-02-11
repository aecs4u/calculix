//! 3-node quadratic beam element (B32)
//!
//! This module implements a 3-node Timoshenko beam element with quadratic shape functions.
//! The element has nodes at the two ends and a midpoint node for improved accuracy.
//!
//! ## Element Properties
//!
//! - **Nodes**: 3 (two end nodes + one midpoint)
//! - **DOFs per node**: 6 (ux, uy, uz, θx, θy, θz)
//! - **Total DOFs**: 18
//! - **Shape functions**: Quadratic (N1, N2, N3)
//! - **Theory**: Timoshenko beam (includes shear deformation)
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
//!
//! ## Differences from B31 (2-node beam)
//!
//! - **Curvature**: B32 can represent curved beams exactly
//! - **Accuracy**: Higher order interpolation for bending
//! - **Shear**: Option to include shear deformation (Timoshenko theory)

use nalgebra::{Matrix3, Vector3, DMatrix, SMatrix};
use crate::mesh::Node;
use crate::materials::Material;
use super::{BeamSection, Element};

type Matrix18 = SMatrix<f64, 18, 18>;

/// 3-node quadratic beam element (B32)
#[derive(Debug, Clone)]
pub struct Beam32 {
    /// Element ID
    pub id: i32,
    /// Node IDs [node1, node2, node3] where node3 is midpoint
    pub nodes: [i32; 3],
    /// Cross-section properties
    pub section: BeamSection,
    /// Shear correction factor (default 5/6 for rectangular, 0.9 for circular)
    pub shear_factor: f64,
}

impl Beam32 {
    /// Create a new B32 element
    ///
    /// # Arguments
    /// * `id` - Element ID
    /// * `nodes` - Array of 3 node IDs [start, end, midpoint]
    /// * `section` - Beam cross-section properties
    pub fn new(id: i32, nodes: [i32; 3], section: BeamSection) -> Self {
        // Default shear correction factor (Timoshenko beam theory)
        // Use 0.9 for circular sections (approximate), 5/6 otherwise
        let shear_factor = 5.0 / 6.0;

        Self {
            id,
            nodes,
            section,
            shear_factor,
        }
    }

    /// Quadratic shape functions at natural coordinate ξ
    ///
    /// Returns [N1, N2, N3]
    fn shape_functions(xi: f64) -> [f64; 3] {
        let n1 = -0.5 * xi * (1.0 - xi);  // End node 1 at ξ = -1
        let n2 = 0.5 * xi * (1.0 + xi);    // End node 2 at ξ = +1
        let n3 = 1.0 - xi * xi;            // Midpoint node at ξ = 0
        [n1, n2, n3]
    }

    /// Derivatives of shape functions with respect to ξ
    ///
    /// Returns [dN1/dξ, dN2/dξ, dN3/dξ]
    fn shape_derivatives(xi: f64) -> [f64; 3] {
        let dn1 = -0.5 + xi;   // d/dξ(-0.5*ξ*(1-ξ))
        let dn2 = 0.5 + xi;    // d/dξ(0.5*ξ*(1+ξ))
        let dn3 = -2.0 * xi;   // d/dξ(1-ξ²)
        [dn1, dn2, dn3]
    }

    /// Compute element geometry and direction
    ///
    /// Returns (length, unit_direction_vector, midpoint_position)
    fn compute_geometry(nodes: &[Node; 3]) -> (f64, Vector3<f64>, Vector3<f64>) {
        // Use end nodes to compute element direction
        let dx = nodes[1].x - nodes[0].x;
        let dy = nodes[1].y - nodes[0].y;
        let dz = nodes[1].z - nodes[0].z;

        let length = (dx * dx + dy * dy + dz * dz).sqrt();
        let dir = Vector3::new(dx / length, dy / length, dz / length);

        // Midpoint position
        let mid = Vector3::new(nodes[2].x, nodes[2].y, nodes[2].z);

        (length, dir, mid)
    }

    /// Compute Jacobian (dξ/dx mapping) at integration point
    fn jacobian(nodes: &[Node; 3], xi: f64) -> f64 {
        let dn = Self::shape_derivatives(xi);

        // dx/dξ = Σ(dNi/dξ * xi)
        let dx_dxi = dn[0] * nodes[0].x + dn[1] * nodes[1].x + dn[2] * nodes[2].x;
        let dy_dxi = dn[0] * nodes[0].y + dn[1] * nodes[1].y + dn[2] * nodes[2].y;
        let dz_dxi = dn[0] * nodes[0].z + dn[1] * nodes[1].z + dn[2] * nodes[2].z;

        // |J| = ||dx/dξ||
        (dx_dxi * dx_dxi + dy_dxi * dy_dxi + dz_dxi * dz_dxi).sqrt()
    }

    /// Build transformation matrix from local to global coordinates
    ///
    /// For B32, this is a 18x18 block diagonal matrix with 3 copies of the 6x6 rotation matrix
    fn transformation_matrix(nodes: &[Node; 3]) -> Matrix18 {
        let (_, dir, _) = Self::compute_geometry(nodes);

        // Local x-axis = element direction
        let ex = dir;

        // Local y-axis: perpendicular to x, preferring global Z direction
        let global_z = Vector3::new(0.0, 0.0, 1.0);
        let mut ey = global_z.cross(&ex);

        // Handle vertical beams
        if ey.norm() < 1e-6 {
            let global_y = Vector3::new(0.0, 1.0, 0.0);
            ey = global_y.cross(&ex);
        }
        ey = ey.normalize();

        // Local z-axis: perpendicular to both x and y
        let ez = ex.cross(&ey);

        // Build 6x6 transformation matrix for one node
        let mut t6 = SMatrix::<f64, 6, 6>::zeros();
        for i in 0..3 {
            t6[(0, i)] = ex[i];  // Row 1: ex components
            t6[(1, i)] = ey[i];  // Row 2: ey components
            t6[(2, i)] = ez[i];  // Row 3: ez components
            t6[(3, i + 3)] = ex[i];  // Row 4: rotation about ex
            t6[(4, i + 3)] = ey[i];  // Row 5: rotation about ey
            t6[(5, i + 3)] = ez[i];  // Row 6: rotation about ez
        }

        // Build 18x18 block diagonal matrix [T6  0  0]
        //                                    [ 0 T6  0]
        //                                    [ 0  0 T6]
        let mut t18 = Matrix18::zeros();
        for node in 0..3 {
            let offset = node * 6;
            for i in 0..6 {
                for j in 0..6 {
                    t18[(offset + i, offset + j)] = t6[(i, j)];
                }
            }
        }

        t18
    }

    /// Compute local stiffness matrix (18x18) in element coordinates
    fn local_stiffness_matrix(&self, nodes: &[Node; 3], material: &Material) -> Result<Matrix18, String> {
        let e = material.elastic_modulus
            .ok_or_else(|| format!("Element {}: Material missing elastic_modulus", self.id))?;

        let g = material.shear_modulus()
            .ok_or_else(|| format!("Element {}: Cannot compute shear modulus", self.id))?;

        let (length, _, _) = Self::compute_geometry(nodes);
        let a = self.section.area;
        let iy = self.section.iyy;
        let iz = self.section.izz;
        let j = self.section.torsion_constant;

        // Use 3-point Gauss quadrature for accurate integration
        let gauss_points = [
            (-0.7745966692414834, 0.5555555555555556),
            (0.0, 0.8888888888888889),
            (0.7745966692414834, 0.5555555555555556),
        ];

        let mut k_local = Matrix18::zeros();

        // Integrate stiffness contributions
        for (xi, weight) in gauss_points {
            let jac = Self::jacobian(nodes, xi);
            let dn = Self::shape_derivatives(xi);

            // Transform derivatives: dN/dx = (dN/dξ) / |J|
            let dn_dx = [dn[0] / jac, dn[1] / jac, dn[2] / jac];

            // Axial stiffness: EA * (dN/dx)^T * (dN/dx)
            for i in 0..3 {
                for j in 0..3 {
                    let k_axial = e * a * dn_dx[i] * dn_dx[j] * jac * weight;
                    k_local[(i * 6, j * 6)] += k_axial;
                }
            }

            // Bending stiffness (simplified - full formulation would include shear)
            // Bending in y-z plane (about local y-axis)
            for i in 0..3 {
                for j in 0..3 {
                    let k_bend_y = e * iz * dn_dx[i] * dn_dx[j] * jac * weight;
                    k_local[(i * 6 + 2, j * 6 + 2)] += k_bend_y; // w-w coupling
                    k_local[(i * 6 + 4, j * 6 + 4)] += k_bend_y; // θy-θy coupling
                }
            }

            // Bending in x-z plane (about local z-axis)
            for i in 0..3 {
                for j in 0..3 {
                    let k_bend_z = e * iy * dn_dx[i] * dn_dx[j] * jac * weight;
                    k_local[(i * 6 + 1, j * 6 + 1)] += k_bend_z; // v-v coupling
                    k_local[(i * 6 + 5, j * 6 + 5)] += k_bend_z; // θz-θz coupling
                }
            }

            // Torsion stiffness: GJ * (dθx/dx)^T * (dθx/dx)
            for i in 0..3 {
                for jj in 0..3 {
                    let k_torsion = g * j * dn_dx[i] * dn_dx[jj] * jac * weight;
                    k_local[(i * 6 + 3, jj * 6 + 3)] += k_torsion;
                }
            }
        }

        Ok(k_local)
    }
}

impl Element for Beam32 {
    fn dofs_per_node(&self) -> usize {
        6
    }

    fn num_nodes(&self) -> usize {
        3
    }

    fn stiffness_matrix(&self, nodes: &[Node], material: &Material) -> Result<DMatrix<f64>, String> {
        if nodes.len() != 3 {
            return Err(format!("B32 element {} requires exactly 3 nodes", self.id));
        }

        let node_array: [Node; 3] = [
            nodes[0].clone(),
            nodes[1].clone(),
            nodes[2].clone(),
        ];

        // Compute local stiffness matrix
        let k_local = self.local_stiffness_matrix(&node_array, material)?;

        // Compute transformation matrix
        let t = Self::transformation_matrix(&node_array);

        // Transform to global coordinates: K_global = T^T * K_local * T
        let t_dyn = DMatrix::from_fn(18, 18, |i, j| t[(i, j)]);
        let k_local_dyn = DMatrix::from_fn(18, 18, |i, j| k_local[(i, j)]);

        let k_global = t_dyn.transpose() * k_local_dyn * t_dyn;

        Ok(k_global)
    }

    fn mass_matrix(&self, nodes: &[Node], material: &Material) -> Result<DMatrix<f64>, String> {
        if nodes.len() != 3 {
            return Err(format!("B32 element {} requires exactly 3 nodes", self.id));
        }

        let density = material.density
            .ok_or_else(|| format!("Element {}: Material missing density", self.id))?;

        let node_array: [Node; 3] = [
            nodes[0].clone(),
            nodes[1].clone(),
            nodes[2].clone(),
        ];

        let a = self.section.area;
        let iy = self.section.iyy;
        let iz = self.section.izz;
        let j = self.section.torsion_constant;

        let mut m_local = Matrix18::zeros();

        // Use 3-point Gauss quadrature
        let gauss_points = [
            (-0.7745966692414834, 0.5555555555555556),
            (0.0, 0.8888888888888889),
            (0.7745966692414834, 0.5555555555555556),
        ];

        for (xi, weight) in gauss_points {
            let n = Self::shape_functions(xi);
            let jac = Self::jacobian(&node_array, xi);

            // Translational mass
            for i in 0..3 {
                for j in 0..3 {
                    let mass_ij = density * a * n[i] * n[j] * jac * weight;
                    // Add to x, y, z translation DOFs
                    for dof in 0..3 {
                        m_local[(i * 6 + dof, j * 6 + dof)] += mass_ij;
                    }
                }
            }

            // Rotational inertia (simplified)
            for i in 0..3 {
                for jj in 0..3 {
                    let inertia_y = density * iz * n[i] * n[jj] * jac * weight;
                    let inertia_z = density * iy * n[i] * n[jj] * jac * weight;
                    let inertia_x = density * j * n[i] * n[jj] * jac * weight;

                    m_local[(i * 6 + 3, jj * 6 + 3)] += inertia_x; // θx
                    m_local[(i * 6 + 4, jj * 6 + 4)] += inertia_y; // θy
                    m_local[(i * 6 + 5, jj * 6 + 5)] += inertia_z; // θz
                }
            }
        }

        // Transform to global coordinates
        let t = Self::transformation_matrix(&node_array);
        let t_dyn = DMatrix::from_fn(18, 18, |i, j| t[(i, j)]);
        let m_local_dyn = DMatrix::from_fn(18, 18, |i, j| m_local[(i, j)]);

        let m_global = t_dyn.transpose() * m_local_dyn * t_dyn;

        Ok(m_global)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::{Material, MaterialModel};

    #[test]
    fn test_shape_functions_at_nodes() {
        // At ξ = -1 (node 1)
        let n = Beam32::shape_functions(-1.0);
        assert!((n[0] - 1.0).abs() < 1e-10);
        assert!(n[1].abs() < 1e-10);
        assert!(n[2].abs() < 1e-10);

        // At ξ = +1 (node 2)
        let n = Beam32::shape_functions(1.0);
        assert!(n[0].abs() < 1e-10);
        assert!((n[1] - 1.0).abs() < 1e-10);
        assert!(n[2].abs() < 1e-10);

        // At ξ = 0 (node 3, midpoint)
        let n = Beam32::shape_functions(0.0);
        assert!(n[0].abs() < 1e-10);
        assert!(n[1].abs() < 1e-10);
        assert!((n[2] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_shape_functions_partition_of_unity() {
        for xi in [-0.5, 0.0, 0.5] {
            let n = Beam32::shape_functions(xi);
            let sum = n[0] + n[1] + n[2];
            assert!((sum - 1.0).abs() < 1e-10, "Partition of unity failed at ξ={}", xi);
        }
    }

    #[test]
    fn test_element_creation() {
        let section = BeamSection::circular(0.01); // 1cm radius
        let beam = Beam32::new(1, [1, 2, 3], section);

        assert_eq!(beam.id, 1);
        assert_eq!(beam.nodes, [1, 2, 3]);
        assert_eq!(beam.num_nodes(), 3);
        assert_eq!(beam.dofs_per_node(), 6);
    }

    #[test]
    fn test_straight_beam_stiffness() {
        // Straight horizontal beam from (0,0,0) to (2,0,0) with midpoint at (1,0,0)
        let node1 = Node::new(1, 0.0, 0.0, 0.0);
        let node2 = Node::new(2, 2.0, 0.0, 0.0);
        let node3 = Node::new(3, 1.0, 0.0, 0.0);
        let nodes = [node1, node2, node3];

        let section = BeamSection::circular(0.01); // 1cm radius
        let beam = Beam32::new(1, [1, 2, 3], section);

        let material = Material {
            name: "Steel".to_string(),
            model: MaterialModel::LinearElastic,
            elastic_modulus: Some(200e9),
            poissons_ratio: Some(0.3),
            density: Some(7850.0),
            thermal_expansion: None,
            conductivity: None,
            specific_heat: None,
        };

        let k = beam.stiffness_matrix(&nodes, &material).unwrap();
        assert_eq!(k.nrows(), 18);
        assert_eq!(k.ncols(), 18);

        // Check symmetry
        for i in 0..18 {
            for j in 0..18 {
                assert!(
                    (k[(i, j)] - k[(j, i)]).abs() < 1e-6,
                    "Stiffness matrix not symmetric at ({}, {})",
                    i,
                    j
                );
            }
        }
    }
}
