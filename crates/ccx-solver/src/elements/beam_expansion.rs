//! Beam element expansion to 3D solid elements
//!
//! Implements CalculiX's beam expansion strategy where beam elements (B31, B32, B32R)
//! are internally expanded into 3D solid elements (C3D20R) for analysis.
//!
//! # Expansion Strategy
//!
//! Each beam node expands into 8 nodes arranged according to the cross-section:
//! - For rectangular sections: nodes at the 4 corners + 4 mid-edges
//! - Section is oriented using the beam normal direction
//!
//! # Example
//!
//! B32R element (3 beam nodes) → 3 sets of 8 nodes = 24 total nodes
//! These form C3D20R elements with proper connectivity.
//!
//! # References
//!
//! CalculiX CrunchiX User's Manual v2.23:
//! - Section 6.2.11: B32R elements
//! - Section 6.9.4: Beam sections

use crate::mesh::{Node, Element, ElementType};
use crate::elements::{BeamSection, SectionShape};
use nalgebra::Vector3;
use std::collections::HashMap;

/// Configuration for beam expansion
#[derive(Debug, Clone)]
pub struct BeamExpansionConfig {
    /// Starting node ID for generated nodes (avoids conflicts)
    pub next_node_id: i32,
    /// Starting element ID for generated solid elements
    pub next_element_id: i32,
}

impl Default for BeamExpansionConfig {
    fn default() -> Self {
        Self {
            next_node_id: 1_000_000,  // Start at high ID to avoid conflicts
            next_element_id: 1_000_000,
        }
    }
}

/// Result of beam expansion
#[derive(Debug)]
pub struct ExpansionResult {
    /// Generated 3D nodes from beam cross-sections
    pub nodes: HashMap<i32, Node>,
    /// Generated C3D20R solid elements
    pub elements: HashMap<i32, Element>,
    /// Mapping from original beam node ID to generated node IDs [8 nodes per beam node]
    pub beam_node_mapping: HashMap<i32, [i32; 8]>,
}

/// Expand a B32R beam element into C3D20R solid elements
///
/// # Arguments
/// * `beam_elem` - Original beam element
/// * `beam_nodes` - Beam node coordinates [3 nodes for B32R]
/// * `section` - Beam section properties
/// * `normal` - Beam normal direction vector
/// * `config` - Expansion configuration
///
/// # Returns
/// ExpansionResult with generated nodes and elements
pub fn expand_b32r(
    beam_elem: &Element,
    beam_nodes: &[Node; 3],
    section: &BeamSection,
    normal: Vector3<f64>,
    config: &mut BeamExpansionConfig,
) -> Result<ExpansionResult, String> {
    if beam_elem.element_type != ElementType::B32 {
        return Err(format!("Expected B32 element, got {:?}", beam_elem.element_type));
    }

    let mut nodes = HashMap::new();
    let mut beam_node_mapping = HashMap::new();

    // Generate 8 nodes for each of the 3 beam nodes
    for (i, beam_node) in beam_nodes.iter().enumerate() {
        let section_nodes = generate_section_nodes(
            beam_node,
            beam_nodes,
            section,
            normal,
            config.next_node_id,
        )?;

        let node_ids: Vec<i32> = section_nodes.iter().map(|n| n.id).collect();
        beam_node_mapping.insert(beam_node.id, [
            node_ids[0], node_ids[1], node_ids[2], node_ids[3],
            node_ids[4], node_ids[5], node_ids[6], node_ids[7],
        ]);

        for node in section_nodes {
            nodes.insert(node.id, node);
            config.next_node_id += 1;
        }
    }

    // Generate C3D20R elements
    // For B32R (3 beam nodes), we create 1 C3D20R element spanning all 3
    let elements = generate_c3d20r_elements(
        beam_elem.id,
        &beam_node_mapping,
        &beam_nodes.iter().map(|n| n.id).collect::<Vec<_>>(),
        config,
    )?;

    Ok(ExpansionResult {
        nodes,
        elements,
        beam_node_mapping,
    })
}

