//! Mesh builder for constructing finite element meshes from input decks.
//!
//! This module extracts node and element data from parsed CalculiX input decks
//! and constructs the Mesh data structure for solver processing.

use crate::mesh::{Element, ElementType, Mesh, Node};
use ccx_inp::{Card, Deck};

/// Builds a mesh from a parsed input deck
pub struct MeshBuilder {
    mesh: Mesh,
    errors: Vec<String>,
}

impl MeshBuilder {
    /// Create a new mesh builder
    pub fn new() -> Self {
        Self {
            mesh: Mesh::new(),
            errors: Vec::new(),
        }
    }

    /// Build a mesh from the given deck
    pub fn build_from_deck(deck: &Deck) -> Result<Mesh, String> {
        let mut builder = Self::new();
        builder.process_deck(deck)?;
        Ok(builder.mesh)
    }

    /// Process all cards in the deck
    fn process_deck(&mut self, deck: &Deck) -> Result<(), String> {
        for card in &deck.cards {
            match card.keyword.to_uppercase().as_str() {
                "NODE" => self.process_node_card(card)?,
                "ELEMENT" => self.process_element_card(card)?,
                _ => {} // Ignore other keywords for now
            }
        }

        // Validate the mesh after building
        self.mesh.validate()?;
        self.mesh.calculate_dofs();

        if !self.errors.is_empty() {
            return Err(format!(
                "Mesh building encountered {} errors:\n{}",
                self.errors.len(),
                self.errors.join("\n")
            ));
        }

        Ok(())
    }

    /// Process a *NODE card
    fn process_node_card(&mut self, card: &Card) -> Result<(), String> {
        for data_line in &card.data_lines {
            let parts: Vec<&str> = data_line.split(',').collect();

            if parts.len() < 4 {
                self.errors.push(format!(
                    "Invalid node data line (expected at least 4 fields): {}",
                    data_line
                ));
                continue;
            }

            // Parse node ID
            let id = match parts[0].trim().parse::<i32>() {
                Ok(id) => id,
                Err(_) => {
                    self.errors
                        .push(format!("Invalid node ID: {}", parts[0].trim()));
                    continue;
                }
            };

            // Parse coordinates
            let x = match parts[1].trim().parse::<f64>() {
                Ok(x) => x,
                Err(_) => {
                    self.errors.push(format!(
                        "Invalid X coordinate for node {}: {}",
                        id,
                        parts[1].trim()
                    ));
                    continue;
                }
            };

            let y = match parts[2].trim().parse::<f64>() {
                Ok(y) => y,
                Err(_) => {
                    self.errors.push(format!(
                        "Invalid Y coordinate for node {}: {}",
                        id,
                        parts[2].trim()
                    ));
                    continue;
                }
            };

            let z = match parts[3].trim().parse::<f64>() {
                Ok(z) => z,
                Err(_) => {
                    self.errors.push(format!(
                        "Invalid Z coordinate for node {}: {}",
                        id,
                        parts[3].trim()
                    ));
                    continue;
                }
            };

            let node = Node::new(id, x, y, z);
            self.mesh.add_node(node);
        }

        Ok(())
    }

