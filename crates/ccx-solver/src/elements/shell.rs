//! Shell elements for structural analysis.
//!
//! Implements S4 (4-node quadrilateral shell element) with:
//! - Membrane action (in-plane stretching)
//! - Bending action (out-of-plane deflection)
//! - Drilling DOF (rotation about normal)
//!
//! Each node has 6 DOFs: ux, uy, uz, θx, θy, θz

use crate::elements::Element;
use crate::materials::Material;
use crate::mesh::Node;
use nalgebra::{DMatrix, SMatrix, Vector3};

/// Shell section properties
#[derive(Debug, Clone, PartialEq)]
pub struct ShellSection {
    /// Shell thickness [m]
    pub thickness: f64,
    /// Optional normal direction for orientation [x, y, z]
    pub normal_direction: Option<[f64; 3]>,
}

impl ShellSection {
    /// Create a new shell section with specified thickness
    pub fn new(thickness: f64) -> Self {
        Self {
            thickness,
            normal_direction: None,
        }
    }

    /// Create a shell section with specified thickness and normal direction
    pub fn with_normal(thickness: f64, normal: [f64; 3]) -> Self {
        Self {
            thickness,
            normal_direction: Some(normal),
        }
    }
}

/// 4-node quadrilateral shell element (S4)
///
/// ## Degrees of Freedom
/// - 6 DOFs per node: [ux, uy, uz, θx, θy, θz]
/// - Total: 24 DOFs (4 nodes × 6 DOFs/node)
///
/// ## Capabilities
/// - Membrane behavior (in-plane stretching)
/// - Bending behavior (out-of-plane deflection)
/// - Drilling stiffness (rotation about surface normal)
///
/// ## Assumptions
/// - Linear elastic material
/// - Small strains (geometric linearity)
/// - Mindlin-Reissner plate theory (includes transverse shear)
#[derive(Debug, Clone)]
pub struct S4 {
    /// Element ID
    pub id: i32,
    /// Node IDs in counter-clockwise order
    pub nodes: Vec<i32>,
    /// Shell section properties
    pub section: ShellSection,
}

impl S4 {
    /// Create a new S4 shell element
    ///
    /// # Arguments
    /// * `id` - Element ID
    /// * `nodes` - Vector of 4 node IDs in counter-clockwise order
    /// * `section` - Shell section properties
    ///
    /// # Panics
    /// Panics if `nodes` does not contain exactly 4 node IDs
    pub fn new(id: i32, nodes: Vec<i32>, section: ShellSection) -> Self {
        assert_eq!(nodes.len(), 4, "S4 element requires exactly 4 nodes");
        Self { id, nodes, section }
    }

    /// Validate the element node count
    fn validate_nodes(&self) -> Result<(), String> {
        if self.nodes.len() != 4 {
            return Err(format!(
                "S4 element {} requires exactly 4 nodes, got {}",
                self.id,
                self.nodes.len()
            ));
        }
        Ok(())
    }

    /// Compute the element area using the shoelace formula
    ///
    /// For a quadrilateral, splits into two triangles and sums areas
    fn element_area(&self, nodes: &[Node]) -> Result<f64, String> {
        if nodes.len() != 4 {
            return Err(format!(
                "Expected 4 nodes for area calculation, got {}",
                nodes.len()
            ));
        }

        // Triangle 1: nodes 0, 1, 2
        let v1 = Vector3::new(
            nodes[1].x - nodes[0].x,
            nodes[1].y - nodes[0].y,
            nodes[1].z - nodes[0].z,
        );
        let v2 = Vector3::new(
            nodes[2].x - nodes[0].x,
            nodes[2].y - nodes[0].y,
            nodes[2].z - nodes[0].z,
        );
        let area1 = 0.5 * v1.cross(&v2).norm();

        // Triangle 2: nodes 0, 2, 3
        let v3 = Vector3::new(
            nodes[3].x - nodes[0].x,
            nodes[3].y - nodes[0].y,
            nodes[3].z - nodes[0].z,
        );
        let area2 = 0.5 * v2.cross(&v3).norm();

        Ok(area1 + area2)
    }

    /// Compute the surface normal vector (unit vector)
    ///
    /// Uses cross product of diagonals: (node3 - node1) × (node4 - node2)
    fn surface_normal(&self, nodes: &[Node]) -> Result<Vector3<f64>, String> {
        if nodes.len() != 4 {
            return Err(format!(
                "Expected 4 nodes for normal calculation, got {}",
                nodes.len()
            ));
        }

        // Diagonal 1: node 0 → node 2
        let diag1 = Vector3::new(
            nodes[2].x - nodes[0].x,
            nodes[2].y - nodes[0].y,
            nodes[2].z - nodes[0].z,
        );

        // Diagonal 2: node 3 → node 1
        let diag2 = Vector3::new(
            nodes[1].x - nodes[3].x,
            nodes[1].y - nodes[3].y,
            nodes[1].z - nodes[3].z,
        );

        let normal = diag1.cross(&diag2);
        let norm = normal.norm();

        if norm < 1e-10 {
            return Err(format!(
                "Element {} has degenerate geometry (zero normal)",
                self.id
            ));
        }

        Ok(normal / norm)
    }

