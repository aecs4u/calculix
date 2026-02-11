//! Error types for ccx-io

use std::fmt;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, IoError>;

#[derive(Error, Debug)]
pub enum IoError {
    #[error("Python error: {0}")]
    Python(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Conversion error: {0}")]
    Conversion(String),

    #[error("Unsupported element type: {0}")]
    UnsupportedElement(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(feature = "nastran")]
impl From<pyo3::PyErr> for IoError {
    fn from(err: pyo3::PyErr) -> Self {
        IoError::Python(format!("{}", err))
    }
}
