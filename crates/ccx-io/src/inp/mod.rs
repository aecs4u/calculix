//! Minimal CalculiX/Abaqus `.inp` deck parser for migration bootstrap.

use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

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
            if header.is_empty() {
                // Legacy decks sometimes use a bare "*" as a visual separator.
                continue;
            }

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

    pub fn parse_file_with_includes(path: impl AsRef<Path>) -> Result<Self, ParseError> {
        let mut include_stack = Vec::<PathBuf>::new();
        let mut active = HashSet::<PathBuf>::new();
        Self::parse_file_with_includes_inner(path.as_ref(), &mut include_stack, &mut active)
    }

    fn parse_file_with_includes_inner(
        path: &Path,
        include_stack: &mut Vec<PathBuf>,
        active: &mut HashSet<PathBuf>,
    ) -> Result<Self, ParseError> {
        let normalized_path = normalize_path(path);
        if active.contains(&normalized_path) {
            let mut chain = include_stack
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>();
            chain.push(normalized_path.display().to_string());
            return Err(ParseError {
                line: 0,
                message: format!("include cycle detected: {}", chain.join(" -> ")),
            });
        }

        include_stack.push(normalized_path.clone());
        active.insert(normalized_path);

        let result = (|| -> Result<Self, ParseError> {
            let raw = fs::read_to_string(path).map_err(|e| ParseError {
                line: 0,
                message: format!("failed to read {}: {e}", path.display()),
            })?;
            let parsed = Self::parse_str(&raw)?;
            let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
            let mut expanded_cards = Vec::<Card>::new();

            for card in parsed.cards {
                let include_target = if normalized_keyword(&card.keyword) == "INCLUDE" {
                    Some(include_input_path(&card).ok_or(ParseError {
                        line: card.line_start,
                        message: "missing INPUT parameter in *INCLUDE card".to_string(),
                    })?)
                } else {
                    None
                };

                expanded_cards.push(card);
                if let Some(raw_include) = include_target {
                    let include_path = resolve_include_path(base_dir, &raw_include);
                    let included =
                        Self::parse_file_with_includes_inner(&include_path, include_stack, active)
                            .map_err(|err| ParseError {
                                line: err.line,
                                message: format!(
                                    "{} (while expanding include {})",
                                    err.message,
                                    include_path.display()
                                ),
                            })?;
                    expanded_cards.extend(included.cards);
                }
            }

            Ok(Self {
                cards: expanded_cards,
            })
        })();

        let popped = include_stack.pop();
        if let Some(path) = popped {
            active.remove(&path);
        }

        result
    }
}

fn is_comment(line: &str) -> bool {
    // Some legacy fixtures prefix comment lines with `>`, e.g. `>** ...`.
    line.trim_start_matches('>').trim_start().starts_with("**")
}

