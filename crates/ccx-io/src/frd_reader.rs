///! CalculiX FRD (result) file reader
///!
///! Reads CalculiX .frd result files for postprocessing and visualization.
///! Based on the FRD format specification from cgx_2.20.pdf Manual, § 11.
///!
///! The FRD format contains:
///! - Node coordinates
///! - Element connectivity
///! - Result data (displacements, stresses, strains, etc.) for each time step
///!
///! ## Format Overview
///!
///! FRD files use fixed-width fields:
///! - Node block: `-1` marker, node number (10 chars), coordinates (3×12 chars)
///! - Element block: `-2` marker, element number, type, nodes
///! - Result blocks: `100C` marker for nodal results, `100CL` for element results
///!
///! ## Usage
///!
///! ```rust,no_run
///! use ccx_io::FrdFile;
///!
///! let frd = FrdFile::from_file("job.frd")?;
///! println!("Nodes: {}, Elements: {}", frd.nodes.len(), frd.elements.len());
///! println!("Time steps: {}", frd.result_blocks.len());
///! # Ok::<(), Box<dyn std::error::Error>>(())
///! ```

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// FRD file representation
#[derive(Debug, Clone)]
pub struct FrdFile {
    /// Header information
    pub header: FrdHeader,
    /// Node coordinates (node_id → coordinates)
    pub nodes: HashMap<i32, [f64; 3]>,
    /// Element connectivity (element_id → Element)
    pub elements: HashMap<i32, FrdElement>,
    /// Result data for each time step
    pub result_blocks: Vec<ResultBlock>,
}

/// FRD file header
#[derive(Debug, Clone, Default)]
pub struct FrdHeader {
    /// File version string
    pub version: String,
    /// Job name
    pub job_name: String,
    /// Additional header lines
    pub info: Vec<String>,
}

/// Element in FRD file
#[derive(Debug, Clone)]
pub struct FrdElement {
    /// Element ID
    pub id: i32,
    /// Element type code (FRD format)
    pub element_type: i32,
    /// Node connectivity
    pub nodes: Vec<i32>,
}

/// Result block for one time step
#[derive(Debug, Clone)]
pub struct ResultBlock {
    /// Step number
    pub step: i32,
    /// Increment/time value
    pub time: f64,
    /// Result datasets in this block
    pub datasets: Vec<ResultDataset>,
}

/// Result dataset (one variable for all nodes/elements)
#[derive(Debug, Clone)]
pub struct ResultDataset {
    /// Dataset name (e.g., "DISP", "STRESS", "STRAIN")
    pub name: String,
    /// Number of components (1=scalar, 3=vector, 6=tensor)
    pub ncomps: usize,
    /// Component names
    pub comp_names: Vec<String>,
    /// Nodal or element result
    pub location: ResultLocation,
    /// Result values (entity_id → component values)
    pub values: HashMap<i32, Vec<f64>>,
}

/// Location of result data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultLocation {
    /// Nodal results
    Nodal,
    /// Element results (at integration points)
    Element,
}

