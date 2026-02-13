# Detailed Implementation Plan: B32R → C3D20R Expansion

## Overview
Complete the implementation to achieve 100% match with CalculiX reference output for beam elements expanded to C3D20R solid elements.

**Total Estimated Time:** 7-11 hours
**Goal:** Exact stress match with `validation/solver/simplebeam.dat.ref`

---

## PHASE 1: BC/Load Transfer (2-3 hours)

### Task 8.1: Design BC Transfer Strategy
**Time:** 30 minutes
**File:** New - `crates/ccx-solver/src/bc_transfer.rs`

**Design:**
```rust
pub struct BCTransfer {
    beam_node_mapping: HashMap<i32, [i32; 8]>,
}

impl BCTransfer {
    /// Transfer displacement BCs from beam node to 8 section nodes
    /// Strategy: If beam node is fixed in DOF i, fix all 8 section nodes in DOF i
    pub fn transfer_displacement_bcs(
        &self,
        original_bcs: &BoundaryConditions,
    ) -> BoundaryConditions;
    
    /// Transfer loads from beam node to 8 section nodes
    /// Strategy: Distribute load equally among 8 nodes (load/8 per node)
    pub fn transfer_loads(
        &self,
        original_loads: &[Load],
    ) -> Vec<Load>;
}
```

**Key Decisions:**
- Fixed BC: Apply to ALL 8 section nodes (preserves constraint)
- Point load: Distribute equally (∑F = F_total, statically equivalent)
- Moment loads: Not needed for simplebeam.inp (only has point load)

**Output:** Design document + empty module structure

---

### Task 8.2: Implement BC Transfer
**Time:** 1-1.5 hours
**Files:**
- `crates/ccx-solver/src/bc_transfer.rs` (new, ~150 lines)
- `crates/ccx-solver/src/lib.rs` (add module)

**Implementation Steps:**

1. **Create BCTransfer struct** (15 min)
   ```rust
   pub struct BCTransfer {
       beam_node_mapping: HashMap<i32, [i32; 8]>,
   }
   ```

2. **Implement displacement BC transfer** (30 min)
   ```rust
   pub fn transfer_displacement_bcs(&self, bcs: &BoundaryConditions) -> BoundaryConditions {
       let mut new_bcs = BoundaryConditions::new();
       
       for bc in bcs.displacement_bcs() {
           if let Some(section_nodes) = self.beam_node_mapping.get(&bc.node_id) {
               // Beam node → transfer to all 8 section nodes
               for &section_node_id in section_nodes {
                   new_bcs.add_displacement_bc(DisplacementBC {
                       node_id: section_node_id,
                       dof_start: bc.dof_start,
                       dof_end: bc.dof_end,
                       value: bc.value,
                   });
               }
           } else {
               // Non-beam node → copy as-is
               new_bcs.add_displacement_bc(bc.clone());
           }
       }
       new_bcs
   }
   ```

3. **Implement load transfer** (30 min)
   ```rust
   pub fn transfer_point_loads(&self, loads: &[PointLoad]) -> Vec<PointLoad> {
       let mut new_loads = Vec::new();
       
       for load in loads {
           if let Some(section_nodes) = self.beam_node_mapping.get(&load.node_id) {
               // Distribute load equally among 8 nodes
               let load_per_node = load.magnitude / 8.0;
               for &section_node_id in section_nodes {
                   new_loads.push(PointLoad {
                       node_id: section_node_id,
                       dof: load.dof,
                       magnitude: load_per_node,
                   });
               }
           } else {
               // Non-beam node → copy as-is
               new_loads.push(load.clone());
           }
       }
       new_loads
   }
   ```

4. **Add unit tests** (15 min)
   - Test BC transfer for fixed node
   - Test load distribution for point load
   - Test pass-through for non-beam nodes

**Success Criteria:**
- ✅ All tests pass
- ✅ Module compiles without errors
- ✅ BCs/loads sum to correct totals

---

