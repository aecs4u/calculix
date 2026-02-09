//! Boundary conditions and loading for finite element analysis.
//!
//! This module handles:
//! - Displacement boundary conditions (*BOUNDARY)
//! - Concentrated loads (*CLOAD)
//! - Distributed loads (*DLOAD)
//! - Pressure loads

use std::collections::HashMap;

/// Degree of freedom index (0-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DofId {
    /// Node ID
    pub node: i32,
    /// DOF index (0 = X, 1 = Y, 2 = Z, 3+ for rotations/temp)
    pub dof: usize,
}

impl DofId {
    /// Create a new DOF identifier
    pub fn new(node: i32, dof: usize) -> Self {
        Self { node, dof }
    }
}

/// A displacement boundary condition (fixed DOF)
#[derive(Debug, Clone, PartialEq)]
pub struct DisplacementBC {
    /// Node ID
    pub node: i32,
    /// First DOF to constrain (1-based from input)
    pub first_dof: usize,
    /// Last DOF to constrain (1-based from input, inclusive)
    pub last_dof: usize,
    /// Prescribed displacement value (0.0 for fixed)
    pub value: f64,
}

impl DisplacementBC {
    /// Create a new displacement boundary condition
    pub fn new(node: i32, first_dof: usize, last_dof: usize, value: f64) -> Self {
        Self {
            node,
            first_dof,
            last_dof,
            value,
        }
    }

    /// Get all DOF IDs affected by this boundary condition (0-based)
    pub fn affected_dofs(&self) -> Vec<DofId> {
        let mut dofs = Vec::new();
        for dof in self.first_dof..=self.last_dof {
            dofs.push(DofId::new(self.node, dof - 1)); // Convert to 0-based
        }
        dofs
    }
}

/// A concentrated load on a node
#[derive(Debug, Clone, PartialEq)]
pub struct ConcentratedLoad {
    /// Node ID
    pub node: i32,
    /// DOF to load (1-based from input)
    pub dof: usize,
    /// Load magnitude
    pub magnitude: f64,
}

impl ConcentratedLoad {
    /// Create a new concentrated load
    pub fn new(node: i32, dof: usize, magnitude: f64) -> Self {
        Self {
            node,
            dof,
            magnitude,
        }
    }

    /// Get the DOF ID for this load (0-based)
    pub fn dof_id(&self) -> DofId {
        DofId::new(self.node, self.dof - 1) // Convert to 0-based
    }
}

/// Type of distributed load
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributedLoadType {
    /// Pressure load (normal to surface)
    Pressure,
    /// Centrifugal load
    Centrifugal,
    /// Gravity load
    Gravity,
    /// Body force
    BodyForce,
}

/// A distributed load on elements
#[derive(Debug, Clone, PartialEq)]
pub struct DistributedLoad {
    /// Element ID or element set name
    pub element: String,
    /// Load type
    pub load_type: DistributedLoadType,
    /// Load magnitude/parameters
    pub magnitude: f64,
    /// Additional parameters (direction vector, etc.)
    pub parameters: Vec<f64>,
}

/// Complete boundary condition and loading specification
#[derive(Debug, Clone)]
pub struct BoundaryConditions {
    /// All displacement boundary conditions
    pub displacement_bcs: Vec<DisplacementBC>,
    /// All concentrated loads
    pub concentrated_loads: Vec<ConcentratedLoad>,
    /// All distributed loads
    pub distributed_loads: Vec<DistributedLoad>,
}

impl BoundaryConditions {
    /// Create an empty boundary conditions object
    pub fn new() -> Self {
        Self {
            displacement_bcs: Vec::new(),
            concentrated_loads: Vec::new(),
            distributed_loads: Vec::new(),
        }
    }

    /// Add a displacement boundary condition
    pub fn add_displacement_bc(&mut self, bc: DisplacementBC) {
        self.displacement_bcs.push(bc);
    }

    /// Add a concentrated load
    pub fn add_concentrated_load(&mut self, load: ConcentratedLoad) {
        self.concentrated_loads.push(load);
    }

    /// Add a distributed load
    pub fn add_distributed_load(&mut self, load: DistributedLoad) {
        self.distributed_loads.push(load);
    }

    /// Get all constrained DOFs as a map (DOF -> prescribed value)
    pub fn get_constrained_dofs(&self) -> HashMap<DofId, f64> {
        let mut constrained = HashMap::new();

        for bc in &self.displacement_bcs {
            for dof_id in bc.affected_dofs() {
                constrained.insert(dof_id, bc.value);
            }
        }

        constrained
    }

    /// Get all nodal loads as a map (DOF -> total load)
    pub fn get_nodal_loads(&self) -> HashMap<DofId, f64> {
        let mut loads = HashMap::new();

        for load in &self.concentrated_loads {
            let dof_id = load.dof_id();
            *loads.entry(dof_id).or_insert(0.0) += load.magnitude;
        }

        loads
    }

