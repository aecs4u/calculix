//! I/O support for the CalculiX Rust migration.
//!
//! This crate provides:
//! - **INP (input deck)** parser for CalculiX/Abaqus keyword format
//! - **DAT/STA/FRD** output writers for migration-stage runs
//! - **JSON-based restart** state persistence/loading
//! - **FRD (result file)** reader for postprocessing
//! - **VTK/VTU export** for ParaView visualization
//! - **Postprocessing utilities** (von Mises, principal stresses/strains)
//! - **Nastran I/O** via pyNastran (optional, enable with `nastran` feature)
//! - **Meshio integration** (Python) for 40+ mesh formats (VTK, STL, Gmsh, ANSYS, etc.)
//!   - Available via `ccx-cli meshio-info`, `meshio-convert`, `meshio-formats`
//!   - Python module: `crates/ccx-io/python/meshio_wrapper.py`
//!   - Install: `pip install meshio` (requires Python 3.7+)

pub mod inp;
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

pub use inp::{Card, Deck, Parameter, ParseError as InpParseError};
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
