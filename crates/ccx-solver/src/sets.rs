//! Node sets and element sets for grouping entities.

use ccx_inp::{Card, Deck};
use std::collections::HashMap;

/// A named set of nodes
#[derive(Debug, Clone)]
pub struct NodeSet {
    /// Set name
    pub name: String,
    /// Node IDs in the set
    pub nodes: Vec<i32>,
}

/// A named set of elements
#[derive(Debug, Clone)]
pub struct ElementSet {
    /// Set name
    pub name: String,
    /// Element IDs in the set
    pub elements: Vec<i32>,
}

/// Collection of all sets in the model
#[derive(Debug, Clone)]
pub struct Sets {
    /// Node sets by name
    pub node_sets: HashMap<String, NodeSet>,
    /// Element sets by name
    pub element_sets: HashMap<String, ElementSet>,
}

impl Sets {
    /// Create an empty sets collection
    pub fn new() -> Self {
        Self {
            node_sets: HashMap::new(),
            element_sets: HashMap::new(),
        }
    }

    /// Add a node set
    pub fn add_node_set(&mut self, set: NodeSet) {
        self.node_sets.insert(set.name.clone(), set);
    }

    /// Add an element set
    pub fn add_element_set(&mut self, set: ElementSet) {
        self.element_sets.insert(set.name.clone(), set);
    }

    /// Get nodes from a node set by name
    pub fn get_nodes(&self, set_name: &str) -> Option<&[i32]> {
        self.node_sets.get(set_name).map(|s| s.nodes.as_slice())
    }

    /// Get elements from an element set by name
    pub fn get_elements(&self, set_name: &str) -> Option<&[i32]> {
        self.element_sets
            .get(set_name)
            .map(|s| s.elements.as_slice())
    }

    /// Build sets from a deck
    pub fn build_from_deck(deck: &Deck) -> Result<Self, String> {
        let mut sets = Self::new();

        for card in &deck.cards {
            match card.keyword.to_uppercase().as_str() {
                "NSET" => {
                    if let Some(nset) = Self::parse_nset(card)? {
                        sets.add_node_set(nset);
                    }
                }
                "ELSET" => {
                    if let Some(elset) = Self::parse_elset(card)? {
                        sets.add_element_set(elset);
                    }
                }
                _ => {}
            }
        }

        Ok(sets)
    }

    /// Parse a *NSET card
    fn parse_nset(card: &Card) -> Result<Option<NodeSet>, String> {
        // Get the NSET parameter
        let nset_param = card
            .parameters
            .iter()
            .find(|p| p.key.to_uppercase() == "NSET");

        let name = match nset_param {
            Some(p) => match &p.value {
                Some(v) => v.clone(),
                None => return Err("NSET parameter missing value".to_string()),
            },
            None => return Ok(None), // No NSET parameter, skip
        };

        let mut nodes = Vec::new();

        for data_line in &card.data_lines {
            for part in data_line.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }

                match part.parse::<i32>() {
                    Ok(node_id) => nodes.push(node_id),
                    Err(_) => {
                        return Err(format!("Invalid node ID in NSET {}: {}", name, part));
                    }
                }
            }
        }

        Ok(Some(NodeSet { name, nodes }))
    }

    /// Parse an *ELSET card
    fn parse_elset(card: &Card) -> Result<Option<ElementSet>, String> {
        // Get the ELSET parameter
        let elset_param = card
            .parameters
            .iter()
            .find(|p| p.key.to_uppercase() == "ELSET");

        let name = match elset_param {
            Some(p) => match &p.value {
                Some(v) => v.clone(),
                None => return Err("ELSET parameter missing value".to_string()),
            },
            None => return Ok(None), // No ELSET parameter, skip
        };

        let mut elements = Vec::new();

        for data_line in &card.data_lines {
            for part in data_line.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }

                match part.parse::<i32>() {
                    Ok(elem_id) => elements.push(elem_id),
                    Err(_) => {
                        return Err(format!("Invalid element ID in ELSET {}: {}", name, part));
                    }
                }
            }
        }

        Ok(Some(ElementSet { name, elements }))
    }
}

impl Default for Sets {
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
    fn parses_node_set() {
        let input = r#"
*NODE
1, 0, 0, 0
2, 1, 0, 0
3, 2, 0, 0
*NSET, NSET=FIXEDNODES
1, 2, 3
"#;

        let deck = parse_deck(input);
        let sets = Sets::build_from_deck(&deck).expect("Failed to build sets");

        assert_eq!(sets.node_sets.len(), 1);
        let nset = sets.node_sets.get("FIXEDNODES").unwrap();
        assert_eq!(nset.nodes, vec![1, 2, 3]);
    }

    #[test]
    fn parses_element_set() {
        let input = r#"
*ELEMENT, TYPE=C3D8, ELSET=ALLELEMS
1, 1, 2, 3, 4, 5, 6, 7, 8
*ELSET, ELSET=GROUP1
1
"#;

        let deck = parse_deck(input);
        let sets = Sets::build_from_deck(&deck).expect("Failed to build sets");

        assert_eq!(sets.element_sets.len(), 1);
        let elset = sets.element_sets.get("GROUP1").unwrap();
        assert_eq!(elset.elements, vec![1]);
    }

    #[test]
    fn handles_multi_line_sets() {
        let input = r#"
*NSET, NSET=ALLNODES
1, 2, 3,
4, 5, 6
"#;

        let deck = parse_deck(input);
        let sets = Sets::build_from_deck(&deck).expect("Failed to build sets");

        let nset = sets.node_sets.get("ALLNODES").unwrap();
        assert_eq!(nset.nodes, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn get_nodes_returns_node_ids() {
        let input = r#"
*NSET, NSET=TEST
10, 20, 30
"#;

        let deck = parse_deck(input);
        let sets = Sets::build_from_deck(&deck).expect("Failed to build sets");

        let nodes = sets.get_nodes("TEST").unwrap();
        assert_eq!(nodes, &[10, 20, 30]);
    }

    #[test]
    fn get_nodes_returns_none_for_missing_set() {
        let sets = Sets::new();
        assert!(sets.get_nodes("NONEXISTENT").is_none());
    }

    #[test]
    fn handles_element_set_from_element_card() {
        let input = r#"
*ELEMENT, TYPE=C3D8, ELSET=Eall
1, 1, 2, 3, 4, 5, 6, 7, 8
2, 9, 10, 11, 12, 13, 14, 15, 16
"#;

        let deck = parse_deck(input);
        let sets = Sets::build_from_deck(&deck).expect("Failed to build sets");

        // Element sets defined in ELEMENT cards are handled by mesh_builder
        // This test just ensures we don't error on them
        assert!(sets.element_sets.is_empty());
    }
}