### Task 8.3: Integrate BC Transfer into Pipeline
**Time:** 30-45 minutes
**File:** `crates/ccx-solver/src/analysis.rs`

**Changes to `expand_b32r_mesh()`:**

1. **Collect beam node mappings** (10 min)
   ```rust
   // After expansion loop
   let mut all_beam_mappings = HashMap::new();
   for (elem_id, element) in &mesh.elements {
       if element.element_type == ElementType::B32 {
           let result = expand_b32r(...)?;
           
           // Collect mappings
           for (beam_node_id, section_nodes) in result.beam_node_mapping {
               all_beam_mappings.insert(beam_node_id, section_nodes);
           }
           
           // Add nodes and elements...
       }
   }
   ```

2. **Return mapping from expand_b32r_mesh** (10 min)
   ```rust
   fn expand_b32r_mesh(
       mesh: &crate::Mesh,
       deck: &Deck,
   ) -> Result<(crate::Mesh, HashMap<i32, [i32; 8]>), String>
   ```

3. **Apply transfer in run()** (15 min)
   ```rust
   // In AnalysisPipeline::run()
   let (mesh, beam_mapping) = if use_expansion && Self::has_b32r_elements(&mesh) {
       Self::expand_b32r_mesh(&mesh, deck)?
   } else {
       (mesh, HashMap::new())
   };
   
   // Build BCs
   let mut bcs = crate::bc_builder::BCBuilder::build_from_deck(deck)?;
   
   // Transfer BCs if expansion was used
   if !beam_mapping.is_empty() {
       let transfer = BCTransfer::new(beam_mapping.clone());
       bcs = transfer.transfer_displacement_bcs(&bcs);
       // Note: Load transfer happens in bc_builder or here
   }
   ```

4. **Test with simplebeam.inp** (10 min)
   ```bash
   CCX_EXPAND_B32R=1 cargo run --release --bin ccx-cli -- solve tests/fixtures/solver/simplebeam.inp
   ```

**Success Criteria:**
- ✅ Solver runs without errors
- ✅ BCs applied to correct nodes (check debug output)
- ✅ Loads distributed correctly

---

## PHASE 2: Stress Recovery (2-3 hours)

### Task 10.1: Implement C3D20R Stress Recovery
**Time:** 1.5-2 hours
**File:** `crates/ccx-solver/src/elements/solid20.rs` (add methods)

**Implementation:**

1. **Add stress recovery method** (45 min)
   ```rust
   impl C3D20 {
       /// Compute stresses at integration points
       pub fn compute_stresses(
           &self,
           nodes: &[Node; 20],
           material: &Material,
           element_displacements: &[f64; 60],  // 20 nodes × 3 DOFs
       ) -> Result<Vec<StressState>, String> {
           let mut stresses = Vec::new();
           
           // Get integration points (8 for C3D20R)
           let int_points = if self.reduced_integration {
               Self::gauss_points_8()
           } else {
               Self::gauss_points_27()
           };
           
           let d_matrix = self.constitutive_matrix(material)?;
           
           for (point, _weight) in int_points {
               let (xi, eta, zeta) = point;
               
               // Compute B-matrix at this point
               let b = Self::b_matrix(nodes, xi, eta, zeta);
               
               // Compute strains: ε = B * u_e
               let strains = b * DVector::from_row_slice(element_displacements);
               
               // Compute stresses: σ = D * ε
               let stress_vector = d_matrix * strains;
               
               stresses.push(StressState {
                   sxx: stress_vector[0],
                   syy: stress_vector[1],
                   szz: stress_vector[2],
                   sxy: stress_vector[3],
                   sxz: stress_vector[4],
                   syz: stress_vector[5],
               });
           }
           
           Ok(stresses)
       }
   }
   ```

2. **Add StressState struct** (15 min)
   ```rust
   #[derive(Debug, Clone, Copy)]
   pub struct StressState {
       pub sxx: f64,
       pub syy: f64,
       pub szz: f64,
       pub sxy: f64,
       pub sxz: f64,
       pub syz: f64,
   }
   ```