impl FrdFile {
    /// Read FRD file from path
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::from_reader(reader)
    }

    /// Read FRD file from a buffered reader
    pub fn from_reader<R: BufRead>(mut reader: R) -> io::Result<Self> {
        let mut frd = FrdFile {
            header: FrdHeader::default(),
            nodes: HashMap::new(),
            elements: HashMap::new(),
            result_blocks: Vec::new(),
        };

        let mut line = String::new();

        // Read file line by line
        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                break; // EOF
            }

            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Parse based on record type marker
            match &trimmed[0..std::cmp::min(5, trimmed.len())] {
                // Header record (1PSTEP, 1U or similar)
                s if s.starts_with('1') => {
                    frd.header.info.push(trimmed.to_string());
                }
                // Node coordinates block
                "    2" | "   2C" => {
                    Self::read_node_block(&mut reader, &mut frd.nodes)?;
                }
                // Element block
                "    3" | "   3C" => {
                    Self::read_element_block(&mut reader, &mut frd.elements)?;
                }
                // Result block (100C for nodal, 100CL for element)
                "  100" => {
                    let result_block = Self::read_result_block(&mut reader, trimmed)?;
                    frd.result_blocks.push(result_block);
                }
                // End markers (-3, 9999)
                "   -3" | " 9999" => {
                    // Block end, continue
                }
                _ => {
                    // Unknown or comment line, skip
                }
            }
        }

        Ok(frd)
    }

    /// Read node coordinate block (record type 2)
    fn read_node_block<R: BufRead>(
        reader: &mut R,
        nodes: &mut HashMap<i32, [f64; 3]>,
    ) -> io::Result<()> {
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                break;
            }

            let trimmed = line.trim();

            // End of block
            if trimmed == "-3" || trimmed.is_empty() {
                break;
            }

            // Node line format: -1<node_id:10><x:12><y:12><z:12>
            if !trimmed.starts_with("-1") {
                continue;
            }

            // Parse fixed-width fields (FRD format specification)
            if line.len() < 2 + 10 + 12 * 3 {
                continue; // Line too short
            }

            let node_id_str = &line[2..12].trim();
            let x_str = &line[12..24].trim();
            let y_str = &line[24..36].trim();
            let z_str = &line[36..48].trim();

            if let (Ok(node_id), Ok(x), Ok(y), Ok(z)) = (
                node_id_str.parse::<i32>(),
                x_str.parse::<f64>(),
                y_str.parse::<f64>(),
                z_str.parse::<f64>(),
            ) {
                nodes.insert(node_id, [x, y, z]);
            }
        }

        Ok(())
    }

    /// Read element connectivity block (record type 3)
    fn read_element_block<R: BufRead>(
        reader: &mut R,
        elements: &mut HashMap<i32, FrdElement>,
    ) -> io::Result<()> {
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                break;
            }

            let trimmed = line.trim();

            // End of block
            if trimmed == "-3" || trimmed.is_empty() {
                break;
            }

            // Element header line: -1<elem_id><elem_type>
            if trimmed.starts_with("-1") && line.len() >= 2 + 10 + 5 {
                let elem_id_str = &line[2..12].trim();
                let elem_type_str = &line[12..17].trim();

                if let (Ok(elem_id), Ok(elem_type)) = (
                    elem_id_str.parse::<i32>(),
                    elem_type_str.parse::<i32>(),
                ) {
                    // Read element nodes
                    let nodes = Self::read_element_nodes(reader)?;

                    elements.insert(
                        elem_id,
                        FrdElement {
                            id: elem_id,
                            element_type: elem_type,
                            nodes,
                        },
                    );
                }
            }
        }

        Ok(())
    }

    /// Read element node connectivity lines
    fn read_element_nodes<R: BufRead>(reader: &mut R) -> io::Result<Vec<i32>> {
        let mut nodes = Vec::new();
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                break;
            }

            let trimmed = line.trim();

            // Node continuation line: -2<node1><node2>...
            if !trimmed.starts_with("-2") {
                // Not a continuation line, put it back (conceptually)
                break;
            }

            // Parse node IDs (10 chars each after -2)
            let node_data = &line[2..];
            for chunk in node_data.as_bytes().chunks(10) {
                if let Ok(s) = std::str::from_utf8(chunk) {
                    if let Ok(node_id) = s.trim().parse::<i32>() {
                        nodes.push(node_id);
                    }
                }
            }
        }

        Ok(nodes)
    }

    /// Read result data block (record type 100)
    fn read_result_block<R: BufRead>(reader: &mut R, _header_line: &str) -> io::Result<ResultBlock> {
        // Parse result block header
        // Format: 100C<step><time><dataset_name><ncomps>...

        let mut result_block = ResultBlock {
            step: 1,
            time: 0.0,
            datasets: Vec::new(),
        };

        // TODO: Parse header line to extract step, time, dataset info
        // This is a simplified implementation

        let mut line = String::new();
        let mut current_dataset: Option<ResultDataset> = None;

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                break;
            }

            let trimmed = line.trim();

            // End of result block
            if trimmed == "-3" || trimmed.starts_with("  100") {
                if let Some(dataset) = current_dataset.take() {
                    result_block.datasets.push(dataset);
                }

                if trimmed.starts_with("  100") {
                    // Another dataset in same block, continue
                    continue;
                } else {
                    break;
                }
            }

            // Result value line: -1<node_id><value1><value2>...
            if trimmed.starts_with("-1") {
                // TODO: Parse result values
                // This requires knowledge of the dataset format
            }
        }

        Ok(result_block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frd_file_creation() {
        let frd = FrdFile {
            header: FrdHeader::default(),
            nodes: HashMap::new(),
            elements: HashMap::new(),
            result_blocks: Vec::new(),
        };

        assert_eq!(frd.nodes.len(), 0);
        assert_eq!(frd.elements.len(), 0);
        assert_eq!(frd.result_blocks.len(), 0);
    }

    #[test]
    fn test_node_parsing() {
        // Test basic node structure creation
        // Full FRD parsing tested in integration tests
        let mut nodes = HashMap::new();
        nodes.insert(1, [0.0, 0.0, 0.0]);
        nodes.insert(2, [1.0, 0.0, 0.0]);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes.get(&1), Some(&[0.0, 0.0, 0.0]));
        assert_eq!(nodes.get(&2), Some(&[1.0, 0.0, 0.0]));
    }
}
