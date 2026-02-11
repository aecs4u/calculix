//! Output and restart I/O support for the CalculiX Rust migration.
//!
//! This crate provides:
//! - lightweight DAT/STA/FRD output writers for migration-stage runs
//! - JSON-based restart state persistence/loading
//! - FRD (result file) reader for postprocessing
//! - VTK/VTU export for ParaView visualization
//! - Postprocessing utilities (von Mises, principal stresses/strains)
//! - Nastran I/O support via pyNastran (optional, enable with `nastran` feature)

pub mod frd_reader;
mod output;
pub mod postprocess;
mod restart;
pub mod vtk_writer;

// Nastran I/O modules (optional, requires `nastran` feature)
#[cfg(feature = "nastran")]
pub mod error;
#[cfg(feature = "nastran")]
pub mod nastran;
#[cfg(feature = "nastran")]
pub mod converters;

pub use frd_reader::{
    FrdElement, FrdFile, FrdHeader, ResultBlock, ResultDataset, ResultLocation,
};
pub use output::{
    JobReport, JobStatus, OutputBundle, write_dat, write_frd_stub, write_output_bundle, write_sta,
};
pub use postprocess::{compute_mises_stress, compute_principal_stresses, TensorComponents};
pub use restart::{RestartState, load_restart, save_restart};
pub use vtk_writer::{VtkFormat, VtkWriter};

#[cfg(feature = "nastran")]
pub use error::{IoError, Result};
#[cfg(feature = "nastran")]
pub use nastran::{NastranReader, BdfData, Op2Data};
#[cfg(feature = "nastran")]
pub use converters::BdfToInpConverter;
