//! Builder for extracting boundary conditions from input decks.

use crate::boundary_conditions::{BoundaryConditions, ConcentratedLoad, DisplacementBC};
use crate::sets::Sets;
use ccx_io::inp::{Card, Deck};

/// Builds boundary conditions from a parsed input deck
pub struct BCBuilder {
    bcs: BoundaryConditions,
    sets: Sets,
    errors: Vec<String>,
}

impl BCBuilder {
    /// Create a new BC builder
    pub fn new() -> Self {
        Self {
            bcs: BoundaryConditions::new(),
            sets: Sets::new(),
            errors: Vec::new(),
        }
    }

    /// Build boundary conditions from the given deck
    pub fn build_from_deck(deck: &Deck) -> Result<BoundaryConditions, String> {
        // First build sets
        let sets = Sets::build_from_deck(deck)?;

        let mut builder = Self {
            bcs: BoundaryConditions::new(),
            sets,
            errors: Vec::new(),
        };
        builder.process_deck(deck)?;
        Ok(builder.bcs)
    }

    /// Process all cards in the deck
    fn process_deck(&mut self, deck: &Deck) -> Result<(), String> {
        for card in &deck.cards {
            match card.keyword.to_uppercase().as_str() {
                "BOUNDARY" => self.process_boundary_card(card)?,
                "CLOAD" => self.process_cload_card(card)?,
                // TODO: Add DLOAD, TEMPERATURE, etc.
                _ => {} // Ignore other keywords
            }
        }

        if !self.errors.is_empty() {
            return Err(format!(
                "BC building encountered {} errors:\n{}",
                self.errors.len(),
                self.errors.join("\n")
            ));
        }

        Ok(())
    }

    /// Process a *BOUNDARY card
    fn process_boundary_card(&mut self, card: &Card) -> Result<(), String> {
        for data_line in &card.data_lines {
            let parts: Vec<&str> = data_line.split(',').collect();

            if parts.len() < 2 {
                self.errors.push(format!(
                    "Invalid BOUNDARY line (expected at least 2 fields): {}",
                    data_line
                ));
                continue;
            }

            // Parse node ID or node set name
            let node_str = parts[0].trim();
            let nodes: Vec<i32> = match node_str.parse::<i32>() {
                Ok(n) => vec![n],
                Err(_) => {
                    // Try to resolve as node set
                    match self.sets.get_nodes(node_str) {
                        Some(set_nodes) => set_nodes.to_vec(),
                        None => {
                            self.errors.push(format!(
                                "Unknown node or node set in BOUNDARY: {}",
                                node_str
                            ));
                            continue;
                        }
                    }
                }
            };

            // Parse first DOF
            let first_dof = match parts[1].trim().parse::<usize>() {
                Ok(d) => d,
                Err(_) => {
                    self.errors.push(format!(
                        "Invalid first DOF in BOUNDARY: {}",
                        parts[1].trim()
                    ));
                    continue;
                }
            };

            // Parse last DOF (default to first_dof if not specified)
            let last_dof = if parts.len() >= 3 && !parts[2].trim().is_empty() {
                match parts[2].trim().parse::<usize>() {
                    Ok(d) => d,
                    Err(_) => {
                        self.errors
                            .push(format!("Invalid last DOF in BOUNDARY: {}", parts[2].trim()));
                        continue;
                    }
                }
            } else {
                first_dof
            };

            // Parse prescribed value (default to 0.0)
            let value = if parts.len() >= 4 && !parts[3].trim().is_empty() {
                match parts[3].trim().parse::<f64>() {
                    Ok(v) => v,
                    Err(_) => {
                        self.errors
                            .push(format!("Invalid value in BOUNDARY: {}", parts[3].trim()));
                        continue;
                    }
                }
            } else {
                0.0
            };

            // Apply BC to all nodes in the set
            for node in nodes {
                let bc = DisplacementBC::new(node, first_dof, last_dof, value);
                self.bcs.add_displacement_bc(bc);
            }
        }

        Ok(())
    }

    /// Process a *CLOAD card
    fn process_cload_card(&mut self, card: &Card) -> Result<(), String> {
        for data_line in &card.data_lines {
            let parts: Vec<&str> = data_line.split(',').collect();

            if parts.len() < 3 {
                self.errors.push(format!(
                    "Invalid CLOAD line (expected at least 3 fields): {}",
                    data_line
                ));
                continue;
            }

            // Parse node ID or node set name
            let node_str = parts[0].trim();
            let nodes: Vec<i32> = match node_str.parse::<i32>() {
                Ok(n) => vec![n],
                Err(_) => {
                    // Try to resolve as node set
                    match self.sets.get_nodes(node_str) {
                        Some(set_nodes) => set_nodes.to_vec(),
                        None => {
                            self.errors
                                .push(format!("Unknown node or node set in CLOAD: {}", node_str));
                            continue;
                        }
                    }
                }
            };

            // Parse DOF
            let dof = match parts[1].trim().parse::<usize>() {
                Ok(d) => d,
                Err(_) => {
                    self.errors
                        .push(format!("Invalid DOF in CLOAD: {}", parts[1].trim()));
                    continue;
                }
            };

            // Parse magnitude
            let magnitude = match parts[2].trim().parse::<f64>() {
                Ok(m) => m,
                Err(_) => {
                    self.errors
                        .push(format!("Invalid magnitude in CLOAD: {}", parts[2].trim()));
                    continue;
                }
            };

            // Apply load to all nodes in the set
            for node in nodes {
                let load = ConcentratedLoad::new(node, dof, magnitude);
                self.bcs.add_concentrated_load(load);
            }
        }

        Ok(())
    }

