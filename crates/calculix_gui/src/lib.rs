//! Migration-stage Rust GUI module for CalculiX GraphiX (CGX).

use std::collections::BTreeMap;

pub mod ported;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LegacyGuiLanguage {
    C,
    Cpp,
    Header,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyGuiSourceUnit {
    pub legacy_rel_path: &'static str,
    pub module_name: &'static str,
    pub language: LegacyGuiLanguage,
    pub line_count: usize,
}

include!(concat!(env!("OUT_DIR"), "/legacy_gui_source_units.rs"));

pub const PORTED_GUI_UNITS: &[&str] = &[
    "compare.c",
    "compareStrings.c",
    "strfind.c",
    "checkIfNumber.c",
    "v_add.c",
    "v_prod.c",
    "v_result.c",
    "v_sprod.c",
    "v_norm.c",
    "v_angle.c",
    "p_angle.c",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiMigrationReport {
    pub total_units: usize,
    pub ported_units: usize,
    pub pending_units: usize,
    pub by_language: BTreeMap<LegacyGuiLanguage, usize>,
}

pub fn legacy_gui_units() -> &'static [LegacyGuiSourceUnit] {
    LEGACY_GUI_SOURCE_UNITS
}

pub fn is_ported_gui_unit(legacy_rel_path: &str) -> bool {
    PORTED_GUI_UNITS.contains(&legacy_rel_path)
}

pub fn gui_migration_report() -> GuiMigrationReport {
    let mut by_language = BTreeMap::<LegacyGuiLanguage, usize>::new();
    let mut ported = 0usize;

    for unit in legacy_gui_units() {
        *by_language.entry(unit.language).or_insert(0) += 1;
        if is_ported_gui_unit(unit.legacy_rel_path) {
            ported += 1;
        }
    }

    let total = legacy_gui_units().len();
    GuiMigrationReport {
        total_units: total,
        ported_units: ported,
        pending_units: total.saturating_sub(ported),
        by_language,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_catalog_contains_cgx_entrypoint() {
        assert!(
            legacy_gui_units()
                .iter()
                .any(|unit| unit.legacy_rel_path == "cgx.c")
        );
    }

    #[test]
    fn report_counts_are_consistent() {
        let report = gui_migration_report();
        assert!(report.total_units > 0);
        assert_eq!(
            report.total_units,
            report.ported_units + report.pending_units
        );
    }

    #[test]
    fn ported_gui_lookup_matches_known_entries() {
        assert!(is_ported_gui_unit("compare.c"));
        assert!(!is_ported_gui_unit("cgx.c"));
    }
}