    /// Get statistics
    pub fn statistics(&self) -> BCStatistics {
        let num_constrained_dofs = self
            .displacement_bcs
            .iter()
            .map(|bc| bc.affected_dofs().len())
            .sum();

        BCStatistics {
            num_displacement_bcs: self.displacement_bcs.len(),
            num_constrained_dofs,
            num_concentrated_loads: self.concentrated_loads.len(),
            num_distributed_loads: self.distributed_loads.len(),
        }
    }
}

impl Default for BoundaryConditions {
    fn default() -> Self {
        Self::new()
    }
}

/// Boundary condition statistics
#[derive(Debug, Clone)]
pub struct BCStatistics {
    /// Number of displacement BC entries
    pub num_displacement_bcs: usize,
    /// Total number of constrained DOFs
    pub num_constrained_dofs: usize,
    /// Number of concentrated loads
    pub num_concentrated_loads: usize,
    /// Number of distributed loads
    pub num_distributed_loads: usize,
}

impl BCStatistics {
    /// Format as a human-readable string
    pub fn format(&self) -> String {
        format!(
            "BCs: {} displacement entries ({} DOFs), {} concentrated loads, {} distributed loads",
            self.num_displacement_bcs,
            self.num_constrained_dofs,
            self.num_concentrated_loads,
            self.num_distributed_loads
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displacement_bc_affects_correct_dofs() {
        let bc = DisplacementBC::new(10, 1, 3, 0.0);
        let dofs = bc.affected_dofs();

        assert_eq!(dofs.len(), 3);
        assert_eq!(dofs[0], DofId::new(10, 0)); // X direction
        assert_eq!(dofs[1], DofId::new(10, 1)); // Y direction
        assert_eq!(dofs[2], DofId::new(10, 2)); // Z direction
    }

    #[test]
    fn displacement_bc_single_dof() {
        let bc = DisplacementBC::new(5, 2, 2, 1.5);
        let dofs = bc.affected_dofs();

        assert_eq!(dofs.len(), 1);
        assert_eq!(dofs[0], DofId::new(5, 1)); // Y direction (2 -> 1 in 0-based)
    }

    #[test]
    fn concentrated_load_dof_id() {
        let load = ConcentratedLoad::new(20, 3, 100.0);
        let dof_id = load.dof_id();

        assert_eq!(dof_id.node, 20);
        assert_eq!(dof_id.dof, 2); // Z direction (3 -> 2 in 0-based)
    }

    #[test]
    fn boundary_conditions_constrained_dofs() {
        let mut bcs = BoundaryConditions::new();

        bcs.add_displacement_bc(DisplacementBC::new(1, 1, 2, 0.0)); // Fix X, Y
        bcs.add_displacement_bc(DisplacementBC::new(2, 3, 3, 0.0)); // Fix Z

        let constrained = bcs.get_constrained_dofs();
        assert_eq!(constrained.len(), 3);
        assert_eq!(constrained.get(&DofId::new(1, 0)), Some(&0.0)); // Node 1, X
        assert_eq!(constrained.get(&DofId::new(1, 1)), Some(&0.0)); // Node 1, Y
        assert_eq!(constrained.get(&DofId::new(2, 2)), Some(&0.0)); // Node 2, Z
    }

    #[test]
    fn boundary_conditions_nodal_loads() {
        let mut bcs = BoundaryConditions::new();

        bcs.add_concentrated_load(ConcentratedLoad::new(1, 1, 100.0));
        bcs.add_concentrated_load(ConcentratedLoad::new(1, 1, 50.0)); // Accumulate
        bcs.add_concentrated_load(ConcentratedLoad::new(2, 2, 200.0));

        let loads = bcs.get_nodal_loads();
        assert_eq!(loads.len(), 2);
        assert_eq!(loads.get(&DofId::new(1, 0)), Some(&150.0)); // Node 1, X: 100+50
        assert_eq!(loads.get(&DofId::new(2, 1)), Some(&200.0)); // Node 2, Y
    }

    #[test]
    fn boundary_conditions_statistics() {
        let mut bcs = BoundaryConditions::new();

        bcs.add_displacement_bc(DisplacementBC::new(1, 1, 3, 0.0)); // 3 DOFs
        bcs.add_displacement_bc(DisplacementBC::new(2, 2, 2, 0.0)); // 1 DOF
        bcs.add_concentrated_load(ConcentratedLoad::new(3, 1, 100.0));
        bcs.add_concentrated_load(ConcentratedLoad::new(4, 2, 200.0));

        let stats = bcs.statistics();
        assert_eq!(stats.num_displacement_bcs, 2);
        assert_eq!(stats.num_constrained_dofs, 4);
        assert_eq!(stats.num_concentrated_loads, 2);
        assert_eq!(stats.num_distributed_loads, 0);
    }

    #[test]
    fn prescribed_displacement_non_zero() {
        let bc = DisplacementBC::new(10, 1, 1, 2.5);
        let constrained = BoundaryConditions {
            displacement_bcs: vec![bc],
            concentrated_loads: vec![],
            distributed_loads: vec![],
        }
        .get_constrained_dofs();

        assert_eq!(constrained.get(&DofId::new(10, 0)), Some(&2.5));
    }
}
