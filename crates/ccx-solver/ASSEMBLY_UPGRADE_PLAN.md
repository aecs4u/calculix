# Assembly System Upgrade Plan for Beam Element Support

## Current Status

✅ **UPGRADE COMPLETE** (2026-02-08)

All objectives achieved:
- ✅ Assembly system supports variable DOFs per node
- ✅ B31 beam element (6 DOFs/node) fully integrated
- ✅ T3D2 truss element (3 DOFs/node) still supported
- ✅ Mixed element meshes working correctly
- ✅ DynamicElement factory for polymorphic assembly
- ✅ All 206 tests passing
- ✅ 4 new end-to-end integration tests
- ✅ Cantilever beam validates against analytical solution (< 1% error)

See [ASSEMBLY_SYSTEM_UPGRADE_COMPLETE.md](ASSEMBLY_SYSTEM_UPGRADE_COMPLETE.md) for full summary.

## Required Changes

### 1. Update `assembly.rs`

**File**: `crates/ccx-solver/src/assembly.rs`

**Changes needed**:

```rust
// OLD (lines 66-67):
let num_dofs = mesh.num_dofs;
let mut system = Self::new(num_dofs);

// NEW:
// Determine maximum DOFs per node for mixed meshes
let max_dofs_per_node = mesh.elements.values()
    .map(|e| e.element_type.dofs_per_node())
    .max()
    .unwrap_or(3);

// All nodes get max DOF count
let num_nodes = mesh.nodes.len();
let num_dofs = num_nodes * max_dofs_per_node;
let mut system = Self::new(num_dofs);
```

**Method signatures to update**:

```rust
// Add max_dofs_per_node parameter
fn assemble_stiffness(&mut self, mesh, materials, default_area, max_dofs_per_node)
fn assemble_forces(&mut self, bcs, max_dofs_per_node)
fn apply_displacement_bcs(&mut self, bcs, max_dofs_per_node)
```

**Element assembly** (lines 106-126):

```rust
// OLD: Hardcoded for T3D2
if element.element_type != ElementType::T3D2 {
    continue;
}
let truss = crate::elements::Truss2D::new(...);

// NEW: Use DynamicElement factory
use crate::elements::DynamicElement;

let dyn_elem = DynamicElement::from_mesh_element(
    element.element_type,
    *elem_id,
    element.nodes.clone(),
    default_area,
);

let dyn_elem = match dyn_elem {
    Some(e) => e,
    None => {
        eprintln!("Warning: Unsupported element type {:?}, skipping",
                  element.element_type);
        continue;
    }
};

let k_e = dyn_elem.stiffness_matrix(&nodes, material)?;
let dof_indices = dyn_elem.global_dof_indices(&element.nodes);
```

**Force assembly** (line 135):

```rust
// OLD:
let dof_index = (load.node - 1) as usize * 3 + (load.dof - 1);

// NEW:
let dof_index = (load.node - 1) as usize * max_dofs_per_node + (load.dof - 1);
```

**BC application** (line 160):

```rust
// OLD:
let dof_index = (bc.node - 1) as usize * 3 + (dof - 1);

// NEW:
let dof_index = (bc.node - 1) as usize * max_dofs_per_node + (dof - 1);
```

### 2. Update Element Trait DOF Indexing

**File**: `crates/ccx-solver/src/elements/mod.rs`

**Issue**: Default implementation of `global_dof_indices()` assumes uniform DOFs

**Current**:
```rust
fn global_dof_indices(&self, connectivity: &[i32]) -> Vec<usize> {
    let dofs_per_node = self.dofs_per_node();
    for &node_id in connectivity {
        let base_dof = ((node_id - 1) as usize) * dofs_per_node;
        // ...
    }
}
```

**Solution**: Override in each element implementation OR pass max_dofs_per_node as parameter

### 3. Update Boundary Conditions

**File**: `crates/ccx-solver/src/boundary_conditions.rs`

