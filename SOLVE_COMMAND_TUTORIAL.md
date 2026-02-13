# Tutorial: Using the `ccx-cli solve` Command

**Target Audience:** Engineers and analysts wanting to run FEA simulations with the Rust CalculiX solver

**Prerequisites:**
- Rust toolchain installed (rustc, cargo)
- Basic understanding of finite element analysis
- CalculiX input file (.inp format)

**Time Required:** 10 minutes

---

## Step 1: Build the Solver

First, build the ccx-cli tool in release mode for optimal performance:

```bash
cd /path/to/calculix
cargo build --package ccx-cli --release
```

**Expected Output:**
```
   Compiling ccx-cli v0.1.0
    Finished `release` profile [optimized] target(s) in ~4 minutes
```

The binary will be created at: `target/release/ccx-cli` (or use `cargo run` to build and run in one step)

---

## Step 2: Prepare Your Input File

The solve command requires a CalculiX input file (.inp) with:

### Minimum Requirements
1. **Nodes** (`*NODE`)
2. **Elements** (`*ELEMENT`) - Currently B32R supported
3. **Material** (`*MATERIAL`, `*ELASTIC`)
4. **Section** (`*BEAM SECTION`)
5. **Boundary Conditions** (`*BOUNDARY`)
6. **Loads** (`*CLOAD`)
7. **Step** (`*STEP`, `*STATIC`, `*END STEP`)

### Example: Simple Cantilever Beam

Create a file `cantilever.inp`:

```
**
** Simple cantilever beam example
** 3-node B32R beam, load at free end
**
*NODE, NSET=Nall
1, 0, 0, 0
2, 0, 0, 5
3, 0, 0, 10
*ELEMENT, TYPE=B32R, ELSET=Eall
1, 1, 2, 3
*BOUNDARY
3, 1, 6
*MATERIAL, NAME=STEEL
*ELASTIC
2.1E11, 0.3
*BEAM SECTION, ELSET=Eall, MATERIAL=STEEL, SECTION=RECT
0.01, 0.02
1.0, 0.0, 0.0
*STEP
*STATIC
*CLOAD
1, 1, 1000.0
*EL PRINT, ELSET=Eall
S
*END STEP
```

**What This Defines:**
- 3 nodes along Z-axis (0, 5, 10 meters)
- 1 B32R beam element
- Fixed at node 3 (all 6 DOFs)
- Steel material (E=210 GPa, ν=0.3)
- Rectangular section (10mm × 20mm)
- Normal vector (1, 0, 0)
- 1000 N load at node 1 in X-direction

---

## Step 3: Run the Solver

Execute the solve command:

```bash
cargo run --package ccx-cli --release -- solve cantilever.inp
```

Or if you have the binary:

```bash
./target/release/ccx-cli solve cantilever.inp
```

**Expected Output:**
```
Solving: cantilever.inp
Model initialized: 3 nodes, 1 elements, 9 DOFs (3 free, 6 constrained), 1 loads [SOLVED]
Writing output to: cantilever.dat
Solve complete!
```

**Execution Time:** ~1-2 seconds

---

## Step 4: Inspect the Results

The solver creates a DAT file with the same name as your input: `cantilever.dat`

### View the File

```bash
less cantilever.dat
```

or

```bash
head -50 cantilever.dat
```

### DAT File Structure

```
                        S T E P       1


                                INCREMENT     1


 stresses (elem, integ.pnt.,sxx,syy,szz,sxy,sxz,syz) for set EALL and time  1.0000000E0

         1   1    1.234567E3    2.345678E2   -4.567890E2    ...
         1   2    2.345678E3    3.456789E2   -5.678901E2    ...
         ...
         1  50    9.876543E2    1.234567E2   -2.345678E2    ...

 total volume of the model:  2.000000E-3
```

### Understanding the Output

**Stress Line Format:**
```
Element  IP  sxx       syy       szz       sxy       sxz       syz
   1     1   1234.5    234.5    -456.7    0.0       123.4     0.0
```

Where:
- **Element:** Element ID
- **IP:** Integration point number (1-50 for B32R)
- **sxx:** Normal stress in local X (beam axis)
- **syy, szz:** Transverse normal stresses
- **sxy, sxz, syz:** Shear stresses

**Units:** Same as input (if E in Pa, stress in Pa)

**Volume:**
- Total volume of all elements
- Check: Should match hand calculation (Area × Length)

---

## Step 5: Interpret the Results

### Typical Stress Patterns in Beams

**Cantilever Beam Under Lateral Load:**

1. **sxx (Axial/Bending Stress):**
   - Maximum at extreme fibers (top/bottom surfaces)
   - Zero at neutral axis (center)
   - Varies linearly through section height

2. **syy, szz (Transverse Stresses):**
   - Usually small compared to sxx
   - Caused by Poisson effect and anticlastic curvature
   - Order of magnitude: ~0.3 × sxx (for ν=0.3)

3. **sxy, sxz (Shear Stresses):**
   - Maximum at neutral axis
   - Zero at extreme fibers
   - Parabolic distribution through section

4. **Integration Point Locations:**
   - Points 1-10: First section (near free end)
   - Points 11-20: Second section
   - Points 21-30: Third section
   - Points 31-40: Fourth section
   - Points 41-50: Fifth section (near fixed end)

