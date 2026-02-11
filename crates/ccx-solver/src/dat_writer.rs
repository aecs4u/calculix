//! DAT file writer for CalculiX output format
//!
//! Writes solver results in the CalculiX .dat format for comparison with reference outputs.

use crate::mesh::Mesh;
use nalgebra::DVector;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Write displacement results to a .dat file in CalculiX format
///
/// # Arguments
/// * `output_path` - Path to output .dat file
/// * `mesh` - Mesh with node definitions
/// * `displacements` - Global displacement vector
/// * `step` - Step number (default 1)
/// * `increment` - Increment number (default 1)
/// * `time` - Analysis time (default 1.0)
///
/// # Example
/// ```ignore
/// write_displacements_dat("output.dat", &mesh, &displacements, 1, 1, 1.0)?;
/// ```
pub fn write_displacements_dat(
    output_path: &Path,
    mesh: &Mesh,
    displacements: &DVector<f64>,
    step: usize,
    increment: usize,
    time: f64,
) -> io::Result<()> {
    let mut file = File::create(output_path)?;

    // Write header
    writeln!(file)?;
    writeln!(file, "                        S T E P       {}", step)?;
    writeln!(file)?;
    writeln!(file)?;
    writeln!(file, "                                INCREMENT     {}", increment)?;
    writeln!(file)?;
    writeln!(file)?;
    writeln!(
        file,
        " displacements (vx,vy,vz) for set NALL and time  {:.7E}",
        time
    )?;
    writeln!(file)?;

    // Determine DOFs per node from mesh
    // Calculate from num_dofs and num_nodes
    let dofs_per_node = if mesh.nodes.is_empty() {
        3
    } else {
        mesh.num_dofs / mesh.nodes.len()
    };

    // Write displacement data for each node
    let mut node_ids: Vec<_> = mesh.nodes.keys().copied().collect();
    node_ids.sort();

    for node_id in node_ids {
        let node_idx = (node_id - 1) as usize;

        // Extract displacements for this node
        let mut ux = 0.0;
        let mut uy = 0.0;
        let mut uz = 0.0;

        // Get DOF indices
        let dof_start = node_idx * dofs_per_node;

        if dof_start < displacements.len() {
            ux = displacements[dof_start];
        }
        if dof_start + 1 < displacements.len() {
            uy = displacements[dof_start + 1];
        }
        if dof_start + 2 < displacements.len() {
            uz = displacements[dof_start + 2];
        }

        writeln!(
            file,
            "{:10} {:13.6E} {:13.6E} {:13.6E}",
            node_id, ux, uy, uz
        )?;
    }

    writeln!(file)?;
    Ok(())
}

/// Write complete analysis results to DAT file
///
/// This is a higher-level function that writes multiple result types
/// (displacements, stresses, strains) if available.
pub fn write_analysis_results(
    output_path: &Path,
    mesh: &Mesh,
    displacements: &DVector<f64>,
) -> io::Result<()> {
    write_displacements_dat(output_path, mesh, displacements, 1, 1, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::{Mesh, Node, Element, ElementType};
    use std::collections::HashMap;

    #[test]
    fn test_write_displacements_simple() {
        // Create simple 2-node mesh
        let mut nodes = HashMap::new();
        nodes.insert(1, Node::new(1, 0.0, 0.0, 0.0));
        nodes.insert(2, Node::new(2, 1.0, 0.0, 0.0));

        let mut elements = HashMap::new();
        elements.insert(
            1,
            Element {
                id: 1,
                element_type: ElementType::T3D2,
                connectivity: vec![1, 2],
            },
        );

        let mesh = Mesh {
            nodes,
            elements,
            num_dofs: 6,
            dofs_per_node: 3,
            node_dof_map: HashMap::new(),
        };

        // Create displacement vector
        let displacements = DVector::from_vec(vec![0.0, 0.0, 0.0, 0.001, 0.0, 0.0]);

        // Write to temporary file
        let temp_path = std::env::temp_dir().join("test_displacements.dat");
        let result = write_displacements_dat(&temp_path, &mesh, &displacements, 1, 1, 1.0);
        assert!(result.is_ok());

        // Verify file was created
        assert!(temp_path.exists());

        // Read and verify content
        let content = std::fs::read_to_string(&temp_path).unwrap();
        assert!(content.contains("S T E P       1"));
        assert!(content.contains("INCREMENT     1"));
        assert!(content.contains("displacements"));
        assert!(content.contains("1  0.000000E+00  0.000000E+00  0.000000E+00"));
        assert!(content.contains("2  1.000000E-03  0.000000E+00  0.000000E+00"));

        // Cleanup
        std::fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_write_analysis_results() {
        let mut nodes = HashMap::new();
        nodes.insert(1, Node::new(1, 0.0, 0.0, 0.0));

        let mesh = Mesh {
            nodes,
            elements: HashMap::new(),
            num_dofs: 3,
            dofs_per_node: 3,
            node_dof_map: HashMap::new(),
        };

        let displacements = DVector::from_vec(vec![0.0, 0.0, 0.0]);

        let temp_path = std::env::temp_dir().join("test_analysis.dat");
        let result = write_analysis_results(&temp_path, &mesh, &displacements);
        assert!(result.is_ok());
        assert!(temp_path.exists());
        std::fs::remove_file(temp_path).ok();
    }
}