3. **Add integration point getter** (15 min)
   ```rust
   impl C3D20 {
       pub fn num_integration_points(&self) -> usize {
           if self.reduced_integration { 8 } else { 27 }
       }
   }
   ```

4. **Test stress recovery** (30 min)
   - Unit test: Simple cube under tension
   - Verify σ = E·ε for uniaxial stress
   - Check symmetry of stress tensor

**Success Criteria:**
- ✅ Stress recovery compiles and runs
- ✅ Unit test passes (±0.1% error)
- ✅ Stresses are physically reasonable

---

### Task 10.2: Integrate Stress Recovery into Solve
**Time:** 30-45 minutes
**File:** `crates/ccx-cli/src/main.rs` (modify `write_dat_output`)

**Changes:**

1. **Detect C3D20 elements** (10 min)
   ```rust
   let has_c3d20 = mesh.elements.values()
       .any(|e| e.element_type == ElementType::C3D20);
   ```

2. **Compute C3D20 stresses** (20 min)
   ```rust
   if element.element_type == ElementType::C3D20 {
       // Get element nodes
       let elem_nodes: Vec<Node> = element.nodes.iter()
           .map(|&id| mesh.nodes.get(&id).unwrap().clone())
           .collect();
       
       // Extract element displacements
       let mut elem_disp = vec![0.0; 60];
       for (i, &node_id) in element.nodes.iter().enumerate() {
           for dof in 0..3 {
               let global_dof = (node_id - 1) as usize * max_dofs + dof;
               elem_disp[i * 3 + dof] = displacements[global_dof];
           }
       }
       
       // Compute stresses
       let c3d20 = C3D20::new_reduced(element.id, /* node array */);
       let stresses = c3d20.compute_stresses(&elem_nodes, &material, &elem_disp)?;
       
       all_stresses.push((element.id, stresses));
   }
   ```

3. **Test end-to-end** (15 min)
   ```bash
   CCX_EXPAND_B32R=1 cargo run --release --bin ccx-cli -- solve tests/fixtures/solver/simplebeam.inp
   ```

**Success Criteria:**
- ✅ Stresses computed without errors
- ✅ DAT file contains stress values
- ✅ Stress magnitudes in reasonable range

---

## PHASE 3: Integration Point Mapping (1-2 hours)

### Task 11.1: Understand Reference Integration Points
**Time:** 30 minutes
**Action:** Analyze reference output structure

**Analysis:**
```bash
# Count integration points in reference
grep "^         1  " validation/solver/simplebeam.dat.ref | wc -l
# Expected: 50 points

# Identify pattern
head -60 validation/solver/simplebeam.dat.ref
```

**Key Questions:**
- How are 50 points distributed? (10 along length × 5 through thickness?)
- What are exact coordinates of integration points?
- Do they match C3D20R Gauss points?

**Output:** Document describing point distribution

---

### Task 11.2: Generate Additional Evaluation Points
**Time:** 1-1.5 hours
**File:** `crates/ccx-solver/src/elements/solid20.rs`

**Strategy:** C3D20R has 8 integration points, but we need 50 for output

