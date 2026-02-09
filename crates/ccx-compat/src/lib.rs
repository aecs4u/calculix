//! Compatibility bridge primitives for incremental C/Fortran migration.
//!
//! This crate provides:
//! - symbol normalization helpers for legacy C/Fortran routines
//! - a runtime registry to route calls through temporary compatibility shims

mod bridge;
mod symbols;

pub use bridge::{
    CallingConvention, CompatError, CompatRegistry, RoutineHandle, RoutineSpec, ScalarRoutine,
};
pub use symbols::{LegacyLanguage, canonical_symbol, fortran_symbol, rust_module_from_legacy_path};
