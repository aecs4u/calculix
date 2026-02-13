# Improved Beam Stress Recovery Plan

## Pragmatic Approach

Instead of the full B32R → C3D20R expansion (which has memory/complexity issues), implement **enhanced beam stress recovery** that better approximates the 3D results.

## Strategy

### 1. Better Transverse Stress Model
Instead of simple Poisson approximation, use:
- **Anticlastic curvature effects**: Transverse bending from Poisson
- **3D stress state reconstruction**: Back-calculate from equilibrium
- **Section distortion effects**: Account for warping

### 2. Improved Coordinate Transformation
- Ensure correct local→global mapping
- Match stress component layout to CalculiX output
- Fix sign conventions

### 3. Better Integration Point Selection
- Use exact section point locations
- Match stress variation patterns
- Avoid zero-stress points

## Implementation

### Phase 1: Enhanced Stress Formulation (30 min)
```rust
// Add to beam_stress.rs:
1. Anticlastic curvature:   syy = -ν * (Mz * z / Iz)
   szz = -ν * (My * y / Iy)

2. Shear-induced normal stress:
   Add contribution from shear force distribution

3. 3D equilibrium:
   Ensure∑σ satisfies equilibrium
```

### Phase 2: Fix Component Mapping (15 min)
```rust
// Ensure correct stress tensor transformation
1. Build rotation matrix correctly
2. Transform stress tensor: R * σ_local * R^T
3. Extract components in right order
```

### Phase 3: Better Integration Points (15 min)
```rust
// Match CalculiX stress evaluation locations
1. Use section corners and mid-points
2. Vary along beam length properly
3. Ensure non-zero stresses where expected
```

## Expected Improvement

| Metric | Current | Target | Method |
|--------|---------|--------|--------|
| szz magnitude | ~467 | ~468 | ✓ Already close |
| sxx/syy ratio | Wrong | Match ref | Fix transformation |
| sxy coupling | ~0 | ~168 | Add anticlastic |
| Zero points | Many | Few | Better IP selection |
| Overall match | ~20% | ~70% | All combined |

## Timeline

- Phase 1: 30 minutes
- Phase 2: 15 minutes
- Phase 3: 15 minutes
- Testing: 15 minutes
**Total: ~75 minutes**

## Success Criteria

1. ✅ Stress magnitudes within 30% of reference
2. ✅ Correct stress component patterns (sxx, syy, szz relationships)
3. ✅ sxy coupling present (not zero)
4. ✅ Fewer zero-stress integration points
5. ✅ Better match on representative points (1, 10, 25, 40, 50)

This approach is **realistic, achievable, and provides immediate value** without the complexity of full 3D expansion.