**Approach:**
```rust
impl C3D20 {
    /// Get evaluation points for stress output (matches CalculiX)
    /// Returns 50 points distributed through element volume
    pub fn output_evaluation_points() -> Vec<(f64, f64, f64)> {
        let mut points = Vec::new();
        
        // 10 stations along ξ (length direction)
        for i in 0..10 {
            let xi = -1.0 + 2.0 * (i as f64) / 9.0;
            
            // 5 points through thickness at each station
            // Pattern: center + 4 quadrants
            for (eta, zeta) in &[
                (0.0, 0.0),           // Center
                (0.577, 0.577),       // Quadrant 1
                (-0.577, 0.577),      // Quadrant 2
                (0.577, -0.577),      // Quadrant 3
                (-0.577, -0.577),     // Quadrant 4
            ] {
                points.push((xi, *eta, *zeta));
            }
        }
        
        points
    }
    
    /// Evaluate stress at arbitrary point (not just integration points)
    pub fn evaluate_stress_at(
        &self,
        nodes: &[Node; 20],
        material: &Material,
        element_displacements: &[f64; 60],
        xi: f64,
        eta: f64,
        zeta: f64,
    ) -> Result<StressState, String> {
        // Same as compute_stresses but for single point
        let b = Self::b_matrix(nodes, xi, eta, zeta);
        let d_matrix = self.constitutive_matrix(material)?;
        let strains = b * DVector::from_row_slice(element_displacements);
        let stress_vector = d_matrix * strains;
        
        Ok(StressState {
            sxx: stress_vector[0],
            syy: stress_vector[1],
            szz: stress_vector[2],
            sxy: stress_vector[3],
            sxz: stress_vector[4],
            syz: stress_vector[5],
        })
    }
}
```

**Success Criteria:**
- ✅ Generates exactly 50 evaluation points
- ✅ Points distributed logically through volume
- ✅ Can evaluate stress at any point

---

## PHASE 4: DAT Writer Update (1-2 hours)

### Task 12.1: Update DAT Output Format
**Time:** 1-1.5 hours
**File:** `crates/ccx-cli/src/main.rs` (update `write_dat_output`)

**Current Format (beam):**
```
 stresses (elem, integ.pnt.,sxx,syy,szz,sxy,sxz,syz) for set EALL and time  1.0000000E0

         1   1    value1   value2   ...
```

**Target Format (C3D20R):** Same format, different values

**Implementation:**

1. **Replace beam stress computation** (30 min)
   ```rust
   // OLD: Beam stress evaluation
   let evaluator = BeamStressEvaluator::new(...);
   let stresses = evaluator.compute_all_stresses(...)?;
   
   // NEW: C3D20R stress evaluation
   let c3d20 = C3D20::new_reduced(...);
   let eval_points = C3D20::output_evaluation_points();
   let mut stresses = Vec::new();
   for (xi, eta, zeta) in eval_points {
       let stress = c3d20.evaluate_stress_at(nodes, material, elem_disp, xi, eta, zeta)?;
       stresses.push(stress);
   }
   ```

2. **Update IntegrationPointStress struct** (15 min)
   ```rust
   struct IntegrationPointStress {
       element_id: i32,
       point_id: usize,
       sxx: f64,
       syy: f64,
       szz: f64,
       sxy: f64,
       sxz: f64,
       syz: f64,
   }
   ```

3. **Write formatted output** (30 min)
   ```rust
   for (elem_id, stresses) in all_stresses {
       for (i, stress) in stresses.iter().enumerate() {
           writeln!(
               file,
               "         {:2}  {:2} {:13.6E} {:13.6E} {:13.6E} {:13.6E} {:13.6E} {:13.6E}",
               elem_id, i + 1,
               stress.sxx, stress.syy, stress.szz,
               stress.sxy, stress.sxz, stress.syz
           )?;
       }
   }
   ```

4. **Verify formatting** (15 min)
   - Check column alignment
   - Check scientific notation format
   - Compare with reference visually

**Success Criteria:**
- ✅ DAT file format matches reference exactly
- ✅ 50 integration points per element
- ✅ Column alignment correct

---

## PHASE 5: Testing & Validation (1 hour)

### Task 13.1: Run Full Test Suite
**Time:** 30 minutes

**Tests:**

1. **Compile and build** (5 min)
   ```bash
   cargo build --release --package ccx-cli
   ```

2. **Run solver with expansion** (5 min)
   ```bash
   CCX_EXPAND_B32R=1 /mnt/mobile/tmp/rcompare-target/release/ccx-cli solve tests/fixtures/solver/simplebeam.inp
   ```

