# C3D20R Node Ordering Fix

## Problem Identified ✅

**Error**: `Negative Jacobian determinant at point 0: -0.10104`
**Cause**: Incorrect node ordering in beam expansion → C3D20R

## Root Cause Analysis

### Current (INCORRECT) Implementation

`crates/ccx-solver/src/elements/beam_expansion.rs` lines 243-252:

```rust
let c3d20r_connectivity = vec![
    // First 8: bottom face corners and mid-edges (from beam node 0)
    nodes0[0], nodes0[1], nodes0[2], nodes0[3],
    nodes0[4], nodes0[5], nodes0[6], nodes0[7],
    // Next 8: top face corners and mid-edges (from beam node 2)
    nodes2[0], nodes2[1], nodes2[2], nodes2[3],
    nodes2[4], nodes2[5], nodes2[6], nodes2[7],
    // Last 4: mid-length edges (from beam node 1)
    nodes1[0], nodes1[1], nodes1[2], nodes1[3],
];
```

**Issue**: This treats each cross-section as "4 corners + 4 mid-edges", which doesn't match C3D20 standard.

### CalculiX C3D20 Standard Ordering

From CalculiX documentation (found in `improveMesh.c`):
- **Nodes 1-8**: Corner nodes (8 total)
  - Nodes 1-4: Bottom face corners
  - Nodes 5-8: Top face corners
- **Nodes 9-20**: Mid-edge nodes (12 total)
  - Nodes 9-12: Bottom face mid-edges (between corners 1-2, 2-3, 3-4, 4-1)
  - Nodes 13-16: Top face mid-edges (between corners 5-6, 6-7, 7-8, 8-5)
  - Nodes 17-20: Vertical mid-edges (between bottom-top: 1-5, 2-6, 3-7, 4-8)

### Section Node Layout (from beam_expansion.rs lines 152-162)

Current section nodes generated at each beam station:
```rust
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
```

**Pattern**: Each station has 4 corners (0-3) + 4 mid-edges (4-7)

## Correct Mapping

### Beam Stations
- **Station 0** (beam node 0): Bottom face of C3D20
- **Station 1** (beam node 1): Middle (for vertical mid-edges)
- **Station 2** (beam node 2): Top face of C3D20

### Correct C3D20 Connectivity

```rust
let c3d20r_connectivity = vec![
    // Nodes 1-4: Bottom face CORNERS ONLY
    nodes0[0], nodes0[1], nodes0[2], nodes0[3],

    // Nodes 5-8: Top face CORNERS ONLY
    nodes2[0], nodes2[1], nodes2[2], nodes2[3],

    // Nodes 9-12: Bottom face MID-EDGES
    nodes0[4], nodes0[5], nodes0[6], nodes0[7],

    // Nodes 13-16: Top face MID-EDGES
    nodes2[4], nodes2[5], nodes2[6], nodes2[7],

    // Nodes 17-20: Vertical MID-EDGES (connecting bottom to top)
    nodes1[0], nodes1[1], nodes1[2], nodes1[3],
];
```

## Implementation Fix

**File**: `crates/ccx-solver/src/elements/beam_expansion.rs`
**Function**: `generate_c3d20r_elements()`
**Lines**: 243-252

### Before (Incorrect):
```rust
let c3d20r_connectivity = vec![
    // First 8: bottom face corners and mid-edges (from beam node 0)
    nodes0[0], nodes0[1], nodes0[2], nodes0[3],
    nodes0[4], nodes0[5], nodes0[6], nodes0[7],
    // Next 8: top face corners and mid-edges (from beam node 2)
    nodes2[0], nodes2[1], nodes2[2], nodes2[3],
    nodes2[4], nodes2[5], nodes2[6], nodes2[7],
    // Last 4: mid-length edges (from beam node 1)
    nodes1[0], nodes1[1], nodes1[2], nodes1[3],
];
```

### After (Correct):
```rust
let c3d20r_connectivity = vec![
    // Nodes 1-4: bottom face corners
    nodes0[0], nodes0[1], nodes0[2], nodes0[3],

    // Nodes 5-8: top face corners
    nodes2[0], nodes2[1], nodes2[2], nodes2[3],

    // Nodes 9-12: bottom face mid-edges
    nodes0[4], nodes0[5], nodes0[6], nodes0[7],

    // Nodes 13-16: top face mid-edges
    nodes2[4], nodes2[5], nodes2[6], nodes2[7],

    // Nodes 17-20: vertical mid-edges (bottom→top)
    nodes1[0], nodes1[1], nodes1[2], nodes1[3],
];
```

## Verification

### Expected Results After Fix:
1. ✅ Positive Jacobian determinant at all integration points
2. ✅ Assembly succeeds
3. ✅ Solver produces solution
4. ✅ Displacements match expected beam behavior

### Test Command:
```bash
CCX_EXPAND_B32R=1 ccx-cli solve tests/fixtures/solver/simplebeam.inp
```

### Success Criteria:
- No "Negative Jacobian" error
- Message contains "[SOLVED]"
- DAT file output generated

## Impact

**Files to modify**: 1
- `crates/ccx-solver/src/elements/beam_expansion.rs` (1 function, 9 lines)

**Risk**: Low - Simple reordering, no logic changes

**Time estimate**: 5 minutes to fix, 2 minutes to test

## References

- CalculiX CG Manual: C3D20 element definition
- `calculix_migration_tooling/cgx_2.23/src/improveMesh.c`: Lines 716-717, 954-955
- Standard hex20 isoparametric element node numbering

## Status

- ✅ Problem identified (negative Jacobian)
- ✅ Root cause found (incorrect node ordering)
- ✅ Solution designed (reorder connectivity vector)
- ⏳ Implementation pending
- ⏳ Testing pending
