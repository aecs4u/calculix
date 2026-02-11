# C3D8 Element Implementation Plan

**Target**: 8-node brick solid element for 3D continuum mechanics
**Impact**: +99 tests, coverage boost from 1.4% → 17.2%
**Complexity**: Medium (3-4 days estimated)
**Priority**: HIGH - Highest ROI for test coverage

---

## Element Overview

### C3D8 Specification
- **Type**: 8-node hexahedral (brick) element
- **Geometry**: Trilinear isoparametric element
- **DOFs**: 3 per node (ux, uy, uz) - translations only
- **Nodes**: 8 corner nodes (no mid-side nodes)
- **Integration**: 2×2×2 Gauss quadrature (8 integration points)

### Node Numbering (CalculiX Convention)
```
       7----------8
      /|         /|
     / |        / |
    3----------4  |
    |  5-------|--6
    | /        | /
    |/         |/
    1----------2

Node order: 1,2,3,4 (bottom face), 5,6,7,8 (top face)
Local coords: ξ,η,ζ ∈ [-1,1]³
```

---

## Implementation Phases

### Phase 1: Element Structure & Shape Functions (Day 1)

#### Files to Create
**`crates/ccx-solver/src/elements/solid.rs`** (300-400 lines)

#### Core Components

1. **Element Struct**
```rust
pub struct C3D8 {
    id: i32,
    nodes: [i32; 8],  // 8 corner nodes
}

impl C3D8 {
    pub fn new(id: i32, nodes: [i32; 8]) -> Self {
        Self { id, nodes }
    }
}
```

2. **Shape Functions** (Trilinear)
```rust
fn shape_functions(xi: f64, eta: f64, zeta: f64) -> [f64; 8] {
    // N_i = (1 + ξξ_i)(1 + ηη_i)(1 + ζζ_i) / 8
    [
        (1.0 - xi) * (1.0 - eta) * (1.0 - zeta) / 8.0,  // N1
        (1.0 + xi) * (1.0 - eta) * (1.0 - zeta) / 8.0,  // N2
        (1.0 + xi) * (1.0 + eta) * (1.0 - zeta) / 8.0,  // N3
        (1.0 - xi) * (1.0 + eta) * (1.0 - zeta) / 8.0,  // N4
        (1.0 - xi) * (1.0 - eta) * (1.0 + zeta) / 8.0,  // N5
        (1.0 + xi) * (1.0 - eta) * (1.0 + zeta) / 8.0,  // N6
        (1.0 + xi) * (1.0 + eta) * (1.0 + zeta) / 8.0,  // N7
        (1.0 - xi) * (1.0 + eta) * (1.0 + zeta) / 8.0,  // N8
    ]
}
```

3. **Shape Function Derivatives** (dN/dξ, dN/dη, dN/dζ)
```rust
fn shape_derivatives(xi: f64, eta: f64, zeta: f64) -> [[f64; 8]; 3] {
    // Returns [dN/dξ, dN/dη, dN/dζ] for all 8 nodes
    // dN_i/dξ = ξ_i(1 + ηη_i)(1 + ζζ_i) / 8, etc.
}
```

4. **Jacobian Matrix** (3×3)
```rust
fn jacobian(&self, nodes: &[Node], xi: f64, eta: f64, zeta: f64)
    -> Result<Matrix3<f64>, String> {
    // J = [dx/dξ  dy/dξ  dz/dξ]
    //     [dx/dη  dy/dη  dz/dη]
    //     [dx/dζ  dy/dζ  dz/dζ]
    //
    // dx/dξ = Σ (dN_i/dξ) * x_i
}
```

