# C3D20R Beam Expansion Implementation Status

## Overview
Implementation of CalculiX's B32R→C3D20R beam expansion strategy for exact stress matching with reference outputs.

**Date**: 2026-02-11
**Status**: Phase 1 Complete (Foundational Infrastructure)
**Compilation**: ✅ Success (81 warnings, 0 errors)

---

## Phase 1: Foundational Components (COMPLETE)

### 1.1 C3D20/C3D20R Element Implementation ✅

**File**: `crates/ccx-solver/src/elements/solid20.rs` (373 lines)

**Completed Features**:
- ✅ Complete serendipity shape functions for 20 nodes (8 corners + 12 mid-edges)
- ✅ Full shape function derivatives (all 20 nodes, 3 directions each)
- ✅ Jacobian matrix computation and inversion
- ✅ B-matrix (strain-displacement) for 6×60 system
- ✅ Constitutive matrix for 3D linear elasticity (Lamé parameters)
- ✅ Stiffness matrix via numerical integration
- ✅ Mass matrix via numerical integration
- ✅ **Two integration schemes**:
  - 27-point Gauss (3×3×3) for C3D20 full integration
  - 8-point Gauss (2×2×2) for C3D20R reduced integration
- ✅ Element trait implementation (stiffness_matrix, mass_matrix, etc.)
- ✅ Unit tests for shape functions and integration

**Key Implementation Details**:
```rust
// Toggle reduced integration via flag
pub struct C3D20 {
    pub id: i32,
    pub nodes: [i32; 20],
    pub reduced_integration: bool,  // Controls 8-pt vs 27-pt integration
}

// Create C3D20R (for beam expansion)
let elem = C3D20::new_reduced(id, nodes);
```

**Integration Points**:
- C3D20R (reduced): 8 points at ξ,η,ζ = ±1/√3
- C3D20 (full): 27 points using 3-point Gauss-Legendre

### 1.2 Beam Expansion Module ✅

**File**: `crates/ccx-solver/src/elements/beam_expansion.rs` (311 lines)

**Completed Features**:
- ✅ B32R→C3D20R expansion function
- ✅ Cross-section node generation (8 nodes per beam node)
- ✅ Local coordinate system computation (tangent, normal, binormal)
- ✅ Rectangular section support
- ✅ C3D20R element connectivity generation
- ✅ Beam node mapping tracking
- ✅ Configuration for avoiding node/element ID conflicts
- ✅ Unit tests for section nodes and local coordinates

**Architecture**:
```
B32R (3 beam nodes) → 3 × 8 section nodes = 24 total nodes
                    → 1 C3D20R element (20-node connectivity)
```

**Node Expansion Pattern** (per beam node):
```
Cross-section view (looking along beam):
  3-------2
  |       |
  7   *   5  (* = beam node)
  |       |
  0-------1

Plus 4 mid-edge nodes: 4 (bottom), 5 (right), 6 (top), 7 (left)
```

**API**:
```rust
pub fn expand_b32r(
    beam_elem: &Element,
    beam_nodes: &[Node; 3],
    section: &BeamSection,
    normal: Vector3<f64>,
    config: &mut BeamExpansionConfig,
) -> Result<ExpansionResult, String>
```

### 1.3 Module Integration ✅

**Modified Files**:
- `crates/ccx-solver/src/elements/mod.rs`
  - ✅ Exported `C3D20`, `expand_b32r`, `BeamExpansionConfig`, `ExpansionResult`
  - ✅ Exported `SectionShape` for expansion API

**Mesh Support**:
- `crates/ccx-solver/src/mesh.rs`
  - ✅ `ElementType::C3D20` already exists
  - ✅ `from_calculix_type()` handles both "C3D20" and "C3D20R"

---

## Phase 2: Solver Integration (TODO)

### 2.1 INP Parser Integration
**Estimate**: 2-3 hours

**Tasks**:
- [ ] Parse beam normal direction from `*BEAM SECTION` card
- [ ] Store normal vector in beam section data structure
- [ ] Validate section type (only RECT supported initially)

**Files to modify**:
- `crates/ccx-io/src/inp/parser.rs`
- Beam section parsing logic

### 2.2 Solve Command Integration
**Estimate**: 4-6 hours

**Tasks**:
- [ ] Detect B32/B32R elements in input mesh
- [ ] Call `expand_b32r()` for each B32R element
- [ ] Merge expanded nodes/elements into global mesh
- [ ] Update boundary conditions to refer to expanded nodes
- [ ] Update load application to expanded nodes
- [ ] Assemble global system using C3D20R elements
- [ ] Solve and map displacements back to beam nodes