    /// Process an *ELEMENT card
    fn process_element_card(&mut self, card: &Card) -> Result<(), String> {
        // Extract element type from parameters
        let type_param = card
            .parameters
            .iter()
            .find(|p| p.key.to_uppercase() == "TYPE")
            .ok_or_else(|| "ELEMENT card missing TYPE parameter".to_string())?;

        let type_value = type_param
            .value
            .as_ref()
            .ok_or_else(|| "ELEMENT TYPE parameter has no value".to_string())?;

        let element_type = ElementType::from_calculix_type(type_value)
            .ok_or_else(|| format!("Unknown element type: {}", type_value))?;

        let expected_nodes = element_type.num_nodes();

        // Process data lines, accumulating nodes for multi-line elements
        let mut current_element_id: Option<i32> = None;
        let mut accumulated_nodes = Vec::new();

        for data_line in &card.data_lines {
            let parts: Vec<&str> = data_line.split(',').collect();

            if parts.is_empty() {
                continue;
            }

            let first_field = parts[0].trim();

            // Determine if this is a new element or continuation
            // Rules:
            // 1. If we have a current element with fewer than expected nodes, this is a continuation
            // 2. Otherwise, try to start a new element
            let is_continuation = current_element_id.is_some()
                && accumulated_nodes.len() < expected_nodes
                && !accumulated_nodes.is_empty();

            if is_continuation {
                // Continuation line - all fields are node IDs
                for node_str in &parts {
                    let node_str = node_str.trim();
                    if node_str.is_empty() {
                        continue;
                    }

                    match node_str.parse::<i32>() {
                        Ok(node_id) => accumulated_nodes.push(node_id),
                        Err(_) => {
                            self.errors.push(format!(
                                "Invalid node ID in element {}: {}",
                                current_element_id.unwrap(),
                                node_str
                            ));
                        }
                    }
                }

                // Check if element is complete
                if accumulated_nodes.len() >= expected_nodes
                    && let Some(elem_id) = current_element_id.take()
                {
                    self.finish_element(elem_id, element_type, &accumulated_nodes, expected_nodes);
                    accumulated_nodes.clear();
                }
            } else {
                // Try to start a new element
                match first_field.parse::<i32>() {
                    Ok(id) => {
                        // Finish previous element if any
                        if let Some(elem_id) = current_element_id.take() {
                            self.finish_element(
                                elem_id,
                                element_type,
                                &accumulated_nodes,
                                expected_nodes,
                            );
                            accumulated_nodes.clear();
                        }

                        // Start new element
                        current_element_id = Some(id);

                        // Collect node IDs from remaining fields
                        for node_str in &parts[1..] {
                            let node_str = node_str.trim();
                            if node_str.is_empty() {
                                continue;
                            }

                            match node_str.parse::<i32>() {
                                Ok(node_id) => accumulated_nodes.push(node_id),
                                Err(_) => {
                                    self.errors.push(format!(
                                        "Invalid node ID in element {}: {}",
                                        id, node_str
                                    ));
                                }
                            }
                        }

                        // Check if element is complete (single-line element)
                        if accumulated_nodes.len() >= expected_nodes
                            && let Some(elem_id) = current_element_id.take()
                        {
                            self.finish_element(
                                elem_id,
                                element_type,
                                &accumulated_nodes,
                                expected_nodes,
                            );
                            accumulated_nodes.clear();
                        }
                    }
                    Err(_) => {
                        self.errors
                            .push(format!("Invalid element ID: {}", first_field));
                    }
                }
            }
        }

        // Finish the last element if any
        if let Some(elem_id) = current_element_id {
            self.finish_element(elem_id, element_type, &accumulated_nodes, expected_nodes);
        }

        Ok(())
    }

    /// Helper to finish building an element
    fn finish_element(
        &mut self,
        id: i32,
        element_type: ElementType,
        nodes: &[i32],
        expected_nodes: usize,
    ) {
        // Check if we have the expected number of nodes
        if nodes.len() != expected_nodes {
            self.errors.push(format!(
                "Element {} of type {:?} has {} nodes but expected {}",
                id,
                element_type,
                nodes.len(),
                expected_nodes
            ));
            return;
        }

        let element = Element::new(id, element_type, nodes.to_vec());
        if let Err(e) = self.mesh.add_element(element) {
            self.errors.push(e);
        }
    }

    /// Get reference to the built mesh
    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    /// Get mutable reference to the mesh
    pub fn mesh_mut(&mut self) -> &mut Mesh {
        &mut self.mesh
    }

    /// Take ownership of the built mesh
    pub fn into_mesh(self) -> Mesh {
        self.mesh
    }
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_deck(input: &str) -> Deck {
        Deck::parse_str(input).expect("Failed to parse deck")
    }

    #[test]
    fn builds_simple_mesh_with_nodes_and_elements() {
        let input = r#"
*NODE
1, 0.0, 0.0, 0.0
2, 1.0, 0.0, 0.0
3, 1.0, 1.0, 0.0
4, 0.0, 1.0, 0.0
5, 0.0, 0.0, 1.0
6, 1.0, 0.0, 1.0
7, 1.0, 1.0, 1.0
8, 0.0, 1.0, 1.0
*ELEMENT, TYPE=C3D8
1, 1, 2, 3, 4, 5, 6, 7, 8
"#;

        let deck = parse_deck(input);
        let mesh = MeshBuilder::build_from_deck(&deck).expect("Failed to build mesh");

        assert_eq!(mesh.nodes.len(), 8);
        assert_eq!(mesh.elements.len(), 1);
        assert_eq!(mesh.num_dofs, 8 * 3); // 8 nodes * 3 DOF

        let node1 = mesh.get_node(1).unwrap();
        assert_eq!(node1.coords(), [0.0, 0.0, 0.0]);

        let elem1 = mesh.get_element(1).unwrap();
        assert_eq!(elem1.element_type, ElementType::C3D8);
        assert_eq!(elem1.nodes.len(), 8);
    }

