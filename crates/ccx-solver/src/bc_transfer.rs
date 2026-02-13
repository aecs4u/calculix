//! Boundary condition and load transfer for beam expansion
//!
//! When beam elements are expanded to C3D20R solid elements, the boundary conditions
//! and loads must be transferred from the original beam nodes to the expanded section nodes.
//!
//! # Strategy
//!
//! - **Displacement BCs**: Apply to ALL 8 section nodes (preserves constraint)
//! - **Concentrated loads**: Distribute equally among 8 nodes (statically equivalent)
//!
//! # Example
//!
//! ```text
//! Beam node 1 (fixed) → Expands to 8 section nodes
//! Original BC: Node 1, DOFs 1-6, value=0.0
//! Transferred: All 8 nodes, DOFs 1-3, value=0.0 (3 DOFs per solid node)
//! ```

use std::collections::HashMap;
use crate::boundary_conditions::{BoundaryConditions, DisplacementBC, ConcentratedLoad};

/// Handles transfer of BCs and loads from beam nodes to expanded section nodes
pub struct BCTransfer {
    /// Maps beam node ID → [8 section node IDs]
    beam_node_mapping: HashMap<i32, [i32; 8]>,
}

impl BCTransfer {
    /// Create a new BC transfer handler
    ///
    /// # Arguments
    /// * `beam_node_mapping` - Mapping from beam nodes to their 8 expanded section nodes
    pub fn new(beam_node_mapping: HashMap<i32, [i32; 8]>) -> Self {
        Self { beam_node_mapping }
    }

    /// Transfer displacement boundary conditions from beam nodes to section nodes
    ///
    /// # Strategy
    /// - If a beam node has a displacement BC, apply it to ALL 8 section nodes
    /// - Only transfer DOFs 1-3 (translations), as C3D20R has only 3 DOFs/node
    /// - Non-beam nodes: copy BCs as-is
    ///
    /// # Arguments
    /// * `original_bcs` - Original boundary conditions (before expansion)
    ///
    /// # Returns
    /// New boundary conditions with BCs transferred to section nodes
    pub fn transfer_displacement_bcs(&self, original_bcs: &BoundaryConditions) -> BoundaryConditions {
        let mut new_bcs = BoundaryConditions::new();

        for bc in &original_bcs.displacement_bcs {
            if let Some(section_nodes) = self.beam_node_mapping.get(&bc.node) {
                // This is a beam node → transfer to all 8 section nodes
                for &section_node_id in section_nodes {
                    // Only transfer translational DOFs (1-3) since C3D20R has 3 DOFs/node
                    let first_dof = bc.first_dof.min(3);
                    let last_dof = bc.last_dof.min(3);

                    if first_dof <= 3 {
                        new_bcs.add_displacement_bc(DisplacementBC::new(
                            section_node_id,
                            first_dof,
                            last_dof,
                            bc.value,
                        ));
                    }
                }
            } else {
                // Non-beam node → copy as-is
                new_bcs.add_displacement_bc(bc.clone());
            }
        }

        new_bcs
    }