    /// Check if the element is planar within a tolerance
    ///
    /// Computes the deviation of each node from the mean plane
    fn is_planar(&self, nodes: &[Node], tolerance: f64) -> Result<bool, String> {
        if nodes.len() != 4 {
            return Err(format!(
                "Expected 4 nodes for planarity check, got {}",
                nodes.len()
            ));
        }

        // Get surface normal and centroid
        let normal = self.surface_normal(nodes)?;
        let centroid = Vector3::new(
            (nodes[0].x + nodes[1].x + nodes[2].x + nodes[3].x) / 4.0,
            (nodes[0].y + nodes[1].y + nodes[2].y + nodes[3].y) / 4.0,
            (nodes[0].z + nodes[1].z + nodes[2].z + nodes[3].z) / 4.0,
        );

        // Check distance from each node to the plane
        for node in nodes {
            let pos = Vector3::new(node.x, node.y, node.z);
            let deviation = (pos - centroid).dot(&normal).abs();
            if deviation > tolerance {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Bilinear shape functions for 4-node quadrilateral in natural coordinates (ξ, η)
    ///
    /// Returns [N1, N2, N3, N4] where:
    /// - N1 = 1/4 * (1-ξ)(1-η)  (node at (-1,-1))
    /// - N2 = 1/4 * (1+ξ)(1-η)  (node at (+1,-1))
    /// - N3 = 1/4 * (1+ξ)(1+η)  (node at (+1,+1))
    /// - N4 = 1/4 * (1-ξ)(1+η)  (node at (-1,+1))
    fn shape_functions(xi: f64, eta: f64) -> [f64; 4] {
        [
            0.25 * (1.0 - xi) * (1.0 - eta),
            0.25 * (1.0 + xi) * (1.0 - eta),
            0.25 * (1.0 + xi) * (1.0 + eta),
            0.25 * (1.0 - xi) * (1.0 + eta),
        ]
    }

    /// Derivatives of shape functions with respect to natural coordinates (ξ, η)
    ///
    /// Returns (dN/dξ, dN/dη) where each is [dN1, dN2, dN3, dN4]
    fn shape_function_derivatives(xi: f64, eta: f64) -> ([f64; 4], [f64; 4]) {
        let dn_dxi = [
            -0.25 * (1.0 - eta),
            0.25 * (1.0 - eta),
            0.25 * (1.0 + eta),
            -0.25 * (1.0 + eta),
        ];
        let dn_deta = [
            -0.25 * (1.0 - xi),
            -0.25 * (1.0 + xi),
            0.25 * (1.0 + xi),
            0.25 * (1.0 - xi),
        ];
        (dn_dxi, dn_deta)
    }

    /// Compute Jacobian matrix and its inverse at a given integration point
    ///
    /// J = [dx/dξ  dy/dξ]
    ///     [dx/dη  dy/dη]
    ///
    /// Returns (J, J_inv, det_J)
    fn jacobian(
        &self,
        nodes: &[Node],
        xi: f64,
        eta: f64,
    ) -> Result<(nalgebra::Matrix2<f64>, nalgebra::Matrix2<f64>, f64), String> {
        let (dn_dxi, dn_deta) = Self::shape_function_derivatives(xi, eta);

        // Compute Jacobian matrix
        let mut dx_dxi = 0.0;
        let mut dy_dxi = 0.0;
        let mut dx_deta = 0.0;
        let mut dy_deta = 0.0;

        for i in 0..4 {
            dx_dxi += dn_dxi[i] * nodes[i].x;
            dy_dxi += dn_dxi[i] * nodes[i].y;
            dx_deta += dn_deta[i] * nodes[i].x;
            dy_deta += dn_deta[i] * nodes[i].y;
        }

        let j = nalgebra::Matrix2::new(dx_dxi, dy_dxi, dx_deta, dy_deta);

        // Compute determinant
        let det_j = j.determinant();
        if det_j.abs() < 1e-10 {
            return Err(format!("Element {} has singular Jacobian", self.id));
        }

        // Compute inverse
        let j_inv = j
            .try_inverse()
            .ok_or_else(|| format!("Element {} Jacobian not invertible", self.id))?;

        Ok((j, j_inv, det_j))
    }

    /// Compute membrane stiffness matrix (in-plane stretching)
    ///
    /// Uses 2×2 Gauss quadrature integration
    /// Returns 8×8 matrix for membrane DOFs: [ux1, uy1, ux2, uy2, ux3, uy3, ux4, uy4]
    fn membrane_stiffness(
        &self,
        nodes: &[Node],
        material: &Material,
    ) -> Result<nalgebra::SMatrix<f64, 8, 8>, String> {
        if nodes.len() != 4 {
            return Err(format!(
                "Expected 4 nodes for membrane stiffness, got {}",
                nodes.len()
            ));
        }

        // Get material properties
        let e = material
            .elastic_modulus
            .ok_or_else(|| "Material missing elastic modulus".to_string())?;
        let nu = material
            .poissons_ratio
            .ok_or_else(|| "Material missing Poisson's ratio".to_string())?;

        // Plane stress material matrix
        let factor = e / (1.0 - nu * nu);
        let d = nalgebra::Matrix3::new(
            factor,
            factor * nu,
            0.0,
            factor * nu,
            factor,
            0.0,
            0.0,
            0.0,
            factor * (1.0 - nu) / 2.0,
        );

        // 2×2 Gauss quadrature points and weights
        let gp = 1.0 / f64::sqrt(3.0); // ±0.577350...
        let gauss_points = [(-gp, -gp), (gp, -gp), (gp, gp), (-gp, gp)];
        let weights = [1.0, 1.0, 1.0, 1.0];

        // Initialize stiffness matrix
        let mut k_membrane = nalgebra::SMatrix::<f64, 8, 8>::zeros();

        // Integrate over element
        for (gp_idx, &(xi, eta)) in gauss_points.iter().enumerate() {
            let weight = weights[gp_idx];

            // Compute Jacobian
            let (_j, j_inv, det_j) = self.jacobian(nodes, xi, eta)?;

            // Shape function derivatives in natural coordinates
            let (dn_dxi, dn_deta) = Self::shape_function_derivatives(xi, eta);

            // Transform derivatives to physical coordinates using inverse Jacobian
            let mut dn_dx = [0.0; 4];
            let mut dn_dy = [0.0; 4];
            for i in 0..4 {
                dn_dx[i] = j_inv[(0, 0)] * dn_dxi[i] + j_inv[(0, 1)] * dn_deta[i];
                dn_dy[i] = j_inv[(1, 0)] * dn_dxi[i] + j_inv[(1, 1)] * dn_deta[i];
            }

            // Build strain-displacement matrix B (3×8)
            // ε = B * u, where ε = [εxx, εyy, γxy]^T
            let mut b = nalgebra::SMatrix::<f64, 3, 8>::zeros();
            for i in 0..4 {
                b[(0, 2 * i)] = dn_dx[i]; // εxx from ux
                b[(1, 2 * i + 1)] = dn_dy[i]; // εyy from uy
                b[(2, 2 * i)] = dn_dy[i]; // γxy from ux
                b[(2, 2 * i + 1)] = dn_dx[i]; // γxy from uy
            }

            // K += B^T * D * B * det(J) * weight * thickness
            let bt_d = b.transpose() * d;
            let bt_d_b = bt_d * b;
            k_membrane += bt_d_b * det_j * weight * self.section.thickness;
        }

        Ok(k_membrane)
    }

    /// Compute bending stiffness matrix (out-of-plane bending)
    ///
    /// Uses Mindlin-Reissner plate theory (includes transverse shear)
    /// Returns 12×12 matrix for bending DOFs: [uz1, θx1, θy1, uz2, θx2, θy2, ...]
    fn bending_stiffness(
        &self,
        nodes: &[Node],
        material: &Material,
    ) -> Result<nalgebra::SMatrix<f64, 12, 12>, String> {
        if nodes.len() != 4 {
            return Err(format!(
                "Expected 4 nodes for bending stiffness, got {}",
                nodes.len()
            ));
        }

        // Get material properties
        let e = material
            .elastic_modulus
            .ok_or_else(|| "Material missing elastic modulus".to_string())?;
        let nu = material
            .poissons_ratio
            .ok_or_else(|| "Material missing Poisson's ratio".to_string())?;
        let g = e / (2.0 * (1.0 + nu)); // Shear modulus

        let t = self.section.thickness;

        // Bending material matrix (moment-curvature relationship)
        // D_b = E*t³/(12(1-ν²)) * [[1, ν, 0], [ν, 1, 0], [0, 0, (1-ν)/2]]
        let d_factor = e * t * t * t / (12.0 * (1.0 - nu * nu));
        let d_bending = nalgebra::Matrix3::new(
            d_factor,
            d_factor * nu,
            0.0,
            d_factor * nu,
            d_factor,
            0.0,
            0.0,
            0.0,
            d_factor * (1.0 - nu) / 2.0,
        );

        // Shear material matrix (for transverse shear coupling)
        // D_s = κ * G * t * [[1, 0], [0, 1]]
        // where κ = 5/6 is the shear correction factor
        let kappa = 5.0 / 6.0;
        let d_shear_factor = kappa * g * t;
        let d_shear = nalgebra::Matrix2::new(d_shear_factor, 0.0, 0.0, d_shear_factor);

        // 2×2 Gauss quadrature
        let gp = 1.0 / f64::sqrt(3.0);
        let gauss_points = [(-gp, -gp), (gp, -gp), (gp, gp), (-gp, gp)];
        let weights = [1.0, 1.0, 1.0, 1.0];

        let mut k_bending = nalgebra::SMatrix::<f64, 12, 12>::zeros();

        for (gp_idx, &(xi, eta)) in gauss_points.iter().enumerate() {
            let weight = weights[gp_idx];
            let (_j, j_inv, det_j) = self.jacobian(nodes, xi, eta)?;
            let n = Self::shape_functions(xi, eta);
            let (dn_dxi, dn_deta) = Self::shape_function_derivatives(xi, eta);

            let mut dn_dx = [0.0; 4];
            let mut dn_dy = [0.0; 4];
            for i in 0..4 {
                dn_dx[i] = j_inv[(0, 0)] * dn_dxi[i] + j_inv[(0, 1)] * dn_deta[i];
                dn_dy[i] = j_inv[(1, 0)] * dn_dxi[i] + j_inv[(1, 1)] * dn_deta[i];
            }

            // === Bending part: Curvature-rotation relationship ===
            // κ = [∂θy/∂x, -∂θx/∂y, (∂θy/∂y - ∂θx/∂x)]
            let mut bb = nalgebra::SMatrix::<f64, 3, 12>::zeros();
            for i in 0..4 {
                bb[(0, 3 * i + 2)] = dn_dx[i]; // κxx from θy
                bb[(1, 3 * i + 1)] = -dn_dy[i]; // κyy from θx
                bb[(2, 3 * i + 1)] = -dn_dx[i]; // κxy from θx
                bb[(2, 3 * i + 2)] = dn_dy[i]; // κxy from θy
            }

            k_bending += bb.transpose() * d_bending * bb * det_j * weight;

            // === Shear part: Couples uz to rotations ===
            // γ = [∂w/∂x - θy, ∂w/∂y + θx]
            let mut bs = nalgebra::SMatrix::<f64, 2, 12>::zeros();
            for i in 0..4 {
                // γxz = ∂w/∂x - θy
                bs[(0, 3 * i)] = dn_dx[i]; // from uz
                bs[(0, 3 * i + 2)] = -n[i]; // from -θy

                // γyz = ∂w/∂y + θx
                bs[(1, 3 * i)] = dn_dy[i]; // from uz
                bs[(1, 3 * i + 1)] = n[i]; // from θx
            }

            k_bending += bs.transpose() * d_shear * bs * det_j * weight;
        }

        Ok(k_bending)
    }

    /// Compute drilling stiffness (rotation about surface normal)
    ///
    /// Adds artificial stiffness to prevent spurious rotation modes
    /// Returns 4×4 matrix for θz DOFs: [θz1, θz2, θz3, θz4]
    fn drilling_stiffness(
        &self,
        nodes: &[Node],
        material: &Material,
    ) -> Result<nalgebra::SMatrix<f64, 4, 4>, String> {
        if nodes.len() != 4 {
            return Err(format!(
                "Expected 4 nodes for drilling stiffness, got {}",
                nodes.len()
            ));
        }

        // Get material properties
        let e = material
            .elastic_modulus
            .ok_or_else(|| "Material missing elastic modulus".to_string())?;
        let nu = material
            .poissons_ratio
            .ok_or_else(|| "Material missing Poisson's ratio".to_string())?;

        let t = self.section.thickness;
        let area = self.element_area(nodes)?;

        // Drilling stiffness magnitude: typically ~1% of bending stiffness
        // α = 0.01 * E*t³/(12(1-ν²)) * area
        let alpha = 0.01 * e * t * t * t / (12.0 * (1.0 - nu * nu)) * area;

        // 2×2 Gauss quadrature
        let gp = 1.0 / f64::sqrt(3.0);
        let gauss_points = [(-gp, -gp), (gp, -gp), (gp, gp), (-gp, gp)];
        let weights = [1.0, 1.0, 1.0, 1.0];

        let mut k_drilling = nalgebra::SMatrix::<f64, 4, 4>::zeros();

        for (gp_idx, &(xi, eta)) in gauss_points.iter().enumerate() {
            let weight = weights[gp_idx];
            let (_j, _j_inv, det_j) = self.jacobian(nodes, xi, eta)?;
            let (dn_dxi, dn_deta) = Self::shape_function_derivatives(xi, eta);

            // Build drilling strain-displacement matrix (1×4)
            // Simple formulation: strain = sum of rotation derivatives
            let mut bd = nalgebra::SMatrix::<f64, 1, 4>::zeros();
            for i in 0..4 {
                bd[(0, i)] = dn_dxi[i] + dn_deta[i];
            }

            // K += α * Bd^T * Bd * det(J) * weight
            k_drilling += alpha * bd.transpose() * bd * det_j * weight;
        }

        Ok(k_drilling)
    }

    /// Compute full local stiffness matrix (membrane + bending + drilling)
    ///
    /// Returns 24×24 matrix combining all stiffness components:
    /// - Membrane (8×8): in-plane stretching [ux, uy]
    /// - Bending (12×12): out-of-plane bending [uz, θx, θy]
    /// - Drilling (4×4): rotation about normal [θz]
    fn local_stiffness(
        &self,
        nodes: &[Node],
        material: &Material,
    ) -> Result<SMatrix<f64, 24, 24>, String> {
        if nodes.len() != 4 {
            return Err(format!(
                "Expected 4 nodes for local stiffness, got {}",
                nodes.len()
            ));
        }

        // Get component stiffness matrices
        let k_membrane = self.membrane_stiffness(nodes, material)?;
        let k_bending = self.bending_stiffness(nodes, material)?;
        let k_drilling = self.drilling_stiffness(nodes, material)?;

        // Assemble into 24×24 matrix
        // Node i DOFs: [ux, uy, uz, θx, θy, θz] at indices [6*i .. 6*i+6]
        let mut k_local = SMatrix::<f64, 24, 24>::zeros();

        // Membrane stiffness: ux, uy DOFs
        for i in 0..4 {
            for j in 0..4 {
                // ux-ux coupling
                k_local[(6 * i, 6 * j)] = k_membrane[(2 * i, 2 * j)];
                // ux-uy coupling
                k_local[(6 * i, 6 * j + 1)] = k_membrane[(2 * i, 2 * j + 1)];
                // uy-ux coupling
                k_local[(6 * i + 1, 6 * j)] = k_membrane[(2 * i + 1, 2 * j)];
                // uy-uy coupling
                k_local[(6 * i + 1, 6 * j + 1)] = k_membrane[(2 * i + 1, 2 * j + 1)];
            }
        }

        // Bending stiffness: uz, θx, θy DOFs
        for i in 0..4 {
            for j in 0..4 {
                // uz-uz coupling
                k_local[(6 * i + 2, 6 * j + 2)] = k_bending[(3 * i, 3 * j)];
                // uz-θx coupling
                k_local[(6 * i + 2, 6 * j + 3)] = k_bending[(3 * i, 3 * j + 1)];
                // uz-θy coupling
                k_local[(6 * i + 2, 6 * j + 4)] = k_bending[(3 * i, 3 * j + 2)];
                // θx-uz coupling
                k_local[(6 * i + 3, 6 * j + 2)] = k_bending[(3 * i + 1, 3 * j)];
                // θx-θx coupling
                k_local[(6 * i + 3, 6 * j + 3)] = k_bending[(3 * i + 1, 3 * j + 1)];
                // θx-θy coupling
                k_local[(6 * i + 3, 6 * j + 4)] = k_bending[(3 * i + 1, 3 * j + 2)];
                // θy-uz coupling
                k_local[(6 * i + 4, 6 * j + 2)] = k_bending[(3 * i + 2, 3 * j)];
                // θy-θx coupling
                k_local[(6 * i + 4, 6 * j + 3)] = k_bending[(3 * i + 2, 3 * j + 1)];
                // θy-θy coupling
                k_local[(6 * i + 4, 6 * j + 4)] = k_bending[(3 * i + 2, 3 * j + 2)];
            }
        }

        // Drilling stiffness: θz DOFs
        for i in 0..4 {
            for j in 0..4 {
                k_local[(6 * i + 5, 6 * j + 5)] = k_drilling[(i, j)];
            }
        }

        Ok(k_local)
    }

    /// Build transformation matrix (local → global coordinates)
    ///
    /// The local coordinate system is defined by:
    /// - Local x-axis: direction from node 0 to node 1
    /// - Local z-axis: surface normal (via cross product)
    /// - Local y-axis: z × x (right-handed system)
    ///
    /// Returns a 24×24 block-diagonal matrix where each 6×6 block contains
    /// the same 3×3 rotation matrix R repeated twice (for translations and rotations)
    fn transformation_matrix(&self, nodes: &[Node]) -> Result<DMatrix<f64>, String> {
        if nodes.len() != 4 {
            return Err(format!(
                "Expected 4 nodes for transformation, got {}",
                nodes.len()
            ));
        }

        // Define local x-axis: direction from node 0 → node 1
        let x_local_vec = Vector3::new(
            nodes[1].x - nodes[0].x,
            nodes[1].y - nodes[0].y,
            nodes[1].z - nodes[0].z,
        );
        let x_local_norm = x_local_vec.norm();
        if x_local_norm < 1e-10 {
            return Err(format!(
                "Element {} has degenerate x-axis (nodes 0 and 1 coincide)",
                self.id
            ));
        }
        let x_local = x_local_vec / x_local_norm;

        // Define local z-axis: surface normal
        let z_local = self.surface_normal(nodes)?;

        // Define local y-axis: z × x (right-handed system)
        let y_local = z_local.cross(&x_local);
        let y_local_norm = y_local.norm();
        if y_local_norm < 1e-10 {
            return Err(format!(
                "Element {} has degenerate y-axis (x and z are parallel)",
                self.id
            ));
        }
        let y_local = y_local / y_local_norm;

        // Build 3×3 rotation matrix R from basis vectors
        // R = [x_local | y_local | z_local] (column vectors)
        let r = nalgebra::Matrix3::from_columns(&[x_local, y_local, z_local]);

        // Expand to 24×24 block-diagonal transformation matrix
        // Each node has 6 DOFs: [ux, uy, uz, θx, θy, θz]
        // The rotation matrix R applies to both translations and rotations
        let mut t = DMatrix::zeros(24, 24);

        for i in 0..4 {
            // Node i: DOFs are at indices [6*i .. 6*i+6]
            // Translation block [6*i .. 6*i+3, 6*i .. 6*i+3] = R
            for row in 0..3 {
                for col in 0..3 {
                    t[(6 * i + row, 6 * i + col)] = r[(row, col)];
                }
            }
            // Rotation block [6*i+3 .. 6*i+6, 6*i+3 .. 6*i+6] = R
            for row in 0..3 {
                for col in 0..3 {
                    t[(6 * i + 3 + row, 6 * i + 3 + col)] = r[(row, col)];
                }
            }
        }

        Ok(t)
    }

    /// Convert uniform pressure load to equivalent nodal forces
    ///
    /// Uses numerical integration to compute equivalent nodal forces for a uniform
    /// pressure applied normal to the shell surface.
    ///
    /// # Formula
    /// F_i = ∫∫ p * N_i(ξ,η) * |J(ξ,η)| dξ dη * n̂
    ///
    /// where:
    /// - p is the pressure magnitude (positive = compression)
    /// - N_i are the shape functions
    /// - J is the Jacobian determinant
    /// - n̂ is the outward surface normal
    ///
    /// # Arguments
    /// * `nodes` - Element node coordinates (4 nodes)
    /// * `pressure` - Pressure magnitude in Pa (positive = compression into surface)
    ///
    /// # Returns
    /// Array of 4 nodal force vectors, each with 6 DOFs [Fx, Fy, Fz, Mx, My, Mz]
    /// Moments are zero for pure pressure load
    ///
    /// # Errors
    /// Returns error if:
    /// - Invalid node count
    /// - Degenerate element geometry
    pub fn pressure_to_nodal_forces(
        &self,
        nodes: &[Node],
        pressure: f64,
    ) -> Result<[nalgebra::SVector<f64, 6>; 4], String> {
        self.validate_nodes()?;

        if nodes.len() != 4 {
            return Err(format!(
                "Expected 4 nodes for pressure calculation, got {}",
                nodes.len()
            ));
        }

        // Compute surface normal (outward direction)
        let normal = self.surface_normal(nodes)?;

        // 2×2 Gauss quadrature points and weights (same as membrane/bending integration)
        let gauss_xi = [-1.0 / f64::sqrt(3.0), 1.0 / f64::sqrt(3.0)];
        let gauss_eta = [-1.0 / f64::sqrt(3.0), 1.0 / f64::sqrt(3.0)];
        let weights = [1.0, 1.0, 1.0, 1.0]; // w_i * w_j = 1.0 * 1.0 for all combinations

        // Initialize nodal forces (zeros)
        type Vector6 = nalgebra::SVector<f64, 6>;
        let mut nodal_forces = [Vector6::zeros(); 4];

        // Numerical integration over element surface
        let mut gp_idx = 0;
        for &xi in &gauss_xi {
            for &eta in &gauss_eta {
                let weight = weights[gp_idx];

                // Shape functions at this Gauss point
                let n = Self::shape_functions(xi, eta);

                // Jacobian determinant (surface differential element dS)
                let (_j, _j_inv, det_j) = self.jacobian(nodes, xi, eta)?;

                // Differential force at this Gauss point: dF = p * |J| * w
                let df = pressure * det_j * weight;

                // Distribute to nodes via shape functions
                for i in 0..4 {
                    // Force contributions (F = N_i * dF * normal)
                    nodal_forces[i][0] += n[i] * df * normal.x; // Fx
                    nodal_forces[i][1] += n[i] * df * normal.y; // Fy
                    nodal_forces[i][2] += n[i] * df * normal.z; // Fz
                    // Moments remain zero (Mx=0, My=0, Mz=0) for uniform pressure
                }

                gp_idx += 1;
            }
        }

        Ok(nodal_forces)
    }

    /// Compute the local mass matrix (24×24) using consistent mass formulation
    ///
    /// # Theory
    /// The consistent mass matrix for shell elements is derived from:
    /// M = ∫∫ ρ * N^T * N * dA
    ///
    /// where:
    /// - ρ = material density [kg/m³]
    /// - N = shape function matrix
    /// - dA = element of area
    ///
    /// For translational DOFs: M_trans_ij = ∫∫ ρ * t * Ni * Nj * |J| dξ dη
    /// For rotational DOFs: M_rot_ij = ∫∫ (ρ * t³/12) * Ni * Nj * |J| dξ dη
    ///
    /// # Integration
    /// Uses 2×2 Gauss quadrature (4 integration points)
    ///
    /// # DOF Ordering (per node)
    /// - DOFs 0-2: Translations (ux, uy, uz)
    /// - DOFs 3-5: Rotations (θx, θy, θz)
    fn local_mass(&self, nodes: &[Node], material: &Material) -> Result<DMatrix<f64>, String> {
        let rho = material.density
            .ok_or("Material missing density (required for mass matrix)")?;
        let t = self.section.thickness;

        // Initialize 24×24 mass matrix
        let mut m = DMatrix::zeros(24, 24);

        // 2×2 Gauss quadrature
        let gp = 1.0 / f64::sqrt(3.0);
        let gauss_points = [(-gp, -gp), (gp, -gp), (gp, gp), (-gp, gp)];
        let weights = [1.0, 1.0, 1.0, 1.0];

        // Integrate over element
        for (gp_idx, &(xi, eta)) in gauss_points.iter().enumerate() {
            let weight = weights[gp_idx];

            // Get shape functions at this Gauss point
            let n = Self::shape_functions(xi, eta);

            // Get Jacobian determinant
            let (_j, _j_inv, det_j) = self.jacobian(nodes, xi, eta)?;

            // Integration factor for translational mass
            let mass_trans = rho * t * det_j * weight;

            // Integration factor for rotational mass (using t³/12 for rotational inertia)
            let mass_rot = rho * t * t * t / 12.0 * det_j * weight;

            // Assemble mass matrix
            for i in 0..4 {
                for j in 0..4 {
                    let mass_contrib = n[i] * n[j];

                    // Translational DOFs (ux, uy, uz) for each node
                    for dof in 0..3 {
                        let row = i * 6 + dof;
                        let col = j * 6 + dof;
                        m[(row, col)] += mass_trans * mass_contrib;
                    }

                    // Rotational DOFs (θx, θy, θz) for each node
                    for dof in 3..6 {
                        let row = i * 6 + dof;
                        let col = j * 6 + dof;
                        m[(row, col)] += mass_rot * mass_contrib;
                    }
                }
            }
        }

        Ok(m)
    }
}

impl Element for S4 {
    fn stiffness_matrix(
        &self,
        nodes: &[Node],
        material: &Material,
    ) -> Result<DMatrix<f64>, String> {
        self.validate_nodes()?;

        // Get local stiffness matrix
        let k_local = self.local_stiffness(nodes, material)?;

        // Get transformation matrix
        let t = self.transformation_matrix(nodes)?;

        // Transform to global coordinates: K_global = T^T * K_local * T
        let k_global = &t.transpose() * k_local * &t;

        Ok(k_global)
    }

    fn num_nodes(&self) -> usize {
        4
    }

    fn dofs_per_node(&self) -> usize {
        6
    }

    fn mass_matrix(
        &self,
        nodes: &[Node],
        material: &Material,
    ) -> Result<DMatrix<f64>, String> {
        self.validate_nodes()?;

        // Get local mass matrix (24×24)
        let m_local = self.local_mass(nodes, material)?;

        // Get transformation matrix (24×24)
        let t = self.transformation_matrix(nodes)?;

        // Transform to global coordinates: M_global = T^T * M_local * T
        let m_global = &t.transpose() * m_local * &t;

        Ok(m_global)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_square_plate_nodes() -> Vec<Node> {
        vec![
            Node::new(1, 0.0, 0.0, 0.0),
            Node::new(2, 1.0, 0.0, 0.0),
            Node::new(3, 1.0, 1.0, 0.0),
            Node::new(4, 0.0, 1.0, 0.0),
        ]
    }

    fn make_steel_material() -> Material {
        let mut mat = Material::new("Steel".to_string());
        mat.elastic_modulus = Some(200e9); // 200 GPa
        mat.poissons_ratio = Some(0.3);
        mat
    }

    #[test]
    fn creates_shell_element() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section.clone());

        assert_eq!(shell.id, 1);
        assert_eq!(shell.nodes, vec![1, 2, 3, 4]);
        assert_eq!(shell.section.thickness, 0.01);
    }

    #[test]
    #[should_panic(expected = "requires exactly 4 nodes")]
    fn rejects_wrong_node_count() {
        let section = ShellSection::new(0.01);
        let _shell = S4::new(1, vec![1, 2, 3], section);
    }

    #[test]
    fn validates_node_count() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);

        assert!(shell.validate_nodes().is_ok());
    }

