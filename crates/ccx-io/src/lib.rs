//! Output and restart I/O support for the CalculiX Rust migration.
//!
//! This crate provides:
//! - lightweight DAT/STA/FRD output writers for migration-stage runs
//! - JSON-based restart state persistence/loading
//! - FRD (result file) reader for postprocessing
//! - VTK/VTU export for ParaView visualization
//! - Postprocessing utilities (von Mises, principal stresses/strains)

pub mod frd_reader;
mod output;
pub mod postprocess;
mod restart;
pub mod vtk_writer;

pub use frd_reader::{
    FrdElement, FrdFile, FrdHeader, ResultBlock, ResultDataset, ResultLocation,
};
pub use output::{
    JobReport, JobStatus, OutputBundle, write_dat, write_frd_stub, write_output_bundle, write_sta,
};
pub use postprocess::{compute_mises_stress, compute_principal_stresses, TensorComponents};
pub use restart::{RestartState, load_restart, save_restart};
pub use vtk_writer::{VtkFormat, VtkWriter};