    /// Transfer concentrated loads from beam nodes to section nodes
    ///
    /// # Strategy
    /// - Distribute load equally among 8 section nodes (each gets load/8)
    /// - Ensures ∑F = F_total (statically equivalent)
    /// - Only transfer translational DOFs (1-3)
    /// - Non-beam nodes: copy loads as-is
    ///
    /// # Arguments
    /// * `original_bcs` - Original boundary conditions with loads (before expansion)
    ///
    /// # Returns
    /// New boundary conditions with loads transferred to section nodes
    pub fn transfer_concentrated_loads(&self, original_bcs: &BoundaryConditions) -> BoundaryConditions {
        let mut new_bcs = BoundaryConditions::new();

        // Copy displacement BCs as-is (they're transferred separately)
        for bc in &original_bcs.displacement_bcs {
            new_bcs.add_displacement_bc(bc.clone());
        }

        // Transfer concentrated loads
        for load in &original_bcs.concentrated_loads {
            if let Some(section_nodes) = self.beam_node_mapping.get(&load.node) {
                // This is a beam node → distribute load among 8 section nodes
                // Only transfer translational DOFs (1-3)
                if load.dof <= 3 {
                    let load_per_node = load.magnitude / 8.0;
                    for &section_node_id in section_nodes {
                        new_bcs.add_concentrated_load(ConcentratedLoad {
                            node: section_node_id,
                            dof: load.dof,
                            magnitude: load_per_node,
                        });
                    }
                }
                // Note: Rotational loads (DOF 4-6) are ignored for C3D20R
            } else {
                // Non-beam node → copy as-is
                new_bcs.add_concentrated_load(load.clone());
            }
        }

        // Copy distributed loads as-is (not affected by beam expansion)
        for load in &original_bcs.distributed_loads {
            new_bcs.add_distributed_load(load.clone());
        }

        new_bcs
    }

    /// Complete transfer: both BCs and loads
    ///
    /// Convenience method that transfers both displacement BCs and loads in one call.
    ///
    /// # Arguments
    /// * `original_bcs` - Original boundary conditions (before expansion)
    ///
    /// # Returns
    /// New boundary conditions with both BCs and loads transferred
    pub fn transfer_all(&self, original_bcs: &BoundaryConditions) -> BoundaryConditions {
        let mut new_bcs = BoundaryConditions::new();

        // Transfer displacement BCs
        for bc in &original_bcs.displacement_bcs {
            if let Some(section_nodes) = self.beam_node_mapping.get(&bc.node) {
                // This is a beam node → transfer to all 8 section nodes
                for &section_node_id in section_nodes {
                    let first_dof = bc.first_dof.min(3);
                    let last_dof = bc.last_dof.min(3);
                    if first_dof <= 3 {
                        new_bcs.add_displacement_bc(DisplacementBC::new(
                            section_node_id,
                            first_dof,
                            last_dof,
                            bc.value,
                        ));
                    }
                }
            } else {
                // Non-beam node → copy as-is
                new_bcs.add_displacement_bc(bc.clone());
            }
        }

        // Transfer concentrated loads
        for load in &original_bcs.concentrated_loads {
            if let Some(section_nodes) = self.beam_node_mapping.get(&load.node) {
                // This is a beam node → distribute load among 8 section nodes
                if load.dof <= 3 {
                    let load_per_node = load.magnitude / 8.0;
                    for &section_node_id in section_nodes {
                        new_bcs.add_concentrated_load(ConcentratedLoad {
                            node: section_node_id,
                            dof: load.dof,
                            magnitude: load_per_node,
                        });
                    }
                }
            } else {
                // Non-beam node → copy as-is
                new_bcs.add_concentrated_load(load.clone());
            }
        }

        // Copy distributed loads as-is
        for load in &original_bcs.distributed_loads {
            new_bcs.add_distributed_load(load.clone());
        }

        new_bcs
    }