    #[test]
    fn computes_element_area() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();

        let area = shell.element_area(&nodes).expect("Should compute area");
        assert!((area - 1.0).abs() < 1e-10, "Square plate area should be 1.0");
    }

    #[test]
    fn computes_surface_normal() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();

        let normal = shell
            .surface_normal(&nodes)
            .expect("Should compute normal");

        // For XY plane, normal should be (0, 0, 1) or (0, 0, -1)
        assert!(normal.z.abs() > 0.99, "Normal should point in Z direction");
        assert!(normal.x.abs() < 1e-10, "Normal X component should be ~0");
        assert!(normal.y.abs() < 1e-10, "Normal Y component should be ~0");
    }

    #[test]
    fn checks_planarity() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();

        let is_planar = shell
            .is_planar(&nodes, 1e-6)
            .expect("Should check planarity");
        assert!(is_planar, "Square plate should be planar");
    }

    #[test]
    fn element_trait_num_nodes() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);

        assert_eq!(shell.num_nodes(), 4);
    }

    #[test]
    fn element_trait_dofs_per_node() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);

        assert_eq!(shell.dofs_per_node(), 6);
    }

    #[test]
    fn drilling_stiffness_dimensions() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        let k_drill = shell
            .drilling_stiffness(&nodes, &material)
            .expect("Should compute drilling stiffness");

        assert_eq!(k_drill.nrows(), 4, "Drilling stiffness should be 4×4");
        assert_eq!(k_drill.ncols(), 4, "Drilling stiffness should be 4×4");
    }

    #[test]
    fn drilling_stiffness_symmetric() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        let k_drill = shell
            .drilling_stiffness(&nodes, &material)
            .expect("Should compute drilling stiffness");

        // Check symmetry
        for i in 0..4 {
            for j in 0..4 {
                let diff = (k_drill[(i, j)] - k_drill[(j, i)]).abs();
                assert!(
                    diff < 1e-6,
                    "Drilling stiffness should be symmetric"
                );
            }
        }
    }

    #[test]
    fn drilling_stiffness_positive() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        let k_drill = shell
            .drilling_stiffness(&nodes, &material)
            .expect("Should compute drilling stiffness");

        // All diagonal elements should be positive
        for i in 0..4 {
            assert!(
                k_drill[(i, i)] > 0.0,
                "Drilling stiffness diagonal elements should be positive"
            );
        }
    }

    #[test]
    fn local_stiffness_dimensions() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        let k_local = shell
            .local_stiffness(&nodes, &material)
            .expect("Should compute local stiffness");

        assert_eq!(k_local.nrows(), 24, "Local stiffness should be 24×24");
        assert_eq!(k_local.ncols(), 24, "Local stiffness should be 24×24");
    }

    #[test]
    fn local_stiffness_symmetric() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        let k_local = shell
            .local_stiffness(&nodes, &material)
            .expect("Should compute local stiffness");

        // Check symmetry
        for i in 0..24 {
            for j in 0..24 {
                let diff = (k_local[(i, j)] - k_local[(j, i)]).abs();
                assert!(
                    diff < 1e-6,
                    "Local stiffness should be symmetric: K[{},{}]={:.6e}, K[{},{}]={:.6e}",
                    i,
                    j,
                    k_local[(i, j)],
                    j,
                    i,
                    k_local[(j, i)]
                );
            }
        }
    }

    #[test]
    fn local_stiffness_positive_definite() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        let k_local = shell
            .local_stiffness(&nodes, &material)
            .expect("Should compute local stiffness");

        // Check positive semi-definite (should have ~6 rigid body modes)
        let eigen = k_local.symmetric_eigen();
        let eigenvalues = eigen.eigenvalues;

        let mut positive_eigenvalues = 0;
        let mut near_zero_eigenvalues = 0;

        for &eig in eigenvalues.iter() {
            if eig > 1e-3 {
                positive_eigenvalues += 1;
            } else if eig > -1e-6 {
                near_zero_eigenvalues += 1;
            } else {
                panic!("Found negative eigenvalue: {}", eig);
            }
        }

        // Expect most eigenvalues to be positive (24 DOFs - ~6 rigid body modes)
        assert!(
            positive_eigenvalues >= 15,
            "Should have at least 15 positive eigenvalues, got {}",
            positive_eigenvalues
        );
        // No negative eigenvalues (checked above by panic)
        assert_eq!(
            positive_eigenvalues + near_zero_eigenvalues,
            24,
            "All eigenvalues should be >= 0"
        );
    }

    #[test]
    fn stiffness_matrix_global() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        let k = shell
            .stiffness_matrix(&nodes, &material)
            .expect("Should compute stiffness");

        assert_eq!(k.nrows(), 24, "Global stiffness should be 24×24");
        assert_eq!(k.ncols(), 24, "Global stiffness should be 24×24");

        // Check symmetry
        for i in 0..24 {
            for j in 0..24 {
                let diff = (k[(i, j)] - k[(j, i)]).abs();
                assert!(
                    diff < 1e-6,
                    "Global stiffness should be symmetric"
                );
            }
        }
    }

    #[test]
    fn transformation_matrix_dimensions() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();

        let t = shell
            .transformation_matrix(&nodes)
            .expect("Should compute transformation");

        assert_eq!(t.nrows(), 24, "Transformation matrix should be 24×24");
        assert_eq!(t.ncols(), 24, "Transformation matrix should be 24×24");
    }

    #[test]
    fn transformation_matrix_orthogonal() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();

        let t = shell
            .transformation_matrix(&nodes)
            .expect("Should compute transformation");

        // Check orthogonality: T^T * T = I
        let identity = &t.transpose() * &t;

        // Check diagonal elements are ~1
        for i in 0..24 {
            assert!(
                (identity[(i, i)] - 1.0).abs() < 1e-10,
                "Diagonal element ({},{}) should be 1.0, got {}",
                i,
                i,
                identity[(i, i)]
            );
        }

        // Check off-diagonal elements are ~0
        for i in 0..24 {
            for j in 0..24 {
                if i != j {
                    assert!(
                        identity[(i, j)].abs() < 1e-10,
                        "Off-diagonal element ({},{}) should be ~0, got {}",
                        i,
                        j,
                        identity[(i, j)]
                    );
                }
            }
        }
    }

    #[test]
    fn transformation_matrix_right_handed() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();

        let t = shell
            .transformation_matrix(&nodes)
            .expect("Should compute transformation");

        // Extract the 3×3 rotation matrix from the first node's translation block
        let r11 = t[(0, 0)];
        let r12 = t[(0, 1)];
        let r13 = t[(0, 2)];
        let r21 = t[(1, 0)];
        let r22 = t[(1, 1)];
        let r23 = t[(1, 2)];
        let r31 = t[(2, 0)];
        let r32 = t[(2, 1)];
        let r33 = t[(2, 2)];

        // Check determinant = +1 (right-handed)
        let det = r11 * (r22 * r33 - r23 * r32) - r12 * (r21 * r33 - r23 * r31)
            + r13 * (r21 * r32 - r22 * r31);

        assert!(
            (det - 1.0).abs() < 1e-10,
            "Determinant should be +1 for right-handed system, got {}",
            det
        );
    }

    #[test]
    fn transformation_matrix_block_diagonal() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();

        let t = shell
            .transformation_matrix(&nodes)
            .expect("Should compute transformation");

        // Verify that the rotation matrix is the same for all 4 nodes
        // Compare node 0's translation block with other nodes' translation blocks
        for node in 1..4 {
            for row in 0..3 {
                for col in 0..3 {
                    let val_node0 = t[(row, col)];
                    let val_nodei = t[(6 * node + row, 6 * node + col)];
                    assert!(
                        (val_node0 - val_nodei).abs() < 1e-10,
                        "Node {} translation block should match node 0",
                        node
                    );
                }
            }
        }

        // Verify that translation and rotation blocks are identical for each node
        for node in 0..4 {
            for row in 0..3 {
                for col in 0..3 {
                    let trans_val = t[(6 * node + row, 6 * node + col)];
                    let rot_val = t[(6 * node + 3 + row, 6 * node + 3 + col)];
                    assert!(
                        (trans_val - rot_val).abs() < 1e-10,
                        "Translation and rotation blocks should match for node {}",
                        node
                    );
                }
            }
        }
    }

    #[test]
    fn transformation_matrix_xy_plane() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes(); // Z=0 plane

        let t = shell
            .transformation_matrix(&nodes)
            .expect("Should compute transformation");

        // For XY plane:
        // - Local x should align with global X (node 0→1 is in X direction)
        // - Local z should align with global Z (surface normal points in Z)
        // - Local y should align with global Y

        // Check local x-axis (first column of rotation matrix)
        let x_local_x = t[(0, 0)];
        let x_local_y = t[(1, 0)];
        let x_local_z = t[(2, 0)];
        assert!(
            (x_local_x - 1.0).abs() < 1e-10,
            "Local x should point in global X"
        );
        assert!(x_local_y.abs() < 1e-10, "Local x should have no Y component");
        assert!(x_local_z.abs() < 1e-10, "Local x should have no Z component");

        // Check local z-axis (third column of rotation matrix)
        let z_local_x = t[(0, 2)];
        let z_local_y = t[(1, 2)];
        let z_local_z = t[(2, 2)];
        assert!(z_local_x.abs() < 1e-10, "Local z should have no X component");
        assert!(z_local_y.abs() < 1e-10, "Local z should have no Y component");
        assert!(
            z_local_z.abs() > 0.99,
            "Local z should point in ±Z direction"
        );
    }

    #[test]
    fn shape_functions_partition_of_unity() {
        // Shape functions should sum to 1 at any point
        let test_points = [
            (0.0, 0.0),
            (0.5, 0.5),
            (-0.7, 0.3),
            (0.9, -0.9),
        ];

        for (xi, eta) in test_points {
            let n = S4::shape_functions(xi, eta);
            let sum: f64 = n.iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-10,
                "Shape functions should sum to 1 at ({}, {}), got {}",
                xi,
                eta,
                sum
            );
        }
    }

    #[test]
    fn shape_functions_at_nodes() {
        // At node i, N_i = 1 and all other N_j = 0
        let node_coords = [
            (-1.0, -1.0), // Node 0
            (1.0, -1.0),  // Node 1
            (1.0, 1.0),   // Node 2
            (-1.0, 1.0),  // Node 3
        ];

        for (i, (xi, eta)) in node_coords.iter().enumerate() {
            let n = S4::shape_functions(*xi, *eta);
            for (j, &val) in n.iter().enumerate() {
                if i == j {
                    assert!(
                        (val - 1.0).abs() < 1e-10,
                        "N_{} should be 1 at node {}",
                        j,
                        i
                    );
                } else {
                    assert!(
                        val.abs() < 1e-10,
                        "N_{} should be 0 at node {}, got {}",
                        j,
                        i,
                        val
                    );
                }
            }
        }
    }

    #[test]
    fn jacobian_computation() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes(); // 1×1 square

        // At element center (0,0)
        let (j, j_inv, det_j) = shell
            .jacobian(&nodes, 0.0, 0.0)
            .expect("Should compute Jacobian");

        // For a 1×1 square, Jacobian should be 0.5*I (scaling from [-1,1]² to [0,1]²)
        assert!(
            (j[(0, 0)] - 0.5).abs() < 1e-10,
            "J[0,0] should be 0.5 for unit square"
        );
        assert!(
            (j[(1, 1)] - 0.5).abs() < 1e-10,
            "J[1,1] should be 0.5 for unit square"
        );
        assert!(j[(0, 1)].abs() < 1e-10, "J[0,1] should be 0 for aligned square");
        assert!(j[(1, 0)].abs() < 1e-10, "J[1,0] should be 0 for aligned square");

        // Determinant should be 0.25
        assert!(
            (det_j - 0.25).abs() < 1e-10,
            "det(J) should be 0.25, got {}",
            det_j
        );

        // Check J * J_inv = I
        let identity = j * j_inv;
        assert!(
            (identity[(0, 0)] - 1.0).abs() < 1e-10,
            "J*J_inv should be identity"
        );
        assert!(
            (identity[(1, 1)] - 1.0).abs() < 1e-10,
            "J*J_inv should be identity"
        );
        assert!(
            identity[(0, 1)].abs() < 1e-10,
            "J*J_inv should be identity"
        );
        assert!(
            identity[(1, 0)].abs() < 1e-10,
            "J*J_inv should be identity"
        );
    }

    #[test]
    fn membrane_stiffness_dimensions() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        let k_mem = shell
            .membrane_stiffness(&nodes, &material)
            .expect("Should compute membrane stiffness");

        assert_eq!(k_mem.nrows(), 8, "Membrane stiffness should be 8×8");
        assert_eq!(k_mem.ncols(), 8, "Membrane stiffness should be 8×8");
    }

    #[test]
    fn membrane_stiffness_symmetric() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        let k_mem = shell
            .membrane_stiffness(&nodes, &material)
            .expect("Should compute membrane stiffness");

        // Check symmetry
        for i in 0..8 {
            for j in 0..8 {
                let diff = (k_mem[(i, j)] - k_mem[(j, i)]).abs();
                assert!(
                    diff < 1e-6,
                    "Membrane stiffness should be symmetric: K[{},{}]={}, K[{},{}]={}",
                    i,
                    j,
                    k_mem[(i, j)],
                    j,
                    i,
                    k_mem[(j, i)]
                );
            }
        }
    }

    #[test]
    fn membrane_stiffness_positive_definite() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        let k_mem = shell
            .membrane_stiffness(&nodes, &material)
            .expect("Should compute membrane stiffness");

        // Check positive semi-definite (all eigenvalues ≥ 0)
        // Note: Membrane stiffness has 3 rigid body modes (2 translations + 1 rotation)
        // so we expect 3 near-zero eigenvalues
        let eigen = k_mem.symmetric_eigen();
        let eigenvalues = eigen.eigenvalues;

        let mut positive_eigenvalues = 0;
        let mut near_zero_eigenvalues = 0;

        for &eig in eigenvalues.iter() {
            if eig > 1e-3 {
                positive_eigenvalues += 1;
            } else if eig > -1e-6 {
                near_zero_eigenvalues += 1;
            } else {
                panic!("Found negative eigenvalue: {}", eig);
            }
        }

        assert_eq!(
            positive_eigenvalues, 5,
            "Should have 5 positive eigenvalues (8 DOFs - 3 rigid body modes)"
        );
        assert_eq!(
            near_zero_eigenvalues, 3,
            "Should have 3 near-zero eigenvalues (rigid body modes)"
        );
    }

    #[test]
    fn bending_stiffness_dimensions() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        let k_bend = shell
            .bending_stiffness(&nodes, &material)
            .expect("Should compute bending stiffness");

        assert_eq!(k_bend.nrows(), 12, "Bending stiffness should be 12×12");
        assert_eq!(k_bend.ncols(), 12, "Bending stiffness should be 12×12");
    }

    #[test]
    fn bending_stiffness_symmetric() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        let k_bend = shell
            .bending_stiffness(&nodes, &material)
            .expect("Should compute bending stiffness");

        // Check symmetry
        for i in 0..12 {
            for j in 0..12 {
                let diff = (k_bend[(i, j)] - k_bend[(j, i)]).abs();
                assert!(
                    diff < 1e-6,
                    "Bending stiffness should be symmetric: K[{},{}]={}, K[{},{}]={}",
                    i,
                    j,
                    k_bend[(i, j)],
                    j,
                    i,
                    k_bend[(j, i)]
                );
            }
        }
    }

    #[test]
    fn bending_stiffness_thickness_dependence() {
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        // Note: Mindlin-Reissner formulation includes bending (∝t³) + shear (∝t)
        // For thin plates, shear dominates, so overall stiffness scales between t and t³
        let section_thin = ShellSection::new(0.01);
        let shell_thin = S4::new(1, vec![1, 2, 3, 4], section_thin);
        let k_thin = shell_thin
            .bending_stiffness(&nodes, &material)
            .expect("Should compute bending stiffness");

        let section_thick = ShellSection::new(0.02);
        let shell_thick = S4::new(2, vec![1, 2, 3, 4], section_thick);
        let k_thick = shell_thick
            .bending_stiffness(&nodes, &material)
            .expect("Should compute bending stiffness");

        // For Mindlin-Reissner: stiffness increases with thickness, bounded by t and t³
        let ratio_uz = k_thick[(0, 0)] / k_thin[(0, 0)];
        assert!(
            ratio_uz >= 2.0 && ratio_uz <= 8.0,
            "Bending stiffness should increase with thickness, got ratio {}",
            ratio_uz
        );

        // Check that thicker plate is stiffer
        assert!(k_thick[(0, 0)] > k_thin[(0, 0)], "Thicker plate should be stiffer");
        assert!(k_thick[(1, 1)] > k_thin[(1, 1)], "Thicker plate should be stiffer");
    }

    #[test]
    fn bending_stiffness_positive_definite() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        let k_bend = shell
            .bending_stiffness(&nodes, &material)
            .expect("Should compute bending stiffness");

        // Check positive semi-definite
        // Bending stiffness has 3 rigid body modes (1 translation in z + 2 rotations about x, y)
        let eigen = k_bend.symmetric_eigen();
        let eigenvalues = eigen.eigenvalues;

        let mut positive_eigenvalues = 0;
        let mut near_zero_eigenvalues = 0;

        for &eig in eigenvalues.iter() {
            if eig > 1e-3 {
                positive_eigenvalues += 1;
            } else if eig > -1e-6 {
                near_zero_eigenvalues += 1;
            } else {
                panic!("Found negative eigenvalue: {}", eig);
            }
        }

        assert!(
            positive_eigenvalues >= 9,
            "Should have at least 9 positive eigenvalues, got {}",
            positive_eigenvalues
        );
        assert!(
            near_zero_eigenvalues <= 3,
            "Should have at most 3 near-zero eigenvalues (rigid body modes), got {}",
            near_zero_eigenvalues
        );
    }

    #[test]
    fn pressure_force_conservation() {
        // Test: Total nodal force = pressure × area
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes(); // 1×1 meter plate
        let pressure = 1000.0; // 1000 Pa

        let nodal_forces = shell
            .pressure_to_nodal_forces(&nodes, pressure)
            .expect("Should compute nodal forces");

        // Sum all nodal forces
        let mut total_force = Vector3::<f64>::zeros();
        for force_vec in &nodal_forces {
            total_force.x += force_vec[0]; // Sum Fx
            total_force.y += force_vec[1]; // Sum Fy
            total_force.z += force_vec[2]; // Sum Fz
        }

        // Expected total force = pressure × area
        let expected_total: f64 = pressure * 1.0; // 1.0 m²

        // Force should be in z-direction (normal to XY plate)
        let total_magnitude: f64 = total_force.norm();
        let error: f64 = (total_magnitude - expected_total).abs() / expected_total * 100.0;

        println!("Pressure force conservation:");
        println!("  Total force: ({}, {}, {})", total_force.x, total_force.y, total_force.z);
        println!("  Expected: {} N", expected_total);
        println!("  Actual: {} N", total_magnitude);
        println!("  Error: {:.4}%", error);

        assert!(
            error < 0.1,
            "Force conservation error should be < 0.1%, got {:.4}%",
            error
        );
    }

    #[test]
    fn pressure_force_direction() {
        // Test: Forces should be perpendicular to surface
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes(); // Plate in XY plane
        let pressure = 1000.0;

        let nodal_forces = shell
            .pressure_to_nodal_forces(&nodes, pressure)
            .expect("Should compute nodal forces");

        // For flat plate in XY plane, force should be in +Z direction
        let surface_normal = shell.surface_normal(&nodes).expect("Should get normal");

        // Check that force is parallel to normal
        for (i, force_vec) in nodal_forces.iter().enumerate() {
            let force = Vector3::<f64>::new(force_vec[0], force_vec[1], force_vec[2]);
            let force_magnitude = force.norm();

            if force_magnitude > 1e-10 {
                // Normalize and check direction
                let force_dir = force / force_magnitude;
                let dot_product = force_dir.dot(&surface_normal);

                println!(
                    "Node {}: force direction = ({:.6}, {:.6}, {:.6}), dot with normal = {:.6}",
                    i + 1,
                    force_dir.x,
                    force_dir.y,
                    force_dir.z,
                    dot_product
                );

                assert!(
                    dot_product > 0.999,
                    "Force at node {} should be parallel to surface normal, got dot = {}",
                    i + 1,
                    dot_product
                );
            }
        }
    }

    #[test]
    fn pressure_uniform_distribution() {
        // Test: Flat plate → equal forces at all nodes
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes(); // Symmetric 1×1 plate
        let pressure = 1000.0;

        let nodal_forces = shell
            .pressure_to_nodal_forces(&nodes, pressure)
            .expect("Should compute nodal forces");

        // For uniform pressure on flat symmetric plate, all nodal forces should be equal
        let f0_magnitude = Vector3::<f64>::new(nodal_forces[0][0], nodal_forces[0][1], nodal_forces[0][2]).norm();

        for (i, force_vec) in nodal_forces.iter().enumerate().skip(1) {
            let fi_magnitude = Vector3::<f64>::new(force_vec[0], force_vec[1], force_vec[2]).norm();
            let diff = (fi_magnitude - f0_magnitude).abs() / f0_magnitude * 100.0;

            println!(
                "Node {}: force magnitude = {:.6}, diff from node 1 = {:.4}%",
                i + 1,
                fi_magnitude,
                diff
            );

            assert!(
                diff < 0.01,
                "Node {} force should equal node 1 force (diff = {:.4}%)",
                i + 1,
                diff
            );
        }

        // All moments should be zero for uniform pressure
        for (i, force_vec) in nodal_forces.iter().enumerate() {
            for j in 3..6 {
                assert!(
                    force_vec[j].abs() < 1e-10,
                    "Node {} moment DOF {} should be zero, got {}",
                    i + 1,
                    j,
                    force_vec[j]
                );
            }
        }
    }

    // ========== Mass Matrix Tests ==========

    #[test]
    fn mass_matrix_requires_density() {
        let shell = S4::new(
            1,
            vec![1, 2, 3, 4],
            ShellSection::new(0.01),
        );
        let nodes = make_square_plate_nodes();

        let mut material = make_steel_material();
        material.density = None; // Missing density

        let result = shell.mass_matrix(&nodes, &material);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("density"));
    }

    #[test]
    fn mass_matrix_is_symmetric() {
        let shell = S4::new(
            1,
            vec![1, 2, 3, 4],
            ShellSection::new(0.01),
        );
        let nodes = make_square_plate_nodes();
        let mut material = make_steel_material();
        material.density = Some(7850.0); // kg/m³

        let m = shell.mass_matrix(&nodes, &material).unwrap();

        // Check symmetry: M[i,j] == M[j,i]
        for i in 0..24 {
            for j in 0..24 {
                let error = (m[(i, j)] - m[(j, i)]).abs();
                assert!(
                    error < 1e-10,
                    "Mass matrix not symmetric at ({}, {}): {} vs {}",
                    i,
                    j,
                    m[(i, j)],
                    m[(j, i)]
                );
            }
        }
    }

    #[test]
    fn mass_matrix_has_positive_diagonals() {
        let shell = S4::new(
            1,
            vec![1, 2, 3, 4],
            ShellSection::new(0.01),
        );
        let nodes = make_square_plate_nodes();
        let mut material = make_steel_material();
        material.density = Some(7850.0); // kg/m³

        let m = shell.mass_matrix(&nodes, &material).unwrap();

        // Check all diagonal entries are positive
        for i in 0..24 {
            assert!(
                m[(i, i)] > 0.0,
                "Diagonal entry M[{}, {}] = {} should be positive",
                i,
                i,
                m[(i, i)]
            );
        }
    }

    #[test]
    fn mass_matrix_dimensions() {
        let shell = S4::new(
            1,
            vec![1, 2, 3, 4],
            ShellSection::new(0.01),
        );
        let nodes = make_square_plate_nodes();
        let mut material = make_steel_material();
        material.density = Some(7850.0); // kg/m³

        let m = shell.mass_matrix(&nodes, &material).unwrap();

        assert_eq!(m.nrows(), 24, "Mass matrix should be 24×24");
        assert_eq!(m.ncols(), 24, "Mass matrix should be 24×24");
    }

    #[test]
    fn mass_matrix_conserves_translational_mass() {
        // Test: Total translational mass should equal ρ*t*A
        let thickness = 0.01; // m
        let shell = S4::new(
            1,
            vec![1, 2, 3, 4],
            ShellSection::new(thickness),
        );
        let nodes = make_square_plate_nodes(); // 1m × 1m plate
        let mut material = make_steel_material();
        let density = 7850.0; // kg/m³
        material.density = Some(density);

        let m = shell.mass_matrix(&nodes, &material).unwrap();

        // Extract translational mass for x-direction (DOFs 0, 6, 12, 18)
        // Sum all entries in these rows
        let mut trans_mass_sum = 0.0;
        for i in [0, 6, 12, 18] {
            for j in 0..24 {
                trans_mass_sum += m[(i, j)];
            }
        }

        // Expected total mass: ρ * t * A = 7850 * 0.01 * 1.0 = 78.5 kg
        let expected_mass = density * thickness * 1.0;

        let error = (trans_mass_sum - expected_mass).abs();
        let relative_error = error / expected_mass;

        assert!(
            relative_error < 1e-6,
            "Translational mass conservation error: {:.2e}% (expected 0%)",
            relative_error * 100.0
        );
    }

    #[test]
    fn mass_matrix_rotational_inertia() {
        // Test: Rotational mass should be proportional to t³/12
        let thickness = 0.01; // m
        let shell = S4::new(
            1,
            vec![1, 2, 3, 4],
            ShellSection::new(thickness),
        );
        let nodes = make_square_plate_nodes();
        let mut material = make_steel_material();
        material.density = Some(7850.0); // kg/m³

        let m = shell.mass_matrix(&nodes, &material).unwrap();

        // Rotational DOFs (3, 4, 5, 9, 10, 11, 15, 16, 17, 21, 22, 23)
        // should have mass proportional to t³/12 vs translational mass with t

        // Get a diagonal translational mass (node 1, x-direction)
        let trans_mass = m[(0, 0)];

        // Get a diagonal rotational mass (node 1, θx)
        let rot_mass = m[(3, 3)];

        // Ratio should be approximately (t³/12) / t = t²/12
        let expected_ratio = thickness * thickness / 12.0;
        let actual_ratio = rot_mass / trans_mass;
        let relative_error = (actual_ratio - expected_ratio).abs() / expected_ratio;

        assert!(
            relative_error < 0.1,
            "Rotational/translational mass ratio error: {:.2e}%",
            relative_error * 100.0
        );
    }

    #[test]
    fn mass_matrix_symmetry_for_square_plate() {
        // Test: For a square plate, all nodes should have equal diagonal mass
        let shell = S4::new(
            1,
            vec![1, 2, 3, 4],
            ShellSection::new(0.01),
        );
        let nodes = make_square_plate_nodes();
        let mut material = make_steel_material();
        material.density = Some(7850.0); // kg/m³

        let m = shell.mass_matrix(&nodes, &material).unwrap();

        // Check that all nodes have equal translational mass (x-direction)
        let m00 = m[(0, 0)];   // Node 1 x-translation
        let m66 = m[(6, 6)];   // Node 2 x-translation
        let m1212 = m[(12, 12)]; // Node 3 x-translation
        let m1818 = m[(18, 18)]; // Node 4 x-translation

        let tolerance = 1e-6;
        assert!(
            (m00 - m66).abs() < tolerance,
            "Nodes 1 and 2 should have equal mass"
        );
        assert!(
            (m00 - m1212).abs() < tolerance,
            "Nodes 1 and 3 should have equal mass"
        );
        assert!(
            (m00 - m1818).abs() < tolerance,
            "Nodes 1 and 4 should have equal mass"
        );
    }
}
