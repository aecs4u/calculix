//! Mesh data structures for finite element analysis.
//!
//! This module provides the core data structures for representing FEA meshes:
//! nodes, elements, and connectivity information.

use std::collections::HashMap;

/// A node in the finite element mesh
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Node ID (1-based indexing from input file)
    pub id: i32,
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
    /// Z coordinate
    pub z: f64,
}

impl Node {
    /// Create a new node
    pub fn new(id: i32, x: f64, y: f64, z: f64) -> Self {
        Self { id, x, y, z }
    }

    /// Get coordinates as an array
    pub fn coords(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}

/// Element type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementType {
    /// 2-node truss element (T3D2)
    T3D2,
    /// 3-node truss element (T3D3)
    T3D3,
    /// 8-node brick element (C3D8)
    C3D8,
    /// 20-node brick element (C3D20)
    C3D20,
    /// 4-node tetrahedral element (C3D4)
    C3D4,
    /// 10-node tetrahedral element (C3D10)
    C3D10,
    /// 6-node wedge element (C3D6)
    C3D6,
    /// 15-node wedge element (C3D15)
    C3D15,
    /// 4-node shell element (S4)
    S4,
    /// 8-node shell element (S8)
    S8,
    /// 3-node shell element (S3)
    S3,
    /// 6-node shell element (S6)
    S6,
    /// 2-node beam element (B31)
    B31,
    /// 3-node beam element (B32)
    B32,
    /// 4-node membrane element (M3D4)
    M3D4,
    /// 8-node membrane element (M3D8)
    M3D8,
    /// 3-node membrane element (M3D3)
    M3D3,
    /// 6-node membrane element (M3D6)
    M3D6,
}

impl ElementType {
    /// Get the number of nodes for this element type
    pub fn num_nodes(&self) -> usize {
        match self {
            ElementType::T3D2 => 2,
            ElementType::T3D3 => 3,
            ElementType::C3D8 => 8,
            ElementType::C3D20 => 20,
            ElementType::C3D4 => 4,
            ElementType::C3D10 => 10,
            ElementType::C3D6 => 6,
            ElementType::C3D15 => 15,
            ElementType::S4 => 4,
            ElementType::S8 => 8,
            ElementType::S3 => 3,
            ElementType::S6 => 6,
            ElementType::B31 => 2,
            ElementType::B32 => 3,
            ElementType::M3D4 => 4,
            ElementType::M3D8 => 8,
            ElementType::M3D3 => 3,
            ElementType::M3D6 => 6,
        }
    }

    /// Get the number of degrees of freedom per node for this element type
    pub fn dofs_per_node(&self) -> usize {
        match self {
            // Truss elements: 3 translational DOFs
            ElementType::T3D2 | ElementType::T3D3 => 3,

            // 3D solid elements: 3 translational DOFs
            ElementType::C3D8 | ElementType::C3D20 |
            ElementType::C3D4 | ElementType::C3D10 |
            ElementType::C3D6 | ElementType::C3D15 => 3,

            // Shell elements: 6 DOFs (3 translation + 3 rotation)
            ElementType::S4 | ElementType::S8 |
            ElementType::S3 | ElementType::S6 => 6,

            // Beam elements: 6 DOFs (3 translation + 3 rotation)
            ElementType::B31 | ElementType::B32 => 6,

            // Membrane elements: 3 translational DOFs
            ElementType::M3D4 | ElementType::M3D8 |
            ElementType::M3D3 | ElementType::M3D6 => 3,
        }
    }

