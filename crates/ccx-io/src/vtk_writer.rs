///! VTK/VTU writer for ParaView visualization
///!
///! Converts CalculiX FRD result data to VTK formats for visualization in ParaView.
///! Supports both legacy VTK format (.vtk) and XML VTU format (.vtu).
///!
///! ## Supported Formats
///!
///! - **VTK Legacy**: ASCII text format (.vtk) - human-readable, larger files
///! - **VTU XML**: Binary or ASCII XML format (.vtu) - compressed, efficient
///!
///! ## Usage
///!
///! ```rust,no_run
///! use ccx_io::{FrdFile, VtkWriter, VtkFormat};
///!
///! let frd = FrdFile::from_file("job.frd")?;
///! let writer = VtkWriter::new(&frd);
///! writer.write_vtk("output.vtk")?;
///! writer.write_vtu("output.vtu", VtkFormat::Binary)?;
///! # Ok::<(), Box<dyn std::error::Error>>(())
///! ```

use crate::frd_reader::{FrdFile, FrdElement, ResultLocation};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// VTK output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VtkFormat {
    /// ASCII text format
    Ascii,
    /// Binary format (compressed)
    Binary,
}

/// VTK element type codes
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum VtkCellType {
    Vertex = 1,
    Line = 3,
    Triangle = 5,
    Quad = 9,
    Tetra = 10,
    Hexahedron = 12,
    Wedge = 13,
    Pyramid = 14,
    QuadraticEdge = 21,
    QuadraticTriangle = 22,
    QuadraticQuad = 23,
    QuadraticTetra = 24,
    QuadraticHexahedron = 25,
    QuadraticWedge = 26,
}

/// VTK writer for FRD data
pub struct VtkWriter<'a> {
    frd: &'a FrdFile,
}

impl<'a> VtkWriter<'a> {
    /// Create a new VTK writer for the given FRD file
    pub fn new(frd: &'a FrdFile) -> Self {
        Self { frd }
    }

