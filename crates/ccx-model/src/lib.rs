//! Migration-stage domain summary extracted from a parsed deck.

use std::collections::BTreeMap;

use ccx_inp::{Card, Deck};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSummary {
    pub total_cards: usize,
    pub total_data_lines: usize,
    pub keyword_counts: BTreeMap<String, usize>,
    pub include_files: Vec<String>,
    pub material_defs: usize,
    pub node_rows: usize,
    pub element_rows: usize,
    pub has_step: bool,
    pub has_static: bool,
    pub has_dynamic: bool,
    pub has_frequency: bool,
    pub has_heat_transfer: bool,
}

impl ModelSummary {
    pub fn from_deck(deck: &Deck) -> Self {
        let mut keyword_counts = BTreeMap::<String, usize>::new();
        let mut include_files = Vec::<String>::new();
        let mut material_defs = 0usize;
        let mut node_rows = 0usize;
        let mut element_rows = 0usize;

        let mut has_step = false;
        let mut has_static = false;
        let mut has_dynamic = false;
        let mut has_frequency = false;
        let mut has_heat_transfer = false;

        for card in &deck.cards {
            *keyword_counts.entry(card.keyword.clone()).or_insert(0) += 1;

            match normalized(&card.keyword).as_str() {
                "STEP" => has_step = true,
                "STATIC" => has_static = true,
                "DYNAMIC" => has_dynamic = true,
                "FREQUENCY" => has_frequency = true,
                "HEATTRANSFER" => has_heat_transfer = true,
                "MATERIAL" => material_defs += 1,
                "NODE" => node_rows += card.data_lines.len(),
                "ELEMENT" => element_rows += card.data_lines.len(),
                "INCLUDE" => {
                    if let Some(input) = include_input(card) {
                        include_files.push(input);
                    }
                }
                _ => {}
            }
        }

        let total_cards = deck.cards.len();
        let total_data_lines = deck.cards.iter().map(|c| c.data_lines.len()).sum();

        Self {
            total_cards,
            total_data_lines,
            keyword_counts,
            include_files,
            material_defs,
            node_rows,
            element_rows,
            has_step,
            has_static,
            has_dynamic,
            has_frequency,
            has_heat_transfer,
        }
    }
}

fn include_input(card: &Card) -> Option<String> {
    card.parameters
        .iter()
        .find(|p| p.key == "INPUT")
        .and_then(|p| p.value.clone())
}

fn normalized(keyword: &str) -> String {
    keyword
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '_')
        .collect::<String>()
        .to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use ccx_inp::Deck;

    use super::ModelSummary;

    #[test]
    fn summarizes_common_analysis_cards() {
        let src = r#"
*INCLUDE, INPUT=mesh.msh
*NODE
1,0,0,0
*ELEMENT
1,1,2,3,4
*MATERIAL, NAME=STEEL
*STEP
*STATIC
1.,1.
*END STEP
"#;
        let deck = Deck::parse_str(src).expect("parse should succeed");
        let s = ModelSummary::from_deck(&deck);
        assert_eq!(s.total_cards, 7);
        assert_eq!(s.node_rows, 1);
        assert_eq!(s.element_rows, 1);
        assert_eq!(s.material_defs, 1);
        assert!(s.has_step);
        assert!(s.has_static);
        assert_eq!(s.include_files, vec!["mesh.msh".to_string()]);
    }
}

