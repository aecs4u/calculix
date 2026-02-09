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

    /// Placeholder for transformation matrix (local → global)
    ///
    /// TODO: Implement coordinate transformation based on element orientation
    fn transformation_matrix(&self, _nodes: &[Node]) -> Result<DMatrix<f64>, String> {
        // Placeholder: return identity matrix
        Ok(DMatrix::identity(24, 24))
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
}
