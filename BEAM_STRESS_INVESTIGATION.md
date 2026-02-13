# Beam Stress Investigation Summary

## Objective
Match stress output from `tests/fixtures/solver/simplebeam.inp` to reference `validation/solver/simplebeam.dat.ref`

## Test Case
- **Element**: 1× B32R (3-node quadratic beam)
- **Geometry**: Cantilever from z=0 (free) to z=10 (fixed)
- **Load**: 1.0 N in X-direction at free end
- **Section**: 0.25×0.25 rectangular
- **Material**: E=1e7, ν=0.3

## Progress Summary

### ✅ Accomplished

1. **Complete Infrastructure** (100%)
   - Created `beam_stress.rs` module with stress evaluator
   - Implemented section force recovery
   - Added stress-to-global coordinate transformations
   - Integrated with DAT file output
   - Parse beam sections and loads from INP files

2. **Key Bug Fixes**
   - Fixed beam length calculation (was using nodes[0]→nodes[1] instead of nodes[0]→nodes[2])
   - Corrected coordinate system (stresses now correctly show szz as dominant component)
   - Implemented direct load usage (bypassing displacement errors)

3. **Validation Status**
   - Volume: **EXACT MATCH** (6.250000E-01)
   - Stress direction: **CORRECT** (szz dominant in global coords)
   - Stress magnitude: **~7x off** (3407 vs 468-1748 reference)

### ❌ Root Causes Identified

#### 1. **Incorrect B32 Bending Stiffness** (CRITICAL)
**Location**: `crates/ccx-solver/src/elements/beam3.rs:216-233`

**Problem**: Uses first derivatives for bending stiffness:
```rust
let k_bend_z = e * iy * dn_dx[i] * dn_dx[j] * jac * weight;
```

**Why Wrong**:
- Euler-Bernoulli bending requires **second derivatives** (curvature: κ = d²w/dx²)
- Current formulation treats bending like axial deformation
- Results in stiffness **~86x too soft** → displacements 86x too large

**Evidence**:
- Expected deflection: δ = PL³/(3EI) = 0.1024
- Actual: 8.8344 (86x too large)
- Ratio matches back-calculated load error (86.27)

#### 2. **Simplified Stress Recovery** (MAJOR)
**Location**: `crates/ccx-solver/src/elements/beam_stress.rs`

**Current Approach**: Simple beam theory
```rust
σ_bending = -Mz * y / Iz  // Only bending stress
τ_shear = 1.5 * V / A      // Average shear
```

**CalculiX B32R Uses**: Sophisticated 3D stress recovery
- Full 6-component stress tensor (sxx, syy, szz, sxy, sxz, syz all non-zero)
- Poisson-induced transverse stresses
- Complex through-thickness variation
- Stress extrapolation from integration points

**Comparison**:
```
My Output (IP 1-2):
  szz: ±3407, sxz: 24, all others: 0

Reference (IP 1-2):
  sxx: -135 to 505, syy: -45 to 168, szz: 468 to 1748
  sxy: -45 to 168, sxz: -16, syz: -0.22
```

## What Would Be Needed for Exact Matching

### Critical Path (Must Fix)

1. **Rewrite B32 Stiffness Matrix**
   - **Effort**: 2-3 days
   - **Approach Options**:
     - A. Hermite cubic interpolation (requires C1 continuity)
     - B. Proper Timoshenko formulation with shear locking prevention
     - C. Port CalculiX's `e_c3d.f` beam formulation
   - **Reference**: Cook et al. "Concepts and Applications of FEA" Ch. 7

2. **Implement CalculiX B32R Stress Recovery**
   - **Effort**: 3-5 days
   - **Source**: CalculiX Fortran files:
     - `results.f` - Main stress recovery
     - `beamintscheme.f` - Integration point scheme
     - `e_c3d.f` - Element formulation
   - **Complexity**: Requires understanding CalculiX's:
     - Reduced integration scheme (B32**R**)
     - Stress extrapolation algorithms
     - Through-thickness evaluation points

### Additional Improvements

3. **3D Stress State**
   - Add Poisson-induced transverse stresses: σ_transverse = -ν * σ_axial
   - Implement proper shear stress distribution (not just average)

4. **Integration Point Scheme**
   - Match CalculiX's exact 50-point scheme
   - Current: generic Gauss + cross-section points
   - Need: CalculiX's specific B32R pattern

## Current Limitations

### Displacement Solution
- **Error**: 86x too large (8.8344 vs 0.1024 expected)
- **Impact**: Cannot use displacements for stress recovery
- **Workaround**: Using applied load directly (partially successful)

### Stress Values
- **Error**: ~2-7x off depending on location
- **Pattern**: Correct (bending distribution) but wrong magnitude
- **Missing**: Full 3D stress tensor (only have 2 of 6 components)

## Recommendations

### Option A: Quick Approximation (1 day)
- Scale stresses by empirical factors to match reference
- Document limitations
- **Pro**: Fast, gets "close enough" for visualization
- **Con**: Not physics-based, won't generalize

### Option B: Proper Implementation (1-2 weeks)
- Fix B32 stiffness matrix completely
- Implement CalculiX-compatible stress recovery
- Validate against full test suite
- **Pro**: Correct, reusable, maintainable
- **Con**: Significant time investment

### Option C: Hybrid Approach (3-5 days)
- Keep current infrastructure
- Add correction factors based on beam theory
- Implement partial 3D stress state
- **Pro**: Balanced effort vs accuracy
- **Con**: Still approximation, limited accuracy

## Files Modified

- `crates/ccx-solver/src/elements/beam_stress.rs` - Stress computation (new, 482 lines)
- `crates/ccx-solver/src/elements/beam3.rs` - Fixed length calculation
- `crates/ccx-cli/src/main.rs` - Integrated stress output (~200 lines added)
- `crates/ccx-solver/src/lib.rs` - Added exports

## Test Results

### Current Output vs Reference
```
Volume: 6.250000E-01 ✓ EXACT MATCH

Stresses (first 3 IPs):
Mine:     szz=±3407, sxz=24, others=0
Reference: szz=468-1748, sxz=-16, plus sxx, syy, sxy, syz

Factor: ~2-7x depending on position
```

## Conclusion

The **infrastructure is complete and correct**. The remaining discrepancy is due to:
1. Incorrect B32 bending stiffness (needs complete rewrite)
2. Simplified stress recovery vs CalculiX's sophisticated algorithm

Fixing properly requires porting/reimplementing CalculiX's decades-old Fortran beam formulation, which is a substantial engineering effort beyond the original scope.

---
**Date**: 2026-02-11
**Status**: Infrastructure complete, awaiting decision on accuracy requirements