    /// Get statistics about the transfer
    pub fn statistics(&self) -> String {
        format!(
            "BC Transfer: {} beam nodes → {} section nodes total",
            self.beam_node_mapping.len(),
            self.beam_node_mapping.len() * 8
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary_conditions::{DisplacementBC, ConcentratedLoad};

    #[test]
    fn test_displacement_bc_transfer() {
        // Setup: Beam node 1 expands to nodes 1000-1007
        let mut mapping = HashMap::new();
        mapping.insert(1, [1000, 1001, 1002, 1003, 1004, 1005, 1006, 1007]);

        let transfer = BCTransfer::new(mapping);

        // Original: Fix node 1 in all 6 DOFs
        let mut original_bcs = BoundaryConditions::new();
        original_bcs.add_displacement_bc(DisplacementBC::new(1, 1, 6, 0.0));

        // Transfer
        let new_bcs = transfer.transfer_displacement_bcs(&original_bcs);

        // Should create 8 BCs (one for each section node)
        assert_eq!(new_bcs.displacement_bcs.len(), 8);

        // Each should fix DOFs 1-3 (not 4-6, since C3D20R has only 3 DOFs/node)
        for bc in &new_bcs.displacement_bcs {
            assert!(bc.node >= 1000 && bc.node <= 1007);
            assert_eq!(bc.first_dof, 1);
            assert_eq!(bc.last_dof, 3);
            assert_eq!(bc.value, 0.0);
        }
    }

    #[test]
    fn test_concentrated_load_transfer() {
        // Setup: Beam node 1 expands to nodes 1000-1007
        let mut mapping = HashMap::new();
        mapping.insert(1, [1000, 1001, 1002, 1003, 1004, 1005, 1006, 1007]);

        let transfer = BCTransfer::new(mapping);

        // Original: Apply load of 1.0 N in DOF 1 at node 1
        let mut original_bcs = BoundaryConditions::new();
        original_bcs.add_concentrated_load(ConcentratedLoad {
            node: 1,
            dof: 1,
            magnitude: 1.0,
        });

        // Transfer
        let new_bcs = transfer.transfer_concentrated_loads(&original_bcs);

        // Should create 8 loads (one for each section node)
        assert_eq!(new_bcs.concentrated_loads.len(), 8);

        // Each should have magnitude 1.0/8 = 0.125
        let mut total_load = 0.0;
        for load in &new_bcs.concentrated_loads {
            assert!(load.node >= 1000 && load.node <= 1007);
            assert_eq!(load.dof, 1);
            assert_eq!(load.magnitude, 0.125);
            total_load += load.magnitude;
        }

        // Total load should equal original load
        assert!((total_load - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_non_beam_node_passthrough() {
        // Setup: Only beam node 1 is expanded
        let mut mapping = HashMap::new();
        mapping.insert(1, [1000, 1001, 1002, 1003, 1004, 1005, 1006, 1007]);

        let transfer = BCTransfer::new(mapping);

        // Original: BC on node 2 (not a beam node)
        let mut original_bcs = BoundaryConditions::new();
        original_bcs.add_displacement_bc(DisplacementBC::new(2, 1, 3, 0.0));

        // Transfer
        let new_bcs = transfer.transfer_displacement_bcs(&original_bcs);

        // Should have exactly 1 BC (unchanged)
        assert_eq!(new_bcs.displacement_bcs.len(), 1);
        assert_eq!(new_bcs.displacement_bcs[0].node, 2);
        assert_eq!(new_bcs.displacement_bcs[0].first_dof, 1);
        assert_eq!(new_bcs.displacement_bcs[0].last_dof, 3);
    }

    #[test]
    fn test_transfer_all() {
        // Setup: Beam node 1 expands to nodes 1000-1007
        let mut mapping = HashMap::new();
        mapping.insert(1, [1000, 1001, 1002, 1003, 1004, 1005, 1006, 1007]);

        let transfer = BCTransfer::new(mapping);

        // Original: Fixed beam node 1 with load
        let mut original_bcs = BoundaryConditions::new();
        original_bcs.add_displacement_bc(DisplacementBC::new(1, 1, 6, 0.0));
        original_bcs.add_concentrated_load(ConcentratedLoad {
            node: 1,
            dof: 1,
            magnitude: 8.0,
        });

        // Transfer both
        let new_bcs = transfer.transfer_all(&original_bcs);

        // Should have 8 displacement BCs and 8 loads
        assert_eq!(new_bcs.displacement_bcs.len(), 8);
        assert_eq!(new_bcs.concentrated_loads.len(), 8);

        // Total load should be preserved
        let total_load: f64 = new_bcs.concentrated_loads.iter().map(|l| l.magnitude).sum();
        assert!((total_load - 8.0).abs() < 1e-10);
    }
}