3. **Compare with reference** (10 min)
   ```bash
   python3 stress_comparison_analysis.py
   ```

4. **Check key metrics** (10 min)
   - Volume: Should be exact (6.250000E-01)
   - Number of points: Should be 50
   - Stress values: Should match within 1%

---

### Task 13.2: Debug and Fix Issues
**Time:** 30 minutes (buffer)

**Common Issues:**
1. **Sign errors** → Check coordinate system
2. **Magnitude errors** → Check constitutive matrix
3. **Wrong point count** → Check evaluation point generation
4. **Format errors** → Check scientific notation

**Debug Strategy:**
```bash
# Enable debug output
CCX_EXPAND_B32R=1 CCX_DEBUG=1 ./ccx-cli solve tests/fixtures/solver/simplebeam.inp

# Compare first 10 points manually
head -20 tests/fixtures/solver/simplebeam.dat
head -20 validation/solver/simplebeam.dat.ref

# Check differences
diff tests/fixtures/solver/simplebeam.dat validation/solver/simplebeam.dat.ref
```

---

## Success Criteria - Final Validation

### Must Pass:
- ✅ Volume exactly 6.250000E-01
- ✅ Exactly 50 integration points
- ✅ All stress components within 5% of reference
- ✅ No compilation errors or warnings (non-cosmetic)
- ✅ python3 stress_comparison_analysis.py shows >95% accuracy

### Stretch Goals:
- ✅ All stress components within 1% of reference
- ✅ Bit-exact match with reference (unlikely but possible)

---

## Timeline Summary

| Phase | Tasks | Time | Dependencies |
|-------|-------|------|--------------|
| 1. BC/Load Transfer | 8.1-8.3 | 2-3h | None |
| 2. Stress Recovery | 10.1-10.2 | 2-3h | Phase 1 |
| 3. Integration Points | 11.1-11.2 | 1-2h | Phase 2 |
| 4. DAT Writer | 12.1 | 1-2h | Phase 3 |
| 5. Testing | 13.1-13.2 | 1h | All above |
| **Total** | | **7-11h** | |

---

## Risk Mitigation

### High Risk Items:
1. **BC/Load transfer** - Wrong distribution breaks solver
   - Mitigation: Test with simple 1-element case first
   
2. **Integration point count mismatch** - Can't match reference
   - Mitigation: Analyze reference structure before implementing

3. **Coordinate system confusion** - Sign errors in stresses
   - Mitigation: Add visualization/debug output for verification

### Medium Risk Items:
1. Stress recovery numerical accuracy
2. DAT format compatibility
3. Edge cases in beam_node_mapping

---

## Rollback Plan

If any phase takes >150% estimated time or encounters blocking issues:

1. **Stop and assess** - Document the blocker
2. **Revert to enhanced beam theory** - Already working (60-77% accuracy)
3. **File issue** - Document what needs fixing for future work
4. **Deliver partial solution** - E.g., expansion without stress recovery

---

## File Change Summary

### New Files:
- `crates/ccx-solver/src/bc_transfer.rs` (~150 lines)
- `C3D20R_IMPLEMENTATION_PLAN.md` (this file)

### Modified Files:
- `crates/ccx-solver/src/elements/solid20.rs` (+200 lines)
- `crates/ccx-solver/src/analysis.rs` (+50 lines)
- `crates/ccx-cli/src/main.rs` (+100 lines)
- `crates/ccx-solver/src/lib.rs` (+1 line)

### Total Code Impact:
- **~500 lines new code**
- **~150 lines modified code**
- **~100 lines tests**

---

## Next Steps

**Recommended approach:**

1. **Read and approve this plan** (5 min)
2. **Start Phase 1** - BC/Load transfer (critical path)
3. **After each phase:** Test and verify before proceeding
4. **Track actual vs estimated time** - Adjust plan if needed
5. **Final validation** - Compare with reference

**Ready to proceed?** Confirm and I'll start with Phase 1, Task 8.1.