5. **Strain-Displacement Matrix (B-matrix)**
```rust
fn strain_displacement_matrix(&self, nodes: &[Node], xi: f64, eta: f64, zeta: f64)
    -> Result<SMatrix<f64, 6, 24>, String> {
    // B matrix: 6 strains × 24 DOFs (8 nodes × 3 DOFs/node)
    // Relates nodal displacements to strains: {ε} = [B]{u}
    //
    // B = [dN/dx  0      0    ]  for all 8 nodes
    //     [0      dN/dy  0    ]
    //     [0      0      dN/dz]
    //     [dN/dy  dN/dx  0    ]
    //     [0      dN/dz  dN/dy]
    //     [dN/dz  0      dN/dx]
    //
    // where dN/dx = J⁻¹ * dN/dξ
}
```

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn shape_functions_partition_of_unity() {
        // Sum of N_i should equal 1.0 at any point
        for (xi, eta, zeta) in test_points {
            let N = shape_functions(xi, eta, zeta);
            assert!((N.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn shape_functions_at_nodes() {
        // N_i = 1 at node i, 0 at others
    }

    #[test]
    fn jacobian_for_unit_cube() {
        // For unit cube, J should be 0.5 * identity
    }
}
```

---

### Phase 2: Stiffness Matrix (Day 2)

#### Constitutive Matrix (D-matrix) - 3D Elasticity
```rust
fn constitutive_matrix(material: &Material) -> Result<SMatrix<f64, 6, 6>, String> {
    // Isotropic linear elastic material
    // D relates stresses to strains: {σ} = [D]{ε}
    //
    // For 3D elasticity (Voigt notation):
    //       [1-ν   ν     ν     0       0       0    ]
    //       [ν     1-ν   ν     0       0       0    ]
    //   E   [ν     ν     1-ν   0       0       0    ]
    // ───── [0     0     0   (1-2ν)/2  0       0    ]
    // (1+ν)(1-2ν)
    //       [0     0     0     0     (1-2ν)/2  0    ]
    //       [0     0     0     0       0     (1-2ν)/2]

    let E = material.elastic_modulus.ok_or("Missing elastic modulus")?;
    let nu = material.poissons_ratio.ok_or("Missing Poisson's ratio")?;

    let factor = E / ((1.0 + nu) * (1.0 - 2.0 * nu));
    let lambda = nu * E / ((1.0 + nu) * (1.0 - 2.0 * nu));
    let mu = E / (2.0 * (1.0 + nu));

    // Build D matrix...
}
```

#### Element Stiffness Matrix
```rust
impl Element for C3D8 {
    fn stiffness_matrix(&self, nodes: &[Node], material: &Material)
        -> Result<DMatrix<f64>, String> {
        // K_e = ∫∫∫ B^T D B |J| dξ dη dζ
        //     ≈ Σ w_i B_i^T D B_i |J_i|  (Gauss quadrature)

        let D = self.constitutive_matrix(material)?;
        let mut K = DMatrix::zeros(24, 24);  // 8 nodes × 3 DOFs

        // 2×2×2 Gauss quadrature
        let gp = 1.0 / f64::sqrt(3.0);  // Gauss point location
        let w = 1.0;  // Gauss weight

        for xi in [-gp, gp] {
            for eta in [-gp, gp] {
                for zeta in [-gp, gp] {
                    let B = self.strain_displacement_matrix(nodes, xi, eta, zeta)?;
                    let J = self.jacobian(nodes, xi, eta, zeta)?;
                    let det_J = J.determinant();

                    // K += B^T * D * B * det(J) * w^3
                    K += B.transpose() * D * B * det_J * w * w * w;
                }
            }
        }

        Ok(K)
    }
}
```

#### Unit Tests
```rust
#[test]
fn stiffness_matrix_symmetry() {
    // K should be symmetric within tolerance
}

#[test]
fn stiffness_matrix_positive_definite() {
    // All eigenvalues should be >= 0
    // (with 6 near-zero for rigid body modes)
}

#[test]
fn unit_cube_under_tension() {
    // Analytical: u = σL/E
    // Compare FEA result
}
```

---

### Phase 3: Mass Matrix (Day 3)

#### Consistent Mass Matrix
```rust
impl Element for C3D8 {
    fn mass_matrix(&self, nodes: &[Node], material: &Material)
        -> Result<DMatrix<f64>, String> {
        // M_e = ∫∫∫ ρ N^T N |J| dξ dη dζ
        //     ≈ Σ w_i ρ N_i^T N_i |J_i|

        let rho = material.density.ok_or("Missing density")?;
        let mut M = DMatrix::zeros(24, 24);

        // 2×2×2 Gauss quadrature
        let gp = 1.0 / f64::sqrt(3.0);
        let w = 1.0;

        for xi in [-gp, gp] {
            for eta in [-gp, gp] {
                for zeta in [-gp, gp] {
                    let N = self.shape_functions(xi, eta, zeta);
                    let J = self.jacobian(nodes, xi, eta, zeta)?;
                    let det_J = J.determinant();

                    // M += N^T * N * rho * det(J) * w^3
                    for i in 0..8 {
                        for j in 0..8 {
                            for k in 0..3 {  // 3 DOFs per node
                                let row = i * 3 + k;
                                let col = j * 3 + k;
                                M[(row, col)] += N[i] * N[j] * rho * det_J * w * w * w;
                            }
                        }
                    }
                }
            }
        }

        Ok(M)
    }
}
```

#### Unit Tests
```rust
#[test]
fn mass_conservation() {
    // Total mass = Σ M_ij should equal ρ*Volume
}

#[test]
fn mass_matrix_symmetry() {
    // M should be symmetric
}
```

---

### Phase 4: Integration into Factory (Day 3)

#### Update Element Factory
**File**: `crates/ccx-solver/src/elements/factory.rs`

```rust
use crate::elements::{C3D8, ...};

pub enum DynamicElement {
    Truss(Truss2D),
    Beam(Beam31),
    Shell4(S4),
    Solid8(C3D8),  // NEW
}

impl DynamicElement {
    pub fn from_mesh_element(...) -> Option<Self> {
        match elem_type {
            ElementType::C3D8 => {
                if nodes.len() != 8 {
                    return None;
                }
                let node_array = [nodes[0], nodes[1], ..., nodes[7]];
                let solid = C3D8::new(elem_id, node_array);
                Some(DynamicElement::Solid8(solid))
            }
            // ... other types
        }
    }

    pub fn stiffness_matrix(&self, ...) -> Result<DMatrix<f64>, String> {
        match self {
            DynamicElement::Solid8(solid) => solid.stiffness_matrix(nodes, material),
            // ...
        }
    }
}
```

#### Update Validation Command
**File**: `crates/ccx-cli/src/main.rs`

```rust
// Check for supported element types (T3D2, B31, S4, C3D8)
let supported_types = ["T3D2", "B31", "S4", "C3D8"];
```

#### Update Analysis Pipeline
**File**: `crates/ccx-solver/src/analysis.rs`

```rust
let has_supported_elements = mesh.elements.values().any(|e| matches!(
    e.element_type,
    ElementType::T3D2
        | ElementType::B31
        | ElementType::S4
        | ElementType::C3D8  // NEW
));
```

---

### Phase 5: Validation (Day 4)

#### Analytical Test Cases

1. **Uniaxial Tension Test**
   ```
   Geometry: 1×1×1 unit cube, 1 element
   BC: Fix one face, apply uniform traction on opposite face
   Load: σ = 100 MPa
   Theory: ε = σ/E, u = εL
   Tolerance: < 1% error
   ```

2. **Pure Shear Test**
   ```
   Geometry: Unit cube
   BC: Apply shear traction
   Theory: γ = τ/G, G = E/(2(1+ν))
   Tolerance: < 2% error
   ```

3. **Cantilever Beam (C3D8 mesh)**
   ```
   Geometry: 10×1×1 beam, 10 C3D8 elements
   BC: Fix one end
   Load: Tip force
   Theory: δ = PL³/(3EI)
   Tolerance: < 5% error (coarse mesh)
   ```

#### Test Files
Create: `crates/ccx-solver/tests/solid_validation.rs`

```rust
#[test]
fn c3d8_uniaxial_tension() {
    // Create single element mesh
    // Apply boundary conditions
    // Run solver
    // Compare displacement with analytical
}

#[test]
fn c3d8_patch_test() {
    // Standard FEA patch test for C3D8
    // Should pass exactly for constant stress
}
```

#### Run Against Fixtures
```bash
cargo run --package ccx-cli --release -- validate
```

Expected: 99 additional tests become runnable (total: 108 tests, 17.2% coverage)

---

## Implementation Checklist

### Core Implementation
- [ ] Create `solid.rs` with C3D8 struct
- [ ] Implement shape functions (trilinear)
- [ ] Implement shape function derivatives
- [ ] Implement Jacobian computation
- [ ] Implement B-matrix (strain-displacement)
- [ ] Implement D-matrix (constitutive)
- [ ] Implement element stiffness matrix
- [ ] Implement element mass matrix
- [ ] Add unit tests for shape functions
- [ ] Add unit tests for stiffness matrix

### Integration
- [ ] Update `elements/mod.rs` to export C3D8
- [ ] Update `factory.rs` with C3D8 case
- [ ] Update `mesh.rs` ElementType enum with C3D8
- [ ] Update `analysis.rs` supported element check
- [ ] Update validation command element detection
- [ ] Update `VALIDATION_SYSTEM.md` documentation

### Testing
- [ ] Write analytical validation tests
- [ ] Test uniaxial tension (< 1% error)
- [ ] Test pure shear (< 2% error)
- [ ] Test patch test (exact pass)
- [ ] Run fixture validation suite
- [ ] Verify 99+ tests runnable
- [ ] Document results

---

## Key Formulas Reference

### 3D Elasticity (Voigt Notation)
```
Stress-strain: {σ} = [D]{ε}
Strain components: {ε} = [εxx, εyy, εzz, γxy, γyz, γzx]ᵀ
Stress components: {σ} = [σxx, σyy, σzz, τxy, τyz, τzx]ᵀ
```

### Gauss Quadrature (2×2×2)
```
Location: ξ, η, ζ = ±1/√3
Weights: w = 1.0 (all points)
Integral: ∫∫∫ f dV ≈ Σᵢ₌₁⁸ w³ f(ξᵢ, ηᵢ, ζᵢ) |J(ξᵢ, ηᵢ, ζᵢ)|
```

### Material Properties
```
E = Young's modulus
ν = Poisson's ratio
G = E/(2(1+ν)) = Shear modulus
λ = νE/((1+ν)(1-2ν)) = Lamé's first parameter
μ = G = Lamé's second parameter
ρ = density
```

---

## Estimated Timeline

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Phase 1 | 1 day | Shape functions, Jacobian, B-matrix |
| Phase 2 | 1 day | Stiffness matrix implementation |
| Phase 3 | 1 day | Mass matrix, factory integration |
| Phase 4 | 0.5 day | Validation command updates |
| Phase 5 | 0.5-1 day | Testing and validation |
| **Total** | **3.5-4 days** | **Working C3D8 element** |

---

## Success Criteria

✅ **Minimum Viable**:
- Single C3D8 element solves correctly
- Uniaxial tension test < 1% error
- Integration with validation system
- At least 10 fixture tests passing

✅ **Full Success**:
- All analytical tests passing
- 90+ fixture tests passing
- Coverage increased to 17%+
- Comprehensive documentation

---

## References

- Hughes, T.J.R. (2000). *The Finite Element Method*
- Zienkiewicz, O.C. & Taylor, R.L. (2000). *The Finite Element Method*
- Bathe, K.J. (1996). *Finite Element Procedures*
- CalculiX CrunchiX User's Manual v2.20

---

**Ready to implement!** This plan provides a complete roadmap for adding C3D8 support with the highest impact on validation coverage.