fn parse_header(header: &str, line: usize) -> Result<(String, Vec<Parameter>), ParseError> {
    let fields = split_header_fields(header);
    let keyword_raw = fields.first().map(|s| s.as_str()).unwrap_or("").trim();
    if keyword_raw.is_empty() {
        return Err(ParseError {
            line,
            message: "empty card keyword".to_string(),
        });
    }
    let keyword = keyword_raw.to_ascii_uppercase();
    let mut parameters = Vec::new();

    for part in fields.iter().skip(1) {
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

fn split_header_fields(header: &str) -> Vec<String> {
    let mut fields = Vec::<String>::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;

    for ch in header.chars() {
        match ch {
            '\'' if !in_double => {
                in_single = !in_single;
                current.push(ch);
            }
            '"' if !in_single => {
                in_double = !in_double;
                current.push(ch);
            }
            ',' if !in_single && !in_double => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    fields.push(current.trim().to_string());
    fields
}

fn include_input_path(card: &Card) -> Option<String> {
    card.parameters
        .iter()
        .find(|p| p.key == "INPUT")
        .and_then(|p| p.value.clone())
}

fn normalized_keyword(keyword: &str) -> String {
    keyword
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '_')
        .collect::<String>()
        .to_ascii_uppercase()
}

fn resolve_include_path(base_dir: &Path, include: &str) -> PathBuf {
    let cleaned = include.trim().trim_matches('"').trim_matches('\'');
    let raw_path = Path::new(cleaned);
    let joined = if raw_path.is_absolute() {
        raw_path.to_path_buf()
    } else {
        base_dir.join(raw_path)
    };
    normalize_path(&joined)
}

fn normalize_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    #[test]
    fn accepts_prompt_prefixed_comments_and_separator_star_line() {
        let src = r#"
>**
** normal comment
*
*NODE
1,0,0,0
"#;
        let deck = Deck::parse_str(src).expect("parser should succeed");
        assert_eq!(deck.cards.len(), 1);
        assert_eq!(deck.cards[0].keyword, "NODE");
    }

    #[test]
    fn parses_known_legacy_fixture_edge_cases() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .expect("crate dir has parent")
            .parent()
            .expect("workspace root exists");
        let base = repo_root.join("tests/fixtures/solver");

        let files = [
            "beamfsh1.inp",
            "contact5.inp",
            "lin_stat_twisted_beam.inp",
            "oneel20fi2.inp",
            "oneel20fi3.inp",
            "opt1dp.inp",
            "rotpipe2.inp",
            "rotpipe3.inp",
        ];

        for file in files {
            let path = base.join(file);
            let result = Deck::parse_file(&path);
            assert!(
                result.is_ok(),
                "expected fixture {} to parse, got {:?}",
                path.display(),
                result.err()
            );
        }
    }

    #[test]
    fn parse_file_with_includes_expands_nested_cards() {
        let tmp = unique_temp_dir("ccx_inp_include_expand");
        fs::create_dir_all(&tmp).expect("create temp directory");
        let root = tmp.join("root.inp");
        let mid = tmp.join("mid.inc");
        let leaf = tmp.join("leaf.inc");

        fs::write(
            &root,
            "*NODE\n1,0,0,0\n*INCLUDE,INPUT=mid.inc\n*STEP\n*STATIC\n1.,1.\n",
        )
        .expect("write root");
        fs::write(
            &mid,
            "*INCLUDE,INPUT=leaf.inc\n*ELEMENT,TYPE=C3D8\n1,1,1,1,1,1,1,1,1\n",
        )
        .expect("write mid");
        fs::write(&leaf, "*MATERIAL,NAME=STEEL\n").expect("write leaf");

        let deck = Deck::parse_file_with_includes(&root).expect("parse with includes");
        let keywords: Vec<&str> = deck.cards.iter().map(|c| c.keyword.as_str()).collect();
        assert!(keywords.contains(&"NODE"));
        assert!(keywords.contains(&"ELEMENT"));
        assert!(keywords.contains(&"MATERIAL"));
        assert_eq!(
            keywords.iter().filter(|kw| **kw == "INCLUDE").count(),
            2,
            "both include cards should be present"
        );
    }

    #[test]
    fn parse_file_with_includes_detects_cycles() {
        let tmp = unique_temp_dir("ccx_inp_include_cycle");
        fs::create_dir_all(&tmp).expect("create temp directory");
        let a = tmp.join("a.inp");
        let b = tmp.join("b.inc");

        fs::write(&a, "*INCLUDE,INPUT=b.inc\n").expect("write a");
        fs::write(&b, "*INCLUDE,INPUT=a.inp\n").expect("write b");

        let err = Deck::parse_file_with_includes(&a).expect_err("cycle should fail");
        assert!(
            err.message.contains("include cycle"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn parse_file_with_includes_handles_comma_in_quoted_input_path() {
        let tmp = unique_temp_dir("ccx_inp_include_comma");
        fs::create_dir_all(&tmp).expect("create temp directory");
        let root = tmp.join("root.inp");
        let include_name = "leaf,part.inc";
        let include_file = tmp.join(include_name);

        fs::write(
            &root,
            format!("*INCLUDE, INPUT=\"{include_name}\"\n*NODE\n1,0,0,0\n"),
        )
        .expect("write root");
        fs::write(&include_file, "*ELEMENT,TYPE=C3D8\n1,1,1,1,1,1,1,1,1\n").expect("write include");

        let deck = Deck::parse_file_with_includes(&root).expect("parse with include");
        let keywords: Vec<&str> = deck.cards.iter().map(|c| c.keyword.as_str()).collect();
        assert!(keywords.contains(&"INCLUDE"));
        assert!(keywords.contains(&"ELEMENT"));
        assert!(keywords.contains(&"NODE"));
    }

    #[test]
    fn parse_file_with_includes_requires_input_parameter() {
        let tmp = unique_temp_dir("ccx_inp_include_missing_input");
        fs::create_dir_all(&tmp).expect("create temp directory");
        let root = tmp.join("root.inp");

        fs::write(&root, "*INCLUDE\n*NODE\n1,0,0,0\n").expect("write root");

        let err = Deck::parse_file_with_includes(&root).expect_err("missing INPUT should fail");
        assert!(
            err.message.contains("missing INPUT parameter"),
            "unexpected error message: {}",
            err.message
        );
        assert_eq!(err.line, 1);
    }

    #[test]
    fn parse_file_with_includes_reports_missing_file() {
        let tmp = unique_temp_dir("ccx_inp_include_missing_file");
        fs::create_dir_all(&tmp).expect("create temp directory");
        let root = tmp.join("root.inp");

        fs::write(&root, "*INCLUDE,INPUT=does_not_exist.inc\n").expect("write root");

        let err = Deck::parse_file_with_includes(&root).expect_err("missing include should fail");
        assert!(
            err.message.contains("while expanding include"),
            "unexpected error message: {}",
            err.message
        );
        assert_eq!(err.line, 0);
    }

    #[test]
    fn parse_file_with_includes_handles_single_quoted_input_path() {
        let tmp = unique_temp_dir("ccx_inp_include_single_quote");
        fs::create_dir_all(&tmp).expect("create temp directory");
        let root = tmp.join("root.inp");
        let include_file = tmp.join("leaf.inc");

        fs::write(&root, "*INCLUDE,INPUT='leaf.inc'\n").expect("write root");
        fs::write(&include_file, "*NODE\n1,0,0,0\n").expect("write include");

        let deck = Deck::parse_file_with_includes(&root).expect("parse with include");
        let keywords: Vec<&str> = deck.cards.iter().map(|c| c.keyword.as_str()).collect();
        assert!(keywords.contains(&"INCLUDE"));
        assert!(keywords.contains(&"NODE"));
    }

    fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}_{pid}_{nanos}"))
    }
}
