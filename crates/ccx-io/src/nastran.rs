//! Nastran file I/O via pyNastran
//!
//! This module provides a Rust API for reading Nastran BDF and OP2 files
//! using Python's pyNastran library through PyO3.

use crate::error::{IoError, Result};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyModule};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Nastran BDF (Bulk Data File) data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdfData {
    pub nodes: HashMap<i32, Node>,
    pub elements: HashMap<i32, Element>,
    pub materials: HashMap<i32, Material>,
    pub properties: HashMap<i32, Property>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    pub id: i32,
    pub elem_type: String,
    pub nodes: Vec<i32>,
    pub property_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub id: i32,
    pub name: String,
    pub elastic_modulus: Option<f64>,
    pub poissons_ratio: Option<f64>,
    pub density: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub id: i32,
    pub property_type: String,
    pub material_id: i32,
    pub thickness: Option<f64>,
    pub area: Option<f64>,
}

/// Nastran OP2 (binary output) data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Op2Data {
    pub displacements: HashMap<i32, Displacement>,
    pub stresses: HashMap<i32, Stress>,
    pub eigenvalues: Vec<f64>,
    pub eigenvectors: HashMap<i32, Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Displacement {
    pub node_id: i32,
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
    pub rx: f64,
    pub ry: f64,
    pub rz: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stress {
    pub element_id: i32,
    pub sx: f64,
    pub sy: f64,
    pub sz: f64,
    pub sxy: f64,
    pub syz: f64,
    pub szx: f64,
}

/// Nastran file reader using pyNastran
pub struct NastranReader {
    python_module: Py<PyModule>,
}

impl NastranReader {
    /// Create a new Nastran reader
    ///
    /// # Errors
    /// Returns error if Python initialization fails or pyNastran is not installed
    pub fn new() -> Result<Self> {
        Python::with_gil(|py| {
            // Import our Python wrapper module
            let python_code = include_str!("../python/nastran_io.py");
            let module = PyModule::from_code(py, python_code, "nastran_io.py", "nastran_io")?;

            Ok(Self {
                python_module: module.into(),
            })
        })
    }

    /// Read a BDF file
    ///
    /// # Arguments
    /// * `path` - Path to the .bdf file
    ///
    /// # Returns
    /// Parsed BDF data
    pub fn read_bdf<P: AsRef<Path>>(&self, path: P) -> Result<BdfData> {
        let path_str = path.as_ref().to_str()
            .ok_or_else(|| IoError::InvalidData("Invalid path".to_string()))?;

        Python::with_gil(|py| {
            let module = self.python_module.as_ref(py);
            let read_bdf = module.getattr("read_bdf")?;

            // Call Python function
            let result = read_bdf.call1((path_str,))?;

            // Convert Python dict to JSON string
            let json_str: String = result.call_method0("to_json")?.extract()?;

            // Parse JSON to Rust struct
            let bdf_data: BdfData = serde_json::from_str(&json_str)?;

            Ok(bdf_data)
        })
    }

    /// Read an OP2 file
    ///
    /// # Arguments
    /// * `path` - Path to the .op2 file
    ///
    /// # Returns
    /// Parsed OP2 data
    pub fn read_op2<P: AsRef<Path>>(&self, path: P) -> Result<Op2Data> {
        let path_str = path.as_ref().to_str()
            .ok_or_else(|| IoError::InvalidData("Invalid path".to_string()))?;

        Python::with_gil(|py| {
            let module = self.python_module.as_ref(py);
            let read_op2 = module.getattr("read_op2")?;

            // Call Python function
            let result = read_op2.call1((path_str,))?;

            // Convert Python dict to JSON string
            let json_str: String = result.call_method0("to_json")?.extract()?;

            // Parse JSON to Rust struct
            let op2_data: Op2Data = serde_json::from_str(&json_str)?;

            Ok(op2_data)
        })
    }

    /// Get BDF statistics
    pub fn get_bdf_stats<P: AsRef<Path>>(&self, path: P) -> Result<BdfStats> {
        let path_str = path.as_ref().to_str()
            .ok_or_else(|| IoError::InvalidData("Invalid path".to_string()))?;

        Python::with_gil(|py| {
            let module = self.python_module.as_ref(py);
            let get_stats = module.getattr("get_bdf_stats")?;

            let result = get_stats.call1((path_str,))?;

            Ok(BdfStats {
                num_nodes: result.get_item("num_nodes")?.extract()?,
                num_elements: result.get_item("num_elements")?.extract()?,
                num_materials: result.get_item("num_materials")?.extract()?,
                num_properties: result.get_item("num_properties")?.extract()?,
                element_types: result.get_item("element_types")?.extract()?,
            })
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdfStats {
    pub num_nodes: usize,
    pub num_elements: usize,
    pub num_materials: usize,
    pub num_properties: usize,
    pub element_types: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires pyNastran installation
    fn test_reader_creation() {
        let reader = NastranReader::new();
        assert!(reader.is_ok());
    }
}
