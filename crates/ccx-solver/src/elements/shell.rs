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

    /// Placeholder for local stiffness matrix
    ///
    /// TODO: Implement membrane + bending + drilling stiffness
    fn local_stiffness(
        &self,
        _nodes: &[Node],
        _material: &Material,
    ) -> Result<SMatrix<f64, 24, 24>, String> {
        // Placeholder: return identity matrix
        Ok(SMatrix::<f64, 24, 24>::identity())
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
    fn stiffness_matrix_placeholder() {
        let section = ShellSection::new(0.01);
        let shell = S4::new(1, vec![1, 2, 3, 4], section);
        let nodes = make_square_plate_nodes();
        let material = make_steel_material();

        let k = shell
            .stiffness_matrix(&nodes, &material)
            .expect("Should compute stiffness");

        assert_eq!(k.nrows(), 24);
        assert_eq!(k.ncols(), 24);
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
}