    /// Parse element type from CalculiX type string
    pub fn from_calculix_type(type_str: &str) -> Option<Self> {
        let type_upper = type_str.to_uppercase();
        match type_upper.as_str() {
            "T3D2" => Some(ElementType::T3D2),
            "C3D8" | "C3D8R" | "C3D8I" => Some(ElementType::C3D8),
            "C3D20" | "C3D20R" => Some(ElementType::C3D20),
            "C3D4" => Some(ElementType::C3D4),
            "C3D10" | "C3D10T" => Some(ElementType::C3D10),
            "C3D6" => Some(ElementType::C3D6),
            "C3D15" => Some(ElementType::C3D15),
            "S4" | "S4R" => Some(ElementType::S4),
            "S8" | "S8R" => Some(ElementType::S8),
            "S3" | "S3R" => Some(ElementType::S3),
            "S6" => Some(ElementType::S6),
            "B31" | "B31R" => Some(ElementType::B31),
            "B32" | "B32R" => Some(ElementType::B32),
            "M3D4" | "M3D4R" => Some(ElementType::M3D4),
            "M3D8" | "M3D8R" => Some(ElementType::M3D8),
            "M3D3" => Some(ElementType::M3D3),
            "M3D6" => Some(ElementType::M3D6),
            _ => None,
        }
    }
}

/// An element in the finite element mesh
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    /// Element ID (1-based indexing from input file)
    pub id: i32,
    /// Element type
    pub element_type: ElementType,
    /// Node connectivity (node IDs)
    pub nodes: Vec<i32>,
}

impl Element {
    /// Create a new element
    pub fn new(id: i32, element_type: ElementType, nodes: Vec<i32>) -> Self {
        Self {
            id,
            element_type,
            nodes,
        }
    }

    /// Validate that the element has the correct number of nodes
    pub fn validate(&self) -> Result<(), String> {
        let expected = self.element_type.num_nodes();
        let actual = self.nodes.len();
        if actual != expected {
            return Err(format!(
                "Element {} of type {:?} has {} nodes but expected {}",
                self.id, self.element_type, actual, expected
            ));
        }
        Ok(())
    }
}

/// Complete finite element mesh
#[derive(Debug, Clone)]
pub struct Mesh {
    /// All nodes in the mesh, indexed by node ID
    pub nodes: HashMap<i32, Node>,
    /// All elements in the mesh, indexed by element ID
    pub elements: HashMap<i32, Element>,
    /// Total number of degrees of freedom
    pub num_dofs: usize,
}