/// Generate 8 nodes for a beam cross-section at a given beam node
///
/// Node arrangement for rectangular section (looking along beam axis):
/// ```text
///   6-------7
///   |       |
///   |   *   |  (* = beam node)
///   |       |
///   4-------5
/// ```
///
/// Plus 4 mid-edge nodes: 0 (bottom-center), 1 (right-center), 2 (top-center), 3 (left-center)
fn generate_section_nodes(
    beam_node: &Node,
    all_beam_nodes: &[Node; 3],
    section: &BeamSection,
    normal_vec: Vector3<f64>,
    start_id: i32,
) -> Result<Vec<Node>, String> {
    // Compute local coordinate system at beam node
    let (tangent, normal, binormal) = compute_beam_local_coords(beam_node, all_beam_nodes, normal_vec)?;

    // Get section dimensions
    let (width, height) = match &section.shape {
        SectionShape::Rectangular { width, height } => (*width, *height),
        _ => return Err("Only rectangular sections supported for expansion".to_string()),
    };

    let hw = width / 2.0;   // Half-width
    let hh = height / 2.0;  // Half-height

    // Generate 8 nodes: 4 corners + 4 mid-edges
    // Corners in local coords: (±hw, ±hh)
    let local_coords = [
        (-hw, -hh),  // Node 0: bottom-left corner
        ( hw, -hh),  // Node 1: bottom-right corner
        ( hw,  hh),  // Node 2: top-right corner
        (-hw,  hh),  // Node 3: top-left corner
        ( 0.0, -hh), // Node 4: bottom-center (mid-edge)
        ( hw,  0.0), // Node 5: right-center (mid-edge)
        ( 0.0,  hh), // Node 6: top-center (mid-edge)
        (-hw,  0.0), // Node 7: left-center (mid-edge)
    ];

    let mut section_nodes = Vec::with_capacity(8);

    for (i, (local_y, local_z)) in local_coords.iter().enumerate() {
        // Transform to global coordinates
        let global_pos = Vector3::new(beam_node.x, beam_node.y, beam_node.z)
            + normal * *local_y
            + binormal * *local_z;

        section_nodes.push(Node::new(
            start_id + i as i32,
            global_pos.x,
            global_pos.y,
            global_pos.z,
        ));
    }

    Ok(section_nodes)
}

/// Compute local coordinate system at a beam node
///
/// Returns (tangent, normal, binormal) as orthonormal basis vectors
fn compute_beam_local_coords(
    beam_node: &Node,
    all_beam_nodes: &[Node; 3],
    normal_vec: Vector3<f64>,
) -> Result<(Vector3<f64>, Vector3<f64>, Vector3<f64>), String> {
    // Tangent: along beam axis (from first to last node)
    let tangent = Vector3::new(
        all_beam_nodes[2].x - all_beam_nodes[0].x,
        all_beam_nodes[2].y - all_beam_nodes[0].y,
        all_beam_nodes[2].z - all_beam_nodes[0].z,
    ).normalize();

    // Normal: from beam section definition
    let normal = normal_vec.normalize();

    // Binormal: complete right-handed system
    let binormal = tangent.cross(&normal).normalize();

    // Re-orthogonalize normal (ensure perfect orthogonality)
    let normal = binormal.cross(&tangent).normalize();

    Ok((tangent, normal, binormal))
}