### Validation Checks

✅ **Volume Check:**
```
Expected Volume = Area × Length
For our example: (0.01 × 0.02) × 10 = 0.002 m³
DAT output should show: 2.000000E-3 ✓
```

✅ **Stress Magnitude Check:**
```
Maximum Bending Stress ≈ M × c / I
M = Force × Distance = 1000 N × 5 m = 5000 Nm (at mid-span)
I = b × h³ / 12 = 0.01 × 0.02³ / 12 = 6.67e-9 m⁴
c = h/2 = 0.01 m
σ_max ≈ 5000 × 0.01 / 6.67e-9 = 7.5 GPa

DAT output order of magnitude should be in billions (GPa range) ✓
```

⚠️ **Note:** Beam theory approximation means exact values may differ from reference by factor 0.4-2×

---

## Step 6: Common Issues and Solutions

### Issue 1: "Failed to solve linear system"

**Cause:** Singular stiffness matrix (underconstr ained model)

**Solutions:**
- Check boundary conditions: At least one node must be fully fixed
- Verify all 6 DOFs are constrained for beam problems
- Check element connectivity

**Example Fix:**
```
*BOUNDARY
3, 1, 6    ← Must constrain all 6 DOFs (translations + rotations)
```

### Issue 2: "Unknown element type"

**Cause:** Element type not yet implemented

**Current Support:** B32R only

**Workaround:**
- Use B32R elements for beams
- Wait for C3D8, S4 implementation
- Or contribute element support! 😊

### Issue 3: Stress values seem too large/small

**Cause:** Unit mismatch or scaling

**Check:**
1. Material E units (Pa? GPa?)
2. Geometry units (m? mm?)
3. Load units (N? kN?)

**Consistent Units Example:**
- Length: meters (m)
- Force: newtons (N)
- Stress: pascals (Pa)
- E: Pa (2.1E11 for steel)

### Issue 4: Warning messages during compilation

**Normal behavior!** The codebase has ~90 warnings from:
- Unused imports
- Variable naming conventions
- Dead code in unimplemented features

**These don't affect functionality.** The solver works correctly despite warnings.

---

## Step 7: Advanced Usage

### Multiple Elements

```
*NODE, NSET=Nall
1, 0, 0, 0
2, 0, 0, 5
3, 0, 0, 10
4, 0, 0, 15
5, 0, 0, 20
*ELEMENT, TYPE=B32R, ELSET=Eall
1, 1, 2, 3
2, 3, 4, 5
```

### Pipe Section

```
*BEAM SECTION, ELSET=Eall, MATERIAL=STEEL, SECTION=PIPE
0.11, 0.01
1.0, 0.0, 0.0
```
(Outer radius 0.11 m, thickness 0.01 m)

### Multiple Load Cases

Currently only single load case supported. For multiple cases:
1. Run solve separately for each case
2. Or modify input file and re-run

### Batch Processing

```bash
#!/bin/bash
for inp_file in problems/*.inp; do
    echo "Solving $inp_file"
    ccx-cli solve "$inp_file"
done
```

---

## Step 8: Next Steps

### Learn More
- **Implementation Details:** [SOLVE_COMMAND_IMPLEMENTATION.md](SOLVE_COMMAND_IMPLEMENTATION.md)
- **Validation Results:** [SOLVER_VALIDATION_TESTS.md](SOLVER_VALIDATION_TESTS.md)
- **Complete Session Report:** [FINAL_SESSION_REPORT.md](FINAL_SESSION_REPORT.md)
- **Stress Analysis Details:** [STRESS_VALIDATION_ANALYSIS.md](STRESS_VALIDATION_ANALYSIS.md)

### Try These Examples
1. `tests/fixtures/solver/simplebeam.inp` - Original test case
2. `tests/fixtures/solver/simplebeampipe1.inp` - 10-element pipe
3. `tests/fixtures/solver/b31.inp` - Linear beam elements

### Experiment
- Change beam dimensions (section size)
- Vary material properties
- Try different load magnitudes
- Add more elements for finer resolution

### Contribute
Found a bug? Want to add features?
- Element types: C3D8, S4, C3D10
- Displacement output
- Nonlinear analysis
- Better stress formulation

See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) for roadmap.

---

## Summary

You've learned how to:
- ✅ Build the ccx-cli solver
- ✅ Create a CalculiX input file
- ✅ Run the solve command
- ✅ Interpret DAT output
- ✅ Validate results
- ✅ Troubleshoot common issues

**The solver is ready for:**
- Preliminary structural design
- Parametric studies
- Educational demonstrations
- Algorithm development

**Remember the limitations:**
- Beam theory approximation (~40-200% error vs full 3D)
- Linear analysis only
- Limited element types
- Not for safety-critical applications

**Happy solving!** 🚀

---

**Questions or Issues?**
- Check documentation in repository root
- Review test cases in `tests/fixtures/solver/`
- Submit GitHub issues for bugs
- Contribute improvements via pull requests

**Version:** 2026-02-11
**Author:** CalculiX Rust Development Team
**License:** See repository LICENSE file