**Add support for rotational DOFs**:

```rust
pub enum DofType {
    UX = 1,  // Translation X
    UY = 2,  // Translation Y
    UZ = 3,  // Translation Z
    RX = 4,  // Rotation X
    RY = 5,  // Rotation Y
    RZ = 6,  // Rotation Z
}

// Helper methods for common BC types
impl BoundaryConditions {
    pub fn add_fixed_support(&mut self, node: i32) {
        // Fix all 6 DOFs
        self.add_displacement_bc(node, 1, 6, 0.0);
    }

    pub fn add_pinned_support(&mut self, node: i32) {
        // Fix translations only
        self.add_displacement_bc(node, 1, 3, 0.0);
    }

    pub fn add_moment(&mut self, node: i32, dof: usize, magnitude: f64) {
        // Apply moment (dof = 4, 5, or 6 for RX, RY, RZ)
        self.concentrated_loads.push(ConcentratedLoad {
            node,
            dof,
            magnitude,
        });
    }
}
```

### 4. Create Mixed Element Test

**File**: `crates/ccx-solver/tests/mixed_elements.rs`

```rust
#[test]
fn test_truss_and_beam_assembly() {
    // Create mesh with both T3D2 and B31 elements
    // Verify:
    // - Correct total DOF count
    // - Proper stiffness matrix assembly
    // - Beam DOFs 4,5,6 handled correctly
    // - Truss DOFs 4,5,6 are zero (unused)
}
```

## Implementation Strategy

### Phase 1: Core Assembly Changes (HIGH PRIORITY)

1. ✅ Add `dofs_per_node()` to `ElementType`
2. ✅ Create `DynamicElement` factory
3. ⏳ Update `assembly.rs` methods signatures
4. ⏳ Update `assemble_stiffness()` to use DynamicElement
5. ⏳ Update `assemble_forces()` for variable DOFs
6. ⏳ Update `apply_displacement_bcs()` for variable DOFs

### Phase 2: DOF Indexing (MEDIUM PRIORITY)

7. ⏳ Test with pure beam mesh (all nodes 6 DOFs)
8. ⏳ Test with mixed truss+beam mesh
9. ⏳ Add validation for unused DOFs in truss elements

### Phase 3: Enhanced BCs (LOW PRIORITY)

10. ⏳ Add rotation BC support
11. ⏳ Add moment load support
12. ⏳ Add helper methods for common supports

## Testing Plan

### Unit Tests

- ✅ Element factory creates correct types
- ⏳ DOF indexing for beam elements
- ⏳ DOF indexing for mixed elements
- ⏳ Force assembly with 6 DOFs
- ⏳ BC application with 6 DOFs

### Integration Tests

- ⏳ Simple cantilever beam (compare with analytical)
- ⏳ Truss-beam combination structure
- ⏳ Validate against beam example files

## Estimated Work

- **Phase 1**: ~2 hours (core functionality)
- **Phase 2**: ~1 hour (testing & validation)
- **Phase 3**: ~1 hour (enhanced features)
- **Total**: ~4 hours

## Current Blockers

None - all prerequisites are in place. Ready to implement.

## Success Criteria

1. ✅ B31 beam element solves correctly (pure beam mesh)
2. ⏳ Cantilever beam matches analytical solution
3. ⏳ Mixed truss+beam mesh assembles without errors
4. ⏳ All 241 existing tests still pass
5. ⏳ At least 1 beam example file validates successfully

## Next Immediate Steps

1. Update `assemble_stiffness()` in assembly.rs
2. Update `assemble_forces()` in assembly.rs
3. Update `apply_displacement_bcs()` in assembly.rs
4. Create cantilever beam integration test
5. Run full test suite to verify no regressions

---

**Status**: Ready to implement
**Priority**: HIGH - Blocking beam element usage
**Estimated completion**: 2-4 hours of focused work