/// Generate C3D20R elements from expanded beam nodes
///
/// For B32R (3 beam nodes), creates 1 C3D20R element
fn generate_c3d20r_elements(
    beam_elem_id: i32,
    beam_node_mapping: &HashMap<i32, [i32; 8]>,
    beam_node_ids: &[i32],
    config: &mut BeamExpansionConfig,
) -> Result<HashMap<i32, Element>, String> {
    let mut elements = HashMap::new();

    if beam_node_ids.len() != 3 {
        return Err(format!("Expected 3 beam nodes for B32R, got {}", beam_node_ids.len()));
    }

    // Get the 8 nodes for each of the 3 beam positions
    let nodes0 = beam_node_mapping.get(&beam_node_ids[0])
        .ok_or("Missing beam node mapping for node 0")?;
    let nodes1 = beam_node_mapping.get(&beam_node_ids[1])
        .ok_or("Missing beam node mapping for node 1")?;
    let nodes2 = beam_node_mapping.get(&beam_node_ids[2])
        .ok_or("Missing beam node mapping for node 2")?;

    // C3D20R node ordering (CalculiX standard):
    // Nodes 1-8: 8 corner nodes (bottom face 1-4, top face 5-8)
    // Nodes 9-20: 12 mid-edge nodes
    //   - Nodes 9-12: bottom face mid-edges
    //   - Nodes 13-16: top face mid-edges
    //   - Nodes 17-20: vertical mid-edges (connecting bottom to top)
    //
    // For quadratic beam (3 stations along length):
    // - Station 0 (beam node 0): bottom face (corners 1-4, mid-edges 9-12)
    // - Station 1 (beam node 1): middle (vertical mid-edges 17-20)
    // - Station 2 (beam node 2): top face (corners 5-8, mid-edges 13-16)
    //
    // Section nodes: [0-3] = corners, [4-7] = mid-edges

    let c3d20r_connectivity = vec![
        // Nodes 1-4: bottom face corners
        nodes0[0], nodes0[1], nodes0[2], nodes0[3],

        // Nodes 5-8: top face corners
        nodes2[0], nodes2[1], nodes2[2], nodes2[3],

        // Nodes 9-12: bottom face mid-edges
        nodes0[4], nodes0[5], nodes0[6], nodes0[7],

        // Nodes 13-16: vertical mid-edges (bottom→top) - FIXED ORDER
        nodes1[0], nodes1[1], nodes1[2], nodes1[3],

        // Nodes 17-20: top face mid-edges - FIXED ORDER
        nodes2[4], nodes2[5], nodes2[6], nodes2[7],
    ];

    let elem = Element {
        id: config.next_element_id,
        element_type: ElementType::C3D20,  // Will use reduced integration internally
        nodes: c3d20r_connectivity,
    };

    elements.insert(elem.id, elem);
    config.next_element_id += 1;

    Ok(elements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn test_section_node_generation() {
        let beam_nodes = [
            Node::new(1, 0.0, 0.0, 0.0),
            Node::new(2, 0.0, 0.0, 5.0),
            Node::new(3, 0.0, 0.0, 10.0),
        ];

        let section = BeamSection {
            shape: SectionShape::Rectangular {
                width: 0.25,
                height: 0.25,
            },
            area: 0.0625,
            iyy: 0.0625 * 0.25_f64.powi(2) / 12.0,
            izz: 0.0625 * 0.25_f64.powi(2) / 12.0,
            torsion_constant: 0.0,
            shear_area_y: None,
            shear_area_z: None,
        };

        let normal = Vector3::new(1.0, 0.0, 0.0);

        let nodes = generate_section_nodes(&beam_nodes[0], &beam_nodes, &section, normal, 1000)
            .expect("Failed to generate section nodes");

        assert_eq!(nodes.len(), 8);

        // Check that nodes are arranged around the beam node
        for node in &nodes {
            let dist = ((node.x - 0.0).powi(2) + (node.y - 0.0).powi(2) + (node.z - 0.0).powi(2)).sqrt();
            // All nodes should be within half-diagonal of section from beam node
            assert!(dist <= 0.25 * 1.5, "Node too far from beam node: {}", dist);
        }
    }

    #[test]
    fn test_beam_expansion_config_default() {
        let config = BeamExpansionConfig::default();
        assert_eq!(config.next_node_id, 1_000_000);
        assert_eq!(config.next_element_id, 1_000_000);
    }

    #[test]
    fn test_local_coords_computation() {
        let beam_nodes = [
            Node::new(1, 0.0, 0.0, 0.0),
            Node::new(2, 0.0, 0.0, 5.0),
            Node::new(3, 0.0, 0.0, 10.0),
        ];

        let normal_vec = Vector3::new(1.0, 0.0, 0.0);

        let (tangent, normal, binormal) = compute_beam_local_coords(
            &beam_nodes[0],
            &beam_nodes,
            normal_vec,
        ).expect("Failed to compute local coords");

        // Tangent should be along Z-axis
        assert!((tangent.z - 1.0).abs() < 1e-10);

        // Normal should be along X-axis
        assert!((normal.x - 1.0).abs() < 1e-10);

        // Binormal should be along Y-axis (or -Y)
        assert!((binormal.y.abs() - 1.0).abs() < 1e-10);

        // Check orthogonality
        assert!(tangent.dot(&normal).abs() < 1e-10);
        assert!(tangent.dot(&binormal).abs() < 1e-10);
        assert!(normal.dot(&binormal).abs() < 1e-10);
    }
}