impl Mesh {
    /// Create a new empty mesh
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            elements: HashMap::new(),
            num_dofs: 0,
        }
    }

    /// Add a node to the mesh
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    /// Add an element to the mesh
    pub fn add_element(&mut self, element: Element) -> Result<(), String> {
        element.validate()?;
        self.elements.insert(element.id, element);
        Ok(())
    }

    /// Get a node by ID
    pub fn get_node(&self, id: i32) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Get an element by ID
    pub fn get_element(&self, id: i32) -> Option<&Element> {
        self.elements.get(&id)
    }

    /// Calculate total degrees of freedom (assumes 3 DOF per node)
    pub fn calculate_dofs(&mut self) {
        // For now, assume 3 DOF per node (structural analysis)
        // TODO: Handle other DOF configurations (thermal, etc.)
        self.num_dofs = self.nodes.len() * 3;
    }

    /// Validate the mesh
    pub fn validate(&self) -> Result<(), String> {
        // Check that all element nodes exist
        for (elem_id, element) in &self.elements {
            for &node_id in &element.nodes {
                if !self.nodes.contains_key(&node_id) {
                    return Err(format!(
                        "Element {} references non-existent node {}",
                        elem_id, node_id
                    ));
                }
            }
        }
        Ok(())
    }

    /// Get mesh statistics
    pub fn statistics(&self) -> MeshStatistics {
        let mut element_type_counts = HashMap::new();
        for element in self.elements.values() {
            *element_type_counts.entry(element.element_type).or_insert(0) += 1;
        }

        MeshStatistics {
            num_nodes: self.nodes.len(),
            num_elements: self.elements.len(),
            num_dofs: self.num_dofs,
            element_type_counts,
        }
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Mesh statistics for reporting
#[derive(Debug, Clone)]
pub struct MeshStatistics {
    /// Total number of nodes
    pub num_nodes: usize,
    /// Total number of elements
    pub num_elements: usize,
    /// Total degrees of freedom
    pub num_dofs: usize,
    /// Count of each element type
    pub element_type_counts: HashMap<ElementType, usize>,
}

impl MeshStatistics {
    /// Format as a human-readable string
    pub fn format(&self) -> String {
        let mut lines = vec![
            format!("Nodes: {}", self.num_nodes),
            format!("Elements: {}", self.num_elements),
            format!("DOFs: {}", self.num_dofs),
        ];

        if !self.element_type_counts.is_empty() {
            lines.push("Element types:".to_string());
            let mut types: Vec<_> = self.element_type_counts.iter().collect();
            types.sort_by_key(|(k, _)| format!("{:?}", k));
            for (elem_type, count) in types {
                lines.push(format!("  {:?}: {}", elem_type, count));
            }
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_creation() {
        let node = Node::new(1, 0.0, 0.0, 0.0);
        assert_eq!(node.id, 1);
        assert_eq!(node.coords(), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn element_type_num_nodes() {
        assert_eq!(ElementType::C3D8.num_nodes(), 8);
        assert_eq!(ElementType::C3D20.num_nodes(), 20);
        assert_eq!(ElementType::C3D4.num_nodes(), 4);
        assert_eq!(ElementType::B31.num_nodes(), 2);
    }

    #[test]
    fn element_type_parsing() {
        assert_eq!(
            ElementType::from_calculix_type("C3D8"),
            Some(ElementType::C3D8)
        );
        assert_eq!(
            ElementType::from_calculix_type("c3d8"),
            Some(ElementType::C3D8)
        );
        assert_eq!(
            ElementType::from_calculix_type("C3D8R"),
            Some(ElementType::C3D8)
        );
        assert_eq!(ElementType::from_calculix_type("INVALID"), None);
    }

    #[test]
    fn element_validation() {
        let elem = Element::new(1, ElementType::C3D8, vec![1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(elem.validate().is_ok());

        let bad_elem = Element::new(2, ElementType::C3D8, vec![1, 2, 3]);
        assert!(bad_elem.validate().is_err());
    }

    #[test]
    fn mesh_add_nodes_and_elements() {
        let mut mesh = Mesh::new();

        mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
        mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));

        assert_eq!(mesh.nodes.len(), 2);
        assert!(mesh.get_node(1).is_some());
        assert!(mesh.get_node(3).is_none());
    }

    #[test]
    fn mesh_validates_element_nodes() {
        let mut mesh = Mesh::new();

        mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
        mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));

        // Element references non-existent node 3
        let elem = Element::new(1, ElementType::B31, vec![1, 3]);
        mesh.add_element(elem).unwrap();

        let result = mesh.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("non-existent node 3"));
    }

    #[test]
    fn mesh_calculates_dofs() {
        let mut mesh = Mesh::new();

        mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
        mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));
        mesh.add_node(Node::new(3, 0.5, 1.0, 0.0));

        mesh.calculate_dofs();
        assert_eq!(mesh.num_dofs, 9); // 3 nodes * 3 DOF
    }

    #[test]
    fn mesh_statistics() {
        let mut mesh = Mesh::new();

        for i in 1..=8 {
            mesh.add_node(Node::new(i, 0.0, 0.0, 0.0));
        }

        let elem1 = Element::new(1, ElementType::C3D8, vec![1, 2, 3, 4, 5, 6, 7, 8]);
        mesh.add_element(elem1).unwrap();

        mesh.calculate_dofs();

        let stats = mesh.statistics();
        assert_eq!(stats.num_nodes, 8);
        assert_eq!(stats.num_elements, 1);
        assert_eq!(stats.num_dofs, 24);
        assert_eq!(stats.element_type_counts.get(&ElementType::C3D8), Some(&1));
    }
}