    /// Get reference to the built boundary conditions
    pub fn bcs(&self) -> &BoundaryConditions {
        &self.bcs
    }

    /// Take ownership of the built boundary conditions
    pub fn into_bcs(self) -> BoundaryConditions {
        self.bcs
    }
}

impl Default for BCBuilder {
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
    fn parses_simple_boundary_conditions() {
        let input = r#"
*NODE
1, 0.0, 0.0, 0.0
2, 1.0, 0.0, 0.0
*BOUNDARY
1, 1, 3, 0.0
2, 3, 3
"#;

        let deck = parse_deck(input);
        let bcs = BCBuilder::build_from_deck(&deck).expect("Failed to build BCs");

        assert_eq!(bcs.displacement_bcs.len(), 2);

        let bc1 = &bcs.displacement_bcs[0];
        assert_eq!(bc1.node, 1);
        assert_eq!(bc1.first_dof, 1);
        assert_eq!(bc1.last_dof, 3);
        assert_eq!(bc1.value, 0.0);

        let bc2 = &bcs.displacement_bcs[1];
        assert_eq!(bc2.node, 2);
        assert_eq!(bc2.first_dof, 3);
        assert_eq!(bc2.last_dof, 3);
        assert_eq!(bc2.value, 0.0);
    }

    #[test]
    fn parses_concentrated_loads() {
        let input = r#"
*NODE
1, 0.0, 0.0, 0.0
2, 1.0, 0.0, 0.0
*CLOAD
1, 1, 100.0
2, 2, -50.5
"#;

        let deck = parse_deck(input);
        let bcs = BCBuilder::build_from_deck(&deck).expect("Failed to build BCs");

        assert_eq!(bcs.concentrated_loads.len(), 2);

        let load1 = &bcs.concentrated_loads[0];
        assert_eq!(load1.node, 1);
        assert_eq!(load1.dof, 1);
        assert_eq!(load1.magnitude, 100.0);

        let load2 = &bcs.concentrated_loads[1];
        assert_eq!(load2.node, 2);
        assert_eq!(load2.dof, 2);
        assert_eq!(load2.magnitude, -50.5);
    }

    #[test]
    fn handles_boundary_with_prescribed_displacement() {
        let input = r#"
*NODE
10, 0.0, 0.0, 0.0
*BOUNDARY
10, 1, 1, 2.5
"#;

        let deck = parse_deck(input);
        let bcs = BCBuilder::build_from_deck(&deck).expect("Failed to build BCs");

        assert_eq!(bcs.displacement_bcs.len(), 1);

        let bc = &bcs.displacement_bcs[0];
        assert_eq!(bc.node, 10);
        assert_eq!(bc.value, 2.5);
    }

    #[test]
    fn handles_boundary_with_default_value() {
        let input = r#"
*NODE
5, 0.0, 0.0, 0.0
*BOUNDARY
5, 2, 2
"#;

        let deck = parse_deck(input);
        let bcs = BCBuilder::build_from_deck(&deck).expect("Failed to build BCs");

        assert_eq!(bcs.displacement_bcs.len(), 1);

        let bc = &bcs.displacement_bcs[0];
        assert_eq!(bc.value, 0.0); // Default value
    }

    #[test]
    fn handles_mixed_bcs_and_loads() {
        let input = r#"
*NODE
1, 0.0, 0.0, 0.0
2, 1.0, 0.0, 0.0
3, 0.5, 1.0, 0.0
*BOUNDARY
1, 1, 3
*CLOAD
2, 2, 1000.0
3, 1, 500.0
"#;

        let deck = parse_deck(input);
        let bcs = BCBuilder::build_from_deck(&deck).expect("Failed to build BCs");

        assert_eq!(bcs.displacement_bcs.len(), 1);
        assert_eq!(bcs.concentrated_loads.len(), 2);
    }

    #[test]
    fn ignores_non_bc_cards() {
        let input = r#"
*NODE
1, 0.0, 0.0, 0.0
*MATERIAL, NAME=STEEL
*ELASTIC
210000, 0.3
*BOUNDARY
1, 1, 3
*STEP
*STATIC
*END STEP
"#;

        let deck = parse_deck(input);
        let bcs = BCBuilder::build_from_deck(&deck).expect("Failed to build BCs");

        assert_eq!(bcs.displacement_bcs.len(), 1);
        assert_eq!(bcs.concentrated_loads.len(), 0);
    }

    #[test]
    fn resolves_node_set_references() {
        // Node sets are now resolved properly
        let input = r#"
*NODE
1, 0, 0, 0
2, 1, 0, 0
3, 2, 0, 0
*NSET, NSET=Nfixz
1, 2
*NSET, NSET=LoadSet
3
*BOUNDARY
Nfixz, 1, 3
*CLOAD
LoadSet, 1, 100.0
"#;

        let deck = parse_deck(input);
        let result = BCBuilder::build_from_deck(&deck);

        assert!(result.is_ok());
        let bcs = result.unwrap();
        // Should have 2 BCs (one for each node in Nfixz)
        assert_eq!(bcs.displacement_bcs.len(), 2);
        // Should have 1 load (for the single node in LoadSet)
        assert_eq!(bcs.concentrated_loads.len(), 1);
    }

    #[test]
    fn handles_scientific_notation_in_loads() {
        let input = r#"
*NODE
1, 0.0, 0.0, 0.0
*CLOAD
1, 1, 1.5e3
"#;

        let deck = parse_deck(input);
        let bcs = BCBuilder::build_from_deck(&deck).expect("Failed to build BCs");

        assert_eq!(bcs.concentrated_loads.len(), 1);
        assert!((bcs.concentrated_loads[0].magnitude - 1500.0).abs() < 1e-10);
    }
}