**Challenges**:
- Boundary condition mapping: Beam node BCs → 8 section node BCs
- Load application: Point load on beam → distributed load on section nodes
- Output mapping: Extract beam node results from 24-node solution

**Files to modify**:
- `crates/ccx-cli/src/main.rs` (solve command)
- `crates/ccx-solver/src/assembly.rs` (if needed)

### 2.3 Stress Recovery for C3D20R
**Estimate**: 3-4 hours

**Tasks**:
- [ ] Implement stress computation at integration points
- [ ] Use B-matrix and constitutive matrix: σ = D * B * u
- [ ] Extrapolate stresses from integration points to nodes
- [ ] Output in CalculiX DAT format
- [ ] Match reference stress ordering and formatting

**Formula**:
```
ε = B * u_e          (strain from element displacements)
σ = D * ε            (stress from constitutive law)
```

**Files to create/modify**:
- `crates/ccx-solver/src/elements/solid20_stress.rs` (new)
- `crates/ccx-cli/src/main.rs` (stress output)

---

## Phase 3: Validation (TODO)

### 3.1 Simple Beam Test
**Estimate**: 2 hours

**Test**: `tests/fixtures/solver/simplebeam.inp`
- 1 B32R element, cantilever, 1N load in X
- Reference: `validation/solver/simplebeam.dat.ref`

**Validation Criteria**:
- [ ] Displacements within 1% of reference
- [ ] Stress magnitudes within 5% of reference
- [ ] Full 6-component stress state (sxx, syy, szz, sxy, sxz, syz)
- [ ] Volume calculation matches exactly

### 3.2 Multi-Element Beam Test
**Estimate**: 1 hour

**Test**: Create 3-element B32R beam
- Validate continuity between elements
- Check stress distribution along length

### 3.3 Regression Tests
**Estimate**: 2-3 hours

**Tasks**:
- [ ] Run all 204 beam examples from `examples/Beam/`
- [ ] Compare outputs with CalculiX reference
- [ ] Document any discrepancies
- [ ] Update validation database

---

## Technical Challenges & Solutions

### Challenge 1: Shape Function Derivatives
**Issue**: Serendipity elements have complex derivative formulas
**Solution**: ✅ Implemented all 20 nodes × 3 directions = 60 derivatives manually
**Lines**: 88-119 in solid20.rs

### Challenge 2: Integration Scheme Selection
**Issue**: C3D20R needs reduced integration to match CalculiX
**Solution**: ✅ Added `reduced_integration` flag with dual scheme support
**Impact**: Avoids volumetric locking, matches CalculiX's element behavior

### Challenge 3: Beam Section API Mismatch
**Issue**: `BeamSection` struct doesn't store normal direction
**Solution**: ✅ Pass normal as separate parameter to `expand_b32r()`
**Trade-off**: Slightly less elegant API, but avoids changing existing structures

### Challenge 4: Node ID Conflicts
**Issue**: Expanded nodes need unique IDs that don't clash with input
**Solution**: ✅ `BeamExpansionConfig` starts IDs at 1,000,000
**Configurable**: User can override if needed

---

## Testing Status

### Unit Tests
**Status**: ✅ All passing

**Coverage**:
- C3D20 shape functions (sum to 1, partition of unity)
- Integration weights (sum to 8 for reference cube)
- Section node generation (distance checks)
- Local coordinate system (orthogonality)

**Run**:
```bash
cargo test --package ccx-solver --lib solid20
cargo test --package ccx-solver --lib beam_expansion
```

### Integration Tests
**Status**: ⏳ Pending Phase 2

---

## Next Steps (Prioritized)

### Immediate (Phase 2.1 - INP Parser)
1. Add normal direction parsing to `*BEAM SECTION`
2. Test with `simplebeam.inp` (normal = [1, 0, 0])

### Short-term (Phase 2.2 - Solve Integration)
3. Create `expand_and_solve_b32r()` function in ccx-cli
4. Handle BC/load mapping from beam to section nodes
5. Test displacement solution accuracy

### Medium-term (Phase 2.3 - Stress Recovery)
6. Implement C3D20R stress evaluation
7. Match CalculiX output format (50 integration points)
8. Validate against `simplebeam.dat.ref`

### Long-term (Phase 3 - Validation)
9. Run full beam example suite (204 files)
10. Document results in validation database
11. Create summary report

---

## Estimated Timeline

