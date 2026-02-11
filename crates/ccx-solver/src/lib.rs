//! Migration-stage Rust solver module for CalculiX.
//!
//! This crate indexes the upstream `ccx_2.23` source tree and provides
//! progressively ported Rust routines.

use std::collections::BTreeMap;

pub mod analysis;
pub mod assembly;
pub mod backend;
pub mod bc_builder;
pub mod boundary_conditions;
pub mod dat_writer;
pub mod distributed_loads;
pub mod dynamic_solver;
pub mod elements;
pub mod materials;
pub mod mesh;
pub mod mesh_builder;
pub mod modal_solver;
pub mod nonlinear_solver;
pub mod ported;
pub mod postprocess;
pub mod sets;
pub mod sparse_assembly;

pub use analysis::{AnalysisConfig, AnalysisPipeline, AnalysisResults, AnalysisType};
pub use assembly::GlobalSystem;
pub use backend::{
    default_backend, EigenResult, EigenSolver, EigenSystemData, LinearSolver, LinearSystemData,
    NativeBackend, PetscBackend, SolveInfo, SolverBackend, SparseTripletsF64,
};
pub use bc_builder::BCBuilder;
pub use boundary_conditions::{BoundaryConditions, ConcentratedLoad, DisplacementBC, DofId};
pub use dat_writer::{write_analysis_results, write_displacements_dat};
pub use distributed_loads::DistributedLoadConverter;
pub use dynamic_solver::{DynamicResults, DynamicSolver, NewmarkConfig};
pub use elements::{Beam31, BeamSection, Element as ElementTrait, SectionProperties, Truss2D};
pub use materials::{Material, MaterialLibrary, MaterialModel, MaterialStatistics};
pub use mesh::{Element, ElementType, Mesh, MeshStatistics, Node};
pub use mesh_builder::MeshBuilder;
pub use modal_solver::{ModalResults, ModalSolver};
pub use nonlinear_solver::{ConvergenceStatus, NonlinearConfig, NonlinearResults, NonlinearSolver};
pub use ported::SUPERSEDED_FORTRAN_FILES;
pub use postprocess::{
    compute_effective_strain, compute_mises_stress, compute_statistics, process_integration_points,
    read_dat_file, write_results, IntegrationPointData, IntegrationPointResult, ResultStatistics,
    StrainState, StressState,
};
pub use sets::{ElementSet, NodeSet, Sets};
pub use sparse_assembly::SparseGlobalSystem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LegacyLanguage {
    C,
    Fortran,
    Header,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacySourceUnit {
    pub legacy_rel_path: &'static str,
    pub module_name: &'static str,
    pub language: LegacyLanguage,
    pub line_count: usize,
}

include!(concat!(env!("OUT_DIR"), "/legacy_source_units.rs"));

pub const PORTED_UNITS: &[&str] = &[
    "compare.c",
    "strcmp1.c",
    "stof.c",
    "stoi.c",
    "superseded/bsort.f",
    "superseded/cident.f",
    "superseded/insertsortd.f",
    "superseded/nident.f",
    "superseded/nident2.f",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub total_units: usize,
    pub ported_units: usize,
    pub superseded_fortran_units: usize,
    pub pending_units: usize,
    pub by_language: BTreeMap<LegacyLanguage, usize>,
}

pub fn legacy_units() -> &'static [LegacySourceUnit] {
    LEGACY_SOURCE_UNITS
}

pub fn is_ported(legacy_rel_path: &str) -> bool {
    PORTED_UNITS.contains(&legacy_rel_path)
}

pub fn migration_report() -> MigrationReport {
    let mut by_language = BTreeMap::<LegacyLanguage, usize>::new();
    let mut ported = 0usize;
    let mut superseded_fortran = 0usize;

    for unit in legacy_units() {
        *by_language.entry(unit.language).or_insert(0) += 1;
        if is_ported(unit.legacy_rel_path) {
            ported += 1;
        }
        if ported::is_superseded_fortran(unit.legacy_rel_path) {
            superseded_fortran += 1;
        }
    }

    let total = legacy_units().len();
    MigrationReport {
        total_units: total,
        ported_units: ported,
        superseded_fortran_units: superseded_fortran,
        pending_units: total.saturating_sub(superseded_fortran),
        by_language,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_catalog_contains_ccx_entrypoint() {
        assert!(
            legacy_units()
                .iter()
                .any(|unit| unit.legacy_rel_path == "ccx_2.23.c")
        );
    }

    #[test]
    fn report_counts_are_consistent() {
        let report = migration_report();
        assert!(report.total_units > 0);
        assert!(report.ported_units <= report.superseded_fortran_units);
        assert_eq!(
            report.total_units,
            report.pending_units + report.superseded_fortran_units
        );
    }

    #[test]
    fn ported_lookup_matches_known_entries() {
        assert!(is_ported("superseded/cident.f"));
        assert!(!is_ported("ccx_2.23.c"));
    }
}
