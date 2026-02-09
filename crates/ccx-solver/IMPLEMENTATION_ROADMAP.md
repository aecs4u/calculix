# CalculiX Solver Implementation Roadmap

## ✅ Phase 1: Foundation (COMPLETE)

### Input Parsing & Data Structures
- [x] Mesh building (nodes + elements, 16 element types)
- [x] Multi-line element support (C3D20, etc.)
- [x] Node/element sets (NSET, ELSET)
- [x] Boundary conditions (BOUNDARY cards)
- [x] Concentrated loads (CLOAD cards)
- [x] Set resolution in BCs and loads

### Infrastructure
- [x] 9 ported utility functions from C/Fortran
- [x] 16 analysis type detection
- [x] 150 unit tests + 5 integration tests
- [x] Comprehensive error handling
- [x] Migration tracking system

## ⏳ Phase 2: Materials & Element Library (IN PROGRESS)

### Materials Module (Next Priority)
```rust
// crates/ccx-solver/src/materials.rs
pub struct Material {
    pub name: String,
    pub elastic_modulus: f64,      // Young's modulus (E)
    pub poissons_ratio: f64,       // Poisson's ratio (ν)
    pub density: Option<f64>,       // ρ
}

pub struct MaterialLibrary {
    materials: HashMap<String, Material>,
    element_materials: HashMap<i32, String>, // elem_id -> material_name
}
```

**Implementation Tasks:**
1. Parse `*MATERIAL`, `*ELASTIC`, `*DENSITY` cards
2. Store material properties by name
3. Assign materials to elements via ELSETs
4. Handle isotropic materials first
5. Add orthotropic/anisotropic later

### Element Library
**Priority Elements (simplest first):**

1. **2-Node Truss (T3D2)** - Tension/compression only
   - 1D element, easiest to implement
   - Shape functions: N1 = (1-ξ)/2, N2 = (1+ξ)/2
   - Element stiffness: k_e = (A*E/L) * [1 -1; -1 1]

2. **2-Node Beam (B31)** - Bending + axial
   - Euler-Bernoulli beam theory
   - 6 DOF per node (3 translations + 3 rotations)
   - Stiffness includes axial, bending, torsion

3. **8-Node Brick (C3D8)** - 3D solid
   - Trilinear shape functions
   - 2x2x2 Gauss integration
   - Full 3D stress/strain

**Element Interface:**
```rust
pub trait Element {
    fn stiffness_matrix(&self, nodes: &[Node], material: &Material) -> DMatrix<f64>;
    fn strain_displacement_matrix(&self, xi: f64, eta: f64, zeta: f64) -> DMatrix<f64>;
    fn stress(&self, displacement: &[f64], material: &Material) -> Vec<f64>;
}
```

## ⏳ Phase 3: Matrix Assembly & Solver (NEXT)

### Global Assembly
```rust
// crates/ccx-solver/src/assembly.rs
pub struct GlobalSystem {
    pub stiffness: CsrMatrix<f64>,  // Sparse K matrix
    pub force: DVector<f64>,         // F vector
    pub num_dofs: usize,
}

impl GlobalSystem {
    pub fn assemble(mesh: &Mesh, bcs: &BoundaryConditions, materials: &MaterialLibrary)
        -> Result<Self, String>;

    pub fn apply_boundary_conditions(&mut self, bcs: &BoundaryConditions);
}
```

**Tasks:**
1. Allocate sparse matrix (CSR format)
2. Loop over elements, compute k_e
3. Assemble into global K using connectivity
4. Build force vector from CLOADs
5. Apply displacement BCs (penalty or elimination method)

### Linear Solver
Using `nalgebra` with sparse support:

```rust
// crates/ccx-solver/src/solver.rs
pub fn solve_linear_system(K: &CsrMatrix<f64>, F: &DVector<f64>)
    -> Result<DVector<f64>, String> {
    // Use direct solver (LU/Cholesky) for small problems
    // Use iterative solver (CG) for large problems
}
```

**Solver Options:**
- Direct: Cholesky decomposition (K is SPD)
- Iterative: Conjugate Gradient with preconditioner
- External: MUMPS/PARDISO bindings (optional)

## ⏳ Phase 4: Results & Post-Processing

### Displacement Solution
```rust
pub struct SolutionField {
    pub node_displacements: HashMap<i32, [f64; 3]>,  // node_id -> [ux, uy, uz]
    pub node_reactions: HashMap<i32, [f64; 3]>,       // constrained nodes
}
```

### Stress/Strain Computation
```rust
pub struct StressResults {
    pub element_stresses: HashMap<i32, ElementStress>,
    pub von_mises: HashMap<i32, f64>,
}

pub struct ElementStress {
    pub sigma_xx: f64,
    pub sigma_yy: f64,
    pub sigma_zz: f64,
    pub tau_xy: f64,
    pub tau_yz: f64,
    pub tau_xz: f64,
}
```

