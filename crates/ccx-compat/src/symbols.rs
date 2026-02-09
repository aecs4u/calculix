#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyLanguage {
    C,
    Fortran,
}

pub fn canonical_symbol(name: &str, language: LegacyLanguage) -> String {
    match language {
        LegacyLanguage::C => sanitize_symbol(name),
        LegacyLanguage::Fortran => fortran_symbol(name),
    }
}

pub fn fortran_symbol(name: &str) -> String {
    let sanitized = sanitize_symbol(name).to_ascii_lowercase();
    if sanitized.ends_with('_') {
        sanitized
    } else {
        format!("{sanitized}_")
    }
}

pub fn rust_module_from_legacy_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for ch in path.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    out
}

fn sanitize_symbol(name: &str) -> String {
    name.trim()
        .trim_end_matches('\0')
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_c_symbols() {
        assert_eq!(canonical_symbol(" compare\0", LegacyLanguage::C), "compare");
    }

    #[test]
    fn canonicalizes_fortran_symbols() {
        assert_eq!(
            canonical_symbol("NIDENT2", LegacyLanguage::Fortran),
            "nident2_"
        );
        assert_eq!(fortran_symbol("calc_"), "calc_");
    }

    #[test]
    fn converts_legacy_path_to_rust_module_name() {
        assert_eq!(
            rust_module_from_legacy_path("superseded/nident2.f"),
            "superseded_nident2_f"
        );
    }
}