| Phase | Component | Estimate | Status |
|-------|-----------|----------|--------|
| 1 | C3D20/C3D20R Element | 6-8 hours | ✅ DONE |
| 1 | Beam Expansion Module | 4-6 hours | ✅ DONE |
| 2 | INP Parser Integration | 2-3 hours | ⏳ TODO |
| 2 | Solve Command Integration | 4-6 hours | ⏳ TODO |
| 2 | Stress Recovery | 3-4 hours | ⏳ TODO |
| 3 | Simple Validation | 2 hours | ⏳ TODO |
| 3 | Multi-Element Test | 1 hour | ⏳ TODO |
| 3 | Full Regression | 2-3 hours | ⏳ TODO |
| **TOTAL** | **Complete Implementation** | **24-33 hours** | **~40% Complete** |

**Phase 1 completed**: 10-14 hours (actual)
**Remaining effort**: 14-19 hours for Phases 2-3

---

## Dependencies

### External Crates
- `nalgebra`: Vector/matrix operations (already in use)
- No new dependencies required

### Internal Modules
- ✅ `crates/ccx-solver/src/mesh.rs` (Node, Element, ElementType)
- ✅ `crates/ccx-solver/src/materials.rs` (Material)
- ✅ `crates/ccx-solver/src/elements/beam.rs` (BeamSection, SectionShape)
- ⏳ `crates/ccx-io/src/inp/parser.rs` (INP parsing - needs update)
- ⏳ `crates/ccx-cli/src/main.rs` (solve command - needs major update)

---

## Configuration Recommendations

Per user request: **"try to avoid hardcoded values and keep the parameter in configuration files"**

### Proposed Configuration Structure

**File**: `crates/ccx-solver/config/beam_expansion.toml` (create)

```toml
[expansion]
# Starting node ID for generated section nodes (avoid conflicts)
next_node_id = 1_000_000

# Starting element ID for generated solid elements
next_element_id = 1_000_000

[integration]
# Integration points for C3D20R (2 = 2×2×2 = 8 points)
reduced_integration_order = 2

# Integration points for C3D20 (3 = 3×3×3 = 27 points)
full_integration_order = 3

[stress]
# Number of stress output points for beam elements
num_integration_points = 50

# Stress extrapolation method: "gauss_to_nodes" or "direct"
extrapolation_method = "gauss_to_nodes"

[validation]
# Tolerance for displacement validation (%)
displacement_tolerance_pct = 1.0

# Tolerance for stress validation (%)
stress_tolerance_pct = 5.0

# Tolerance for volume calculation (absolute)
volume_tolerance = 1e-10
```

**Implementation**:
- Use `serde` + `toml` crate for parsing
- Load config once at startup
- Pass config structs to functions instead of hardcoded values

---

## Files Created/Modified

### Created (2 files, 684 lines):
1. `crates/ccx-solver/src/elements/solid20.rs` (373 lines)
2. `crates/ccx-solver/src/elements/beam_expansion.rs` (311 lines)

### Modified (1 file):
1. `crates/ccx-solver/src/elements/mod.rs` (+2 lines exports)

### Total LOC Added**: ~686 lines of Rust code

---

## Known Limitations

### Current Scope
1. **Rectangular sections only**: Circular/custom not yet implemented
2. **Linear analysis only**: No geometric/material nonlinearity
3. **Static loads only**: Dynamic analysis not supported
4. **Single material**: No composite beams

### Future Enhancements
- Add circular section support
- Implement pipe sections
- Support user-defined sections
- Add temperature loading
- Support dynamic analysis

---

## References

1. **CalculiX Documentation**: CrunchiX User's Manual v2.23
   - Section 6.2.11: B32R beam elements
   - Section 6.9.4: Beam sections
   - Section 6.8: Element types

2. **FEA Theory**:
   - Cook et al., "Concepts and Applications of FEA", Ch. 7-8
   - Zienkiewicz & Taylor, "The Finite Element Method", Vol. 1

3. **Implementation References**:
   - CalculiX source: `e_c3d.f`, `results.f`, `beamintscheme.f`
   - Previous work: `BEAM_STRESS_INVESTIGATION.md`

---

## Contact & Collaboration

**Next Session Goals**:
1. Complete INP parser integration (2-3 hours)
2. Begin solve command integration (4-6 hours)
3. First test run with `simplebeam.inp`

**Questions for User**:
- Priority: Speed vs accuracy? (affect integration order choices)
- Output format: Full 50-point stress output or summary only?
- Validation criteria: What error thresholds are acceptable?

---

**Status Summary**: Phase 1 infrastructure complete and compiling. Ready to proceed to Phase 2 (solver integration).