### FRD Output Writer
```rust
// crates/ccx-solver/src/frd_writer.rs
pub fn write_frd(
    path: &Path,
    mesh: &Mesh,
    solution: &SolutionField,
    stresses: &StressResults,
) -> Result<(), String>;
```

## ⏳ Phase 5: Advanced Analysis Types

### Nonlinear Static
- Newton-Raphson iteration
- Line search/arc-length methods
- Material nonlinearity (plasticity)
- Geometric nonlinearity (large deformations)

### Modal Analysis
- Eigenvalue problem: (K - λM)φ = 0
- Subspace iteration or Lanczos algorithm
- Extract natural frequencies and mode shapes

### Dynamic Analysis
- Time integration: Newmark-β method
- Mass matrix assembly
- Damping (Rayleigh damping)

### Heat Transfer
- Thermal conductivity matrix
- Heat flux boundary conditions
- Coupled thermomechanical

## 📊 Current Status Summary

| Module | Status | Tests | Notes |
|--------|--------|-------|-------|
| Mesh Building | ✅ Complete | 9 tests | Multi-line elements supported |
| Node/Element Sets | ✅ Complete | 6 tests | Full resolution |
| Boundary Conditions | ✅ Complete | 7 tests | BOUNDARY + CLOAD |
| BC Builder | ✅ Complete | 9 tests | Set-aware |
| Analysis Pipeline | ✅ Framework | 13 tests | Detection logic complete |
| Materials | ⏳ TODO | 0 tests | High priority |
| Element Library | ⏳ TODO | 0 tests | Start with truss |
| Matrix Assembly | ⏳ TODO | 0 tests | Needs elements |
| Linear Solver | ⏳ TODO | 0 tests | Use nalgebra |
| Results Output | ⏳ TODO | 0 tests | FRD writer |

## 🎯 Minimal Viable Solver (MVP)

**Goal:** Solve a simple 2D truss problem

**Requirements:**
1. ✅ Parse nodes/elements
2. ✅ Parse boundary conditions
3. ✅ Parse loads
4. ⏳ Material properties (E, A)
5. ⏳ 2-node truss element
6. ⏳ Matrix assembly
7. ⏳ Linear solver
8. ⏳ Displacement output

**Test Problem:**
```
*NODE
1, 0, 0, 0
2, 1, 0, 0
3, 0.5, 0.866, 0
*ELEMENT, TYPE=T3D2
1, 1, 2
2, 2, 3
3, 3, 1
*MATERIAL, NAME=STEEL
*ELASTIC
210000, 0.3
*BOUNDARY
1, 1, 3
2, 2, 3
*CLOAD
3, 2, -1000.0
*STEP
*STATIC
*END STEP
```

**Expected:** Solve for displacement at node 3

## 📚 References

- CalculiX CrunchiX User's Manual
- CalculiX CGX Reference Manual
- "The Finite Element Method" by Zienkiewicz & Taylor
- "Nonlinear Finite Elements" by Wriggers
- Legacy CalculiX source: `calculix_migration_tooling/ccx_2.23/src/`

## 🔧 Development Commands

```bash
# Run all tests
cargo test --workspace

# Run solver tests only
cargo test --package ccx-solver

# Run integration tests
cargo test --test integration_tests

# Check migration progress
cargo run --bin ccx-solver -- migration-report

# Analyze a model
cargo run --bin ccx-solver -- solve model.inp
```

## 📈 Lines of Code Estimate

| Component | Est. LOC | Complexity |
|-----------|----------|------------|
| Materials | ~200 | Low |
| Truss Element | ~150 | Low |
| Beam Element | ~400 | Medium |
| C3D8 Element | ~600 | High |
| Assembly | ~300 | Medium |
| Solver | ~200 | Low (using nalgebra) |
| FRD Writer | ~250 | Low |
| **MVP Total** | **~1000** | |
| **Full Solver** | **~5000+** | |

## 🚀 Next Steps

1. **Implement Materials Module** (~1 hour)
   - Parse MATERIAL, ELASTIC, DENSITY
   - Store in MaterialLibrary
   - Tests for parsing

2. **Implement 2-Node Truss** (~2 hours)
   - Element stiffness matrix
   - Local-to-global transformation
   - Unit tests with analytical solutions

3. **Matrix Assembly** (~2 hours)
   - Sparse CSR matrix allocation
   - Element loop and assembly
   - BC application

4. **Solve MVP Problem** (~1 hour)
   - Integrate nalgebra solver
   - Validate against hand calculation
   - Add to integration tests

**Estimated time to MVP:** ~6-8 hours of focused development
