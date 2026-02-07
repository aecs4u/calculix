//! Minimal CalculiX/Abaqus `.inp` deck parser for migration bootstrap.

use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deck {
    pub cards: Vec<Card>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub keyword: String,
    pub parameters: Vec<Parameter>,
    pub data_lines: Vec<String>,
    pub line_start: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

impl Deck {
    pub fn parse_file(path: impl AsRef<Path>) -> Result<Self, ParseError> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path).map_err(|e| ParseError {
            line: 0,
            message: format!("failed to read {}: {e}", path.display()),
        })?;
        Self::parse_str(&raw)
    }

    pub fn parse_str(raw: &str) -> Result<Self, ParseError> {
        let lines: Vec<&str> = raw.lines().collect();
        let mut cards = Vec::new();
        let mut i = 0usize;

        while i < lines.len() {
            let trimmed = lines[i].trim();

            if trimmed.is_empty() || is_comment(trimmed) {
                i += 1;
                continue;
            }

            if !trimmed.starts_with('*') {
                return Err(ParseError {
                    line: i + 1,
                    message: "expected card starting with '*'".to_string(),
                });
            }

            let line_start = i + 1;
            let mut header = trimmed.trim_start_matches('*').trim().to_string();
            i += 1;

            // Support basic Abaqus-style header continuation with leading comma.
            while i < lines.len() {
                let next = lines[i].trim();
                if next.starts_with(',') {
                    header.push_str(next);
                    i += 1;
                    continue;
                }
                break;
            }

            let (keyword, parameters) = parse_header(&header, line_start)?;

            let mut data_lines = Vec::new();
            while i < lines.len() {
                let candidate = lines[i].trim();
                if candidate.is_empty() || is_comment(candidate) {
                    i += 1;
                    continue;
                }
                if candidate.starts_with('*') {
                    break;
                }
                data_lines.push(candidate.to_string());
                i += 1;
            }

            cards.push(Card {
                keyword,
                parameters,
                data_lines,
                line_start,
            });
        }

        Ok(Deck { cards })
    }
}

fn is_comment(line: &str) -> bool {
    line.starts_with("**")
}

fn parse_header(header: &str, line: usize) -> Result<(String, Vec<Parameter>), ParseError> {
    let mut parts = header.split(',');
    let keyword_raw = parts.next().unwrap_or_default().trim();
    if keyword_raw.is_empty() {
        return Err(ParseError {
            line,
            message: "empty card keyword".to_string(),
        });
    }
    let keyword = keyword_raw.to_ascii_uppercase();
    let mut parameters = Vec::new();

    for part in parts {
        let item = part.trim();
        if item.is_empty() {
            continue;
        }
        if let Some((k, v)) = item.split_once('=') {
            parameters.push(Parameter {
                key: k.trim().to_ascii_uppercase(),
                value: Some(v.trim().to_string()),
            });
        } else {
            parameters.push(Parameter {
                key: item.to_ascii_uppercase(),
                value: None,
            });
        }
    }

    Ok((keyword, parameters))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_cards_and_data() {
        let src = r#"
** comment
*HEADING
My model
*NODE, NSET=NALL
1,0,0,0
2,1,0,0
*ELEMENT, TYPE=C3D8, ELSET=EALL
1,1,2,3,4,5,6,7,8
"#;

        let deck = Deck::parse_str(src).expect("parser should succeed");
        assert_eq!(deck.cards.len(), 3);
        assert_eq!(deck.cards[1].keyword, "NODE");
        assert_eq!(deck.cards[1].data_lines.len(), 2);
        assert_eq!(deck.cards[2].keyword, "ELEMENT");
    }

    #[test]
    fn parses_header_continuation() {
        let src = r#"
*STEP, INC=100
, NLGEOM
*STATIC
1., 1.
"#;

        let deck = Deck::parse_str(src).expect("parser should succeed");
        assert_eq!(deck.cards.len(), 2);
        assert_eq!(deck.cards[0].keyword, "STEP");
        assert!(
            deck.cards[0]
                .parameters
                .iter()
                .any(|p| p.key == "NLGEOM" && p.value.is_none())
        );
    }

    #[test]
    fn fails_on_orphan_data_before_first_card() {
        let src = "1,2,3\n*NODE\n1,0,0,0\n";
        let err = Deck::parse_str(src).expect_err("should fail");
        assert_eq!(err.line, 1);
    }
}