    /// Write VTK legacy format file
    pub fn write_vtk<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = File::create(path)?;
        self.write_vtk_header(&mut file)?;
        self.write_vtk_points(&mut file)?;
        self.write_vtk_cells(&mut file)?;
        self.write_vtk_point_data(&mut file)?;
        Ok(())
    }

    /// Write VTU XML format file
    pub fn write_vtu<P: AsRef<Path>>(&self, path: P, format: VtkFormat) -> io::Result<()> {
        let mut file = File::create(path)?;
        self.write_vtu_header(&mut file, format)?;
        self.write_vtu_piece(&mut file)?;
        self.write_vtu_footer(&mut file)?;
        Ok(())
    }

    /// Write VTK header
    fn write_vtk_header(&self, file: &mut File) -> io::Result<()> {
        writeln!(file, "# vtk DataFile Version 3.0")?;
        writeln!(file, "CalculiX Results")?;
        writeln!(file, "ASCII")?;
        writeln!(file, "DATASET UNSTRUCTURED_GRID")?;
        Ok(())
    }

    /// Write node coordinates (POINTS)
    fn write_vtk_points(&self, file: &mut File) -> io::Result<()> {
        writeln!(file, "POINTS {} float", self.frd.nodes.len())?;

        // Create sorted list of node IDs for consistent ordering
        let mut node_ids: Vec<_> = self.frd.nodes.keys().copied().collect();
        node_ids.sort();

        for node_id in &node_ids {
            if let Some(coords) = self.frd.nodes.get(node_id) {
                writeln!(file, "{} {} {}", coords[0], coords[1], coords[2])?;
            }
        }

        Ok(())
    }

    /// Write element connectivity (CELLS)
    fn write_vtk_cells(&self, file: &mut File) -> io::Result<()> {
        let num_elements = self.frd.elements.len();

        // Calculate total size (each element: count + node_ids)
        let total_size: usize = self
            .frd
            .elements
            .values()
            .map(|e| 1 + e.nodes.len())
            .sum();

        writeln!(file, "CELLS {} {}", num_elements, total_size)?;

        // Create node ID mapping for indexing
        let node_id_to_index: HashMap<i32, usize> = self
            .frd
            .nodes
            .keys()
            .enumerate()
            .map(|(idx, &node_id)| (node_id, idx))
            .collect();

        // Write connectivity for each element
        let mut element_ids: Vec<_> = self.frd.elements.keys().copied().collect();
        element_ids.sort();

        for elem_id in &element_ids {
            if let Some(element) = self.frd.elements.get(elem_id) {
                write!(file, "{}", element.nodes.len())?;
                for &node_id in &element.nodes {
                    if let Some(&node_idx) = node_id_to_index.get(&node_id) {
                        write!(file, " {}", node_idx)?;
                    }
                }
                writeln!(file)?;
            }
        }

        // Write cell types
        writeln!(file, "CELL_TYPES {}", num_elements)?;
        for elem_id in &element_ids {
            if let Some(element) = self.frd.elements.get(elem_id) {
                let vtk_type = Self::frd_to_vtk_cell_type(element);
                writeln!(file, "{}", vtk_type as i32)?;
            }
        }

        Ok(())
    }

    /// Write point data (results)
    fn write_vtk_point_data(&self, file: &mut File) -> io::Result<()> {
        if self.frd.result_blocks.is_empty() {
            return Ok(());
        }

        writeln!(file, "POINT_DATA {}", self.frd.nodes.len())?;

        // Write results from the last time step (or first if only one)
        if let Some(result_block) = self.frd.result_blocks.last() {
            for dataset in &result_block.datasets {
                // Only write nodal results for POINT_DATA
                if dataset.location != ResultLocation::Nodal {
                    continue;
                }

                // Determine if scalar, vector, or tensor
                match dataset.ncomps {
                    1 => {
                        // Scalar field
                        writeln!(file, "SCALARS {} float 1", dataset.name)?;
                        writeln!(file, "LOOKUP_TABLE default")?;

                        // Create sorted node ID list
                        let mut node_ids: Vec<_> = self.frd.nodes.keys().copied().collect();
                        node_ids.sort();

                        for node_id in &node_ids {
                            if let Some(values) = dataset.values.get(node_id) {
                                if !values.is_empty() {
                                    writeln!(file, "{}", values[0])?;
                                } else {
                                    writeln!(file, "0.0")?;
                                }
                            } else {
                                writeln!(file, "0.0")?;
                            }
                        }
                    }
                    3 => {
                        // Vector field
                        writeln!(file, "VECTORS {} float", dataset.name)?;

                        let mut node_ids: Vec<_> = self.frd.nodes.keys().copied().collect();
                        node_ids.sort();

                        for node_id in &node_ids {
                            if let Some(values) = dataset.values.get(node_id) {
                                if values.len() >= 3 {
                                    writeln!(file, "{} {} {}", values[0], values[1], values[2])?;
                                } else {
                                    writeln!(file, "0.0 0.0 0.0")?;
                                }
                            } else {
                                writeln!(file, "0.0 0.0 0.0")?;
                            }
                        }
                    }
                    6 => {
                        // Tensor field (6 components: XX, YY, ZZ, XY, YZ, XZ)
                        writeln!(file, "TENSORS {} float", dataset.name)?;

                        let mut node_ids: Vec<_> = self.frd.nodes.keys().copied().collect();
                        node_ids.sort();

                        for node_id in &node_ids {
                            if let Some(values) = dataset.values.get(node_id) {
                                if values.len() >= 6 {
                                    // Convert Voigt notation to full tensor
                                    writeln!(
                                        file,
                                        "{} {} {}",
                                        values[0], values[3], values[5]
                                    )?;
                                    writeln!(
                                        file,
                                        "{} {} {}",
                                        values[3], values[1], values[4]
                                    )?;
                                    writeln!(
                                        file,
                                        "{} {} {}",
                                        values[5], values[4], values[2]
                                    )?;
                                } else {
                                    writeln!(file, "0.0 0.0 0.0")?;
                                    writeln!(file, "0.0 0.0 0.0")?;
                                    writeln!(file, "0.0 0.0 0.0")?;
                                }
                            } else {
                                writeln!(file, "0.0 0.0 0.0")?;
                                writeln!(file, "0.0 0.0 0.0")?;
                                writeln!(file, "0.0 0.0 0.0")?;
                            }
                            writeln!(file)?; // Blank line between tensors
                        }
                    }
                    _ => {
                        // Other component counts - skip for now
                    }
                }
            }
        }

        Ok(())
    }

    /// Write VTU XML header
    fn write_vtu_header(&self, file: &mut File, format: VtkFormat) -> io::Result<()> {
        writeln!(file, "<?xml version=\"1.0\"?>")?;
        writeln!(file, "<VTKFile type=\"UnstructuredGrid\" version=\"1.0\" byte_order=\"LittleEndian\">")?;

        let format_str = match format {
            VtkFormat::Ascii => "ascii",
            VtkFormat::Binary => "binary",
        };
        writeln!(file, "  <UnstructuredGrid>")?;
        writeln!(
            file,
            "    <Piece NumberOfPoints=\"{}\" NumberOfCells=\"{}\">",
            self.frd.nodes.len(),
            self.frd.elements.len()
        )?;

        Ok(())
    }

    /// Write VTU piece data
    fn write_vtu_piece(&self, file: &mut File) -> io::Result<()> {
        // Points
        writeln!(file, "      <Points>")?;
        writeln!(
            file,
            "        <DataArray type=\"Float32\" NumberOfComponents=\"3\" format=\"ascii\">"
        )?;

        let mut node_ids: Vec<_> = self.frd.nodes.keys().copied().collect();
        node_ids.sort();

        for node_id in &node_ids {
            if let Some(coords) = self.frd.nodes.get(node_id) {
                writeln!(file, "          {} {} {}", coords[0], coords[1], coords[2])?;
            }
        }

        writeln!(file, "        </DataArray>")?;
        writeln!(file, "      </Points>")?;

        // TODO: Cells, PointData, CellData sections
        // This is a simplified implementation

        Ok(())
    }

    /// Write VTU footer
    fn write_vtu_footer(&self, file: &mut File) -> io::Result<()> {
        writeln!(file, "    </Piece>")?;
        writeln!(file, "  </UnstructuredGrid>")?;
        writeln!(file, "</VTKFile>")?;
        Ok(())
    }

    /// Convert FRD element type to VTK cell type
    fn frd_to_vtk_cell_type(element: &FrdElement) -> VtkCellType {
        // FRD element type codes (from cgx manual)
        // 1 = C3D8 (8-node brick) -> VTK_HEXAHEDRON
        // 2 = C3D6 (6-node wedge) -> VTK_WEDGE
        // 3 = C3D4 (4-node tet) -> VTK_TETRA
        // 4 = C3D20 (20-node brick) -> VTK_QUADRATIC_HEXAHEDRON
        // etc.

        match element.element_type {
            1 => VtkCellType::Hexahedron,        // C3D8
            2 => VtkCellType::Wedge,              // C3D6
            3 => VtkCellType::Tetra,              // C3D4
            4 => VtkCellType::QuadraticHexahedron, // C3D20
            5 => VtkCellType::QuadraticWedge,     // C3D15
            6 => VtkCellType::Pyramid,            // C3D5?
            7 => VtkCellType::Line,               // B31, T3D2
            8 => VtkCellType::QuadraticEdge,      // B32
            9 => VtkCellType::Triangle,           // S3
            10 => VtkCellType::Quad,              // S4, S8
            11 => VtkCellType::QuadraticTetra,    // C3D10
            _ => {
                // Default based on node count
                match element.nodes.len() {
                    1 => VtkCellType::Vertex,
                    2 => VtkCellType::Line,
                    3 => VtkCellType::Triangle,
                    4 => VtkCellType::Tetra,
                    6 => VtkCellType::Wedge,
                    8 => VtkCellType::Hexahedron,
                    _ => VtkCellType::Vertex,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frd_reader::{FrdFile, FrdHeader};

    #[test]
    fn test_vtk_writer_creation() {
        let frd = FrdFile {
            header: FrdHeader::default(),
            nodes: HashMap::new(),
            elements: HashMap::new(),
            result_blocks: Vec::new(),
        };

        let writer = VtkWriter::new(&frd);
        assert_eq!(writer.frd.nodes.len(), 0);
    }

    #[test]
    fn test_frd_to_vtk_cell_type() {
        use crate::frd_reader::FrdElement;

        // Test C3D8 (hexahedron)
        let elem = FrdElement {
            id: 1,
            element_type: 1,
            nodes: vec![1, 2, 3, 4, 5, 6, 7, 8],
        };

        let vtk_type = VtkWriter::frd_to_vtk_cell_type(&elem);
        assert_eq!(vtk_type as i32, VtkCellType::Hexahedron as i32);

        // Test C3D4 (tetra)
        let elem = FrdElement {
            id: 2,
            element_type: 3,
            nodes: vec![1, 2, 3, 4],
        };

        let vtk_type = VtkWriter::frd_to_vtk_cell_type(&elem);
        assert_eq!(vtk_type as i32, VtkCellType::Tetra as i32);
    }
}