    #[test]
    fn handles_multiple_node_cards() {
        let input = r#"
*NODE
1, 0.0, 0.0, 0.0
2, 1.0, 0.0, 0.0
*NODE
3, 1.0, 1.0, 0.0
4, 0.0, 1.0, 0.0
"#;

        let deck = parse_deck(input);
        let mesh = MeshBuilder::build_from_deck(&deck).expect("Failed to build mesh");

        assert_eq!(mesh.nodes.len(), 4);
    }

    #[test]
    fn handles_multiple_element_types() {
        let input = r#"
*NODE
1, 0.0, 0.0, 0.0
2, 1.0, 0.0, 0.0
3, 0.5, 1.0, 0.0
4, 2.0, 0.0, 0.0
*ELEMENT, TYPE=B31
1, 1, 2
*ELEMENT, TYPE=B31
2, 2, 4
"#;

        let deck = parse_deck(input);
        let mesh = MeshBuilder::build_from_deck(&deck).expect("Failed to build mesh");

        assert_eq!(mesh.nodes.len(), 4);
        assert_eq!(mesh.elements.len(), 2);

        let elem1 = mesh.get_element(1).unwrap();
        assert_eq!(elem1.element_type, ElementType::B31);
        assert_eq!(elem1.nodes, vec![1, 2]);
    }

    #[test]
    fn rejects_elements_with_wrong_node_count() {
        let input = r#"
*NODE
1, 0.0, 0.0, 0.0
2, 1.0, 0.0, 0.0
3, 1.0, 1.0, 0.0
*ELEMENT, TYPE=C3D8
1, 1, 2, 3
"#;

        let deck = parse_deck(input);
        let result = MeshBuilder::build_from_deck(&deck);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected 8"));
    }

    #[test]
    fn rejects_elements_referencing_missing_nodes() {
        let input = r#"
*NODE
1, 0.0, 0.0, 0.0
2, 1.0, 0.0, 0.0
*ELEMENT, TYPE=B31
1, 1, 999
"#;

        let deck = parse_deck(input);
        let result = MeshBuilder::build_from_deck(&deck);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("non-existent node 999"));
    }

    #[test]
    fn handles_element_type_variants() {
        let input = r#"
*NODE
1, 0.0, 0.0, 0.0
2, 1.0, 0.0, 0.0
3, 1.0, 1.0, 0.0
4, 0.0, 1.0, 0.0
5, 0.0, 0.0, 1.0
6, 1.0, 0.0, 1.0
7, 1.0, 1.0, 1.0
8, 0.0, 1.0, 1.0
*ELEMENT, TYPE=C3D8R
1, 1, 2, 3, 4, 5, 6, 7, 8
"#;

        let deck = parse_deck(input);
        let mesh = MeshBuilder::build_from_deck(&deck).expect("Failed to build mesh");

        let elem = mesh.get_element(1).unwrap();
        assert_eq!(elem.element_type, ElementType::C3D8); // C3D8R maps to C3D8
    }

    #[test]
    fn ignores_non_mesh_cards() {
        let input = r#"
*NODE
1, 0.0, 0.0, 0.0
2, 1.0, 0.0, 0.0
*MATERIAL, NAME=STEEL
*ELASTIC
210000, 0.3
*ELEMENT, TYPE=B31
1, 1, 2
*STEP
*STATIC
*END STEP
"#;

        let deck = parse_deck(input);
        let mesh = MeshBuilder::build_from_deck(&deck).expect("Failed to build mesh");

        assert_eq!(mesh.nodes.len(), 2);
        assert_eq!(mesh.elements.len(), 1);
    }

    #[test]
    fn handles_negative_coordinates() {
        let input = r#"
*NODE
1, -1.5, -2.3, -0.5
2, 1.0, 0.0, 0.0
"#;

        let deck = parse_deck(input);
        let mesh = MeshBuilder::build_from_deck(&deck).expect("Failed to build mesh");

        let node1 = mesh.get_node(1).unwrap();
        assert_eq!(node1.coords(), [-1.5, -2.3, -0.5]);
    }

    #[test]
    fn handles_scientific_notation_in_coordinates() {
        let input = r#"
*NODE
1, 1.5e-3, -2.3E+2, 4.2e0
2, 1.0, 0.0, 0.0
"#;

        let deck = parse_deck(input);
        let mesh = MeshBuilder::build_from_deck(&deck).expect("Failed to build mesh");

        let node1 = mesh.get_node(1).unwrap();
        assert!((node1.x - 0.0015).abs() < 1e-10);
        assert!((node1.y + 230.0).abs() < 1e-10);
        assert!((node1.z - 4.2).abs() < 1e-10);
    }
}
