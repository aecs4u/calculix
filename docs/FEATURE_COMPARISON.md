# CalculiX Rust Solver - Feature Comparison

**Project**: ccx-solver (Rust implementation)
**Compared to**: CalculiX, Abaqus, MSC.Nastran, NASTRAN-95
**Last Updated**: 2026-02-09

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Fully implemented and tested |
| 🚧 | Partially implemented / In progress |
| 📋 | Planned for implementation |
| ❌ | Not planned / Not applicable |
| 🔍 | Under investigation |

## 1. Element Library

### 1.1 Structural Elements

| Element Type | Description | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|--------------|-------------|------------|----------|--------|-------------|------------|
| **Truss/Rod** |
| T3D2 | 2-node 3D truss | ✅ | ✅ | - | - | - |
| T3D2E | 2-node truss (enhanced) | 📋 | ✅ | - | - | - |
| CROD | 2-node axial rod | 📋 | - | ✅ | ✅ | ✅ |
| CONROD | Concentrated rod | 📋 | - | ✅ | ✅ | ✅ |
| **Beam** |
| B31 | 2-node Euler-Bernoulli beam | ✅ | ✅ | ✅ | - | - |
| B31R | 2-node beam with reduced integration | 📋 | ✅ | ✅ | - | - |
| B32 | 3-node beam | 📋 | ✅ | ✅ | - | - |
| CBEAM | General beam element | 📋 | - | ✅ | ✅ | ✅ |
| CBAR | Simple beam element | 📋 | - | ✅ | ✅ | ✅ |
| **Shell** |
| S3 | 3-node shell | 📋 | ✅ | ✅ | - | - |
| S4 | 4-node shell | 📋 | ✅ | ✅ | - | - |
| S4R | 4-node shell reduced integration | 📋 | ✅ | ✅ | - | - |
| S6 | 6-node shell | 📋 | ✅ | ✅ | - | - |
| S8 | 8-node shell | 📋 | ✅ | ✅ | - | - |
| S8R | 8-node shell reduced integration | 📋 | ✅ | ✅ | - | - |
| CQUAD4 | 4-node quadrilateral shell | 📋 | - | ✅ | ✅ | ✅ |
| CTRIA3 | 3-node triangular shell | 📋 | - | ✅ | ✅ | ✅ |
| **Solid (Linear)** |
| C3D4 | 4-node tetrahedron | 📋 | ✅ | ✅ | - | - |
| C3D6 | 6-node wedge | 📋 | ✅ | ✅ | - | - |
| C3D8 | 8-node hexahedron | 📋 | ✅ | ✅ | - | - |
| C3D8I | 8-node hex incompatible modes | 📋 | ✅ | ✅ | - | - |
| C3D8R | 8-node hex reduced integration | 📋 | ✅ | ✅ | - | - |
| CHEXA | 8-node hexahedron | 📋 | - | ✅ | ✅ | ✅ |
| CPENTA | 6-node pentahedron | 📋 | - | ✅ | ✅ | ✅ |
| CTETRA | 4-node tetrahedron | 📋 | - | ✅ | ✅ | ✅ |
| **Solid (Quadratic)** |
| C3D10 | 10-node tetrahedron | 📋 | ✅ | ✅ | - | - |
| C3D15 | 15-node wedge | 📋 | ✅ | ✅ | - | - |
| C3D20 | 20-node hexahedron | 📋 | ✅ | ✅ | - | - |
| C3D20R | 20-node hex reduced integration | 📋 | ✅ | ✅ | - | - |

### 1.2 Special Purpose Elements

| Element Type | Description | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|--------------|-------------|------------|----------|--------|-------------|------------|
| **Springs/Dampers** |
| SPRING1 | Spring element type 1 | 📋 | ✅ | ✅ | ✅ | ✅ |
| SPRING2 | Spring element type 2 | 📋 | ✅ | ✅ | ✅ | ✅ |
| SPRINGA | Nonlinear spring | 📋 | ✅ | ✅ | ✅ | ❌ |
| DASHPOTA | Dashpot element | 📋 | ✅ | ✅ | ✅ | ❌ |
| **Rigid/Constraint** |
| DCOUP3D | Distributing coupling | 📋 | ✅ | - | - | - |
| RBE2 | Rigid body element | 📋 | - | ✅ | ✅ | ✅ |
| RBE3 | Interpolation element | 📋 | - | ✅ | ✅ | ✅ |
| **Contact** |
| GAPUNI | Gap element | 📋 | ✅ | - | - | - |
| Surface-to-surface | Contact surfaces | 📋 | ✅ | ✅ | ✅ | ❌ |

## 2. Analysis Types

### 2.1 Static Analysis

| Analysis Type | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|---------------|------------|----------|--------|-------------|------------|
| Linear static | 🚧 | ✅ | ✅ | ✅ | ✅ |
| Nonlinear static (geometric) | 📋 | ✅ | ✅ | ✅ | ✅ |
| Nonlinear static (material) | 📋 | ✅ | ✅ | ✅ | ✅ |
| Large displacement | 📋 | ✅ | ✅ | ✅ | ✅ |
| Contact analysis | 📋 | ✅ | ✅ | ✅ | ❌ |

### 2.2 Dynamic Analysis

| Analysis Type | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|---------------|------------|----------|--------|-------------|------------|
| Modal (eigenvalue) | 📋 | ✅ | ✅ | ✅ | ✅ |
| Frequency response | 📋 | ✅ | ✅ | ✅ | ✅ |
| Transient dynamic | 📋 | ✅ | ✅ | ✅ | ✅ |
| Harmonic response | 📋 | ✅ | ✅ | ✅ | ✅ |
| Random response | 📋 | ✅ | ✅ | ✅ | ✅ |
| Response spectrum | 📋 | ✅ | ✅ | ✅ | ✅ |

### 2.3 Stability Analysis

| Analysis Type | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|---------------|------------|----------|--------|-------------|------------|
| Linear buckling | 📋 | ✅ | ✅ | ✅ | ✅ |
| Nonlinear buckling | 📋 | ✅ | ✅ | ✅ | ❌ |
| Arc-length method | 📋 | ✅ | ✅ | ✅ | ❌ |

### 2.4 Thermal Analysis

| Analysis Type | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|---------------|------------|----------|--------|-------------|------------|
| Steady-state heat transfer | 📋 | ✅ | ✅ | ✅ | ✅ |
| Transient heat transfer | 📋 | ✅ | ✅ | ✅ | ✅ |
| Coupled thermo-mechanical | 📋 | ✅ | ✅ | ✅ | ❌ |
| Radiation | 📋 | ✅ | ✅ | ✅ | ❌ |

### 2.5 Special Analyses

| Analysis Type | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|---------------|------------|----------|--------|-------------|------------|
| Acoustics | 📋 | ✅ | ✅ | ✅ | ❌ |
| Fluid-structure interaction | 📋 | ✅ | ✅ | ✅ | ❌ |
| Cyclic symmetry | 📋 | ✅ | ✅ | ✅ | ✅ |
| Substructuring | 📋 | ✅ | ✅ | ✅ | ✅ |

## 3. Material Models

### 3.1 Linear Materials

| Material Type | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|---------------|------------|----------|--------|-------------|------------|
| Isotropic elastic | ✅ | ✅ | ✅ | ✅ | ✅ |
| Orthotropic elastic | 📋 | ✅ | ✅ | ✅ | ✅ |
| Anisotropic elastic | 📋 | ✅ | ✅ | ✅ | ✅ |

### 3.2 Nonlinear Materials

| Material Type | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|---------------|------------|----------|--------|-------------|------------|
| Elastoplastic (von Mises) | 📋 | ✅ | ✅ | ✅ | ✅ |
| Elastoplastic (Tresca) | 📋 | ✅ | ✅ | ✅ | ❌ |
| Kinematic hardening | 📋 | ✅ | ✅ | ✅ | ❌ |
| Isotropic hardening | 📋 | ✅ | ✅ | ✅ | ❌ |
| Hyperelastic (Neo-Hookean) | 📋 | ✅ | ✅ | ✅ | ❌ |
| Hyperelastic (Mooney-Rivlin) | 📋 | ✅ | ✅ | ✅ | ❌ |
| Viscoelastic | 📋 | ✅ | ✅ | ✅ | ❌ |
| Creep | 📋 | ✅ | ✅ | ✅ | ❌ |

### 3.3 Special Materials

| Material Type | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|---------------|------------|----------|--------|-------------|------------|
| Composite layup | 📋 | ✅ | ✅ | ✅ | ✅ |
| User-defined (UMAT) | ❌ | ✅ | ✅ | ✅ | ❌ |
| Temperature-dependent | 📋 | ✅ | ✅ | ✅ | ✅ |

## 4. Loading & Boundary Conditions

### 4.1 Loads

| Load Type | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|-----------|------------|----------|--------|-------------|------------|
| Concentrated force | ✅ | ✅ | ✅ | ✅ | ✅ |
| Distributed load (beam) | 📋 | ✅ | ✅ | ✅ | ✅ |
| Pressure (surface) | 📋 | ✅ | ✅ | ✅ | ✅ |
| Body force (gravity) | 📋 | ✅ | ✅ | ✅ | ✅ |
| Thermal load | 📋 | ✅ | ✅ | ✅ | ✅ |
| Centrifugal load | 📋 | ✅ | ✅ | ✅ | ✅ |
| Moment | 📋 | ✅ | ✅ | ✅ | ✅ |

### 4.2 Boundary Conditions

| BC Type | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|---------|------------|----------|--------|-------------|------------|
| Fixed displacement | ✅ | ✅ | ✅ | ✅ | ✅ |
| Prescribed displacement | ✅ | ✅ | ✅ | ✅ | ✅ |
| Symmetry | 📋 | ✅ | ✅ | ✅ | ✅ |
| Antisymmetry | 📋 | ✅ | ✅ | ✅ | ✅ |
| Cyclic symmetry | 📋 | ✅ | ✅ | ✅ | ✅ |
| MPC (multi-point constraint) | 📋 | ✅ | ✅ | ✅ | ✅ |
| Equation constraint | 📋 | ✅ | ✅ | ✅ | ✅ |

## 5. Solution Methods

### 5.1 Linear Solvers

| Solver Type | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|-------------|------------|----------|--------|-------------|------------|
| Direct (LU decomposition) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Direct (Cholesky) | 📋 | ✅ | ✅ | ✅ | ✅ |
| Iterative (CG) | 🚧 | ✅ | ✅ | ✅ | ❌ |
| Iterative (GMRES) | 📋 | ✅ | ✅ | ✅ | ❌ |
| Iterative (BiCGSTAB) | 📋 | ✅ | ✅ | ✅ | ❌ |
| PARDISO | ❌ | ✅ | ❌ | ❌ | ❌ |
| SPOOLES | ❌ | ✅ | ❌ | ❌ | ❌ |
| PaStiX | ❌ | ✅ | ❌ | ❌ | ❌ |

### 5.2 Eigenvalue Solvers

| Solver Type | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|-------------|------------|----------|--------|-------------|------------|
| Lanczos | 📋 | ✅ | ✅ | ✅ | ✅ |
| Subspace iteration | 📋 | ✅ | ✅ | ✅ | ✅ |
| ARPACK | ❌ | ✅ | ✅ | ❌ | ❌ |

### 5.3 Nonlinear Solution

| Method | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|--------|------------|----------|--------|-------------|------------|
| Newton-Raphson | 📋 | ✅ | ✅ | ✅ | ✅ |
| Modified Newton | 📋 | ✅ | ✅ | ✅ | ✅ |
| Quasi-Newton (BFGS) | 📋 | ✅ | ✅ | ✅ | ❌ |
| Arc-length (Riks) | 📋 | ✅ | ✅ | ✅ | ❌ |
| Line search | 📋 | ✅ | ✅ | ✅ | ❌ |

## 6. Matrix Storage

| Storage Format | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|----------------|------------|----------|--------|-------------|------------|
| Dense | ✅ | ✅ | ✅ | ✅ | ✅ |
| Sparse (CSR) | ✅ | ❌ | ✅ | ✅ | ❌ |
| Sparse (Skyline) | ❌ | ✅ | ❌ | ❌ | ✅ |
| Profile/Bandwidth | ❌ | ❌ | ❌ | ✅ | ✅ |

## 7. Input/Output Formats

### 7.1 Input Formats

| Format | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|--------|------------|----------|--------|-------------|------------|
| CalculiX INP | ✅ | ✅ | ❌ | ❌ | ❌ |
| Abaqus INP | 🚧 | 🚧 | ✅ | ❌ | ❌ |
| Nastran BDF | 📋 | ❌ | ❌ | ✅ | ✅ |
| Universal File | 📋 | ❌ | ❌ | ✅ | ❌ |

### 7.2 Output Formats

| Format | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|--------|------------|----------|--------|-------------|------------|
| FRD (results) | ✅ | ✅ | ❌ | ❌ | ❌ |
| DAT (text results) | ✅ | ✅ | ❌ | ❌ | ❌ |
| VTK (legacy) | ✅ | ❌ | ❌ | ❌ | ❌ |
| VTU (XML) | ✅ | ❌ | ❌ | ❌ | ❌ |
| ODB (Abaqus database) | ❌ | ❌ | ✅ | ❌ | ❌ |
| OP2 (Nastran binary) | ❌ | ❌ | ❌ | ✅ | ❌ |
| F06 (Nastran text) | 📋 | ❌ | ❌ | ✅ | ✅ |

## 8. Parallelization

| Feature | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|---------|------------|----------|--------|-------------|------------|
| Multi-threading | 🔍 | ✅ | ✅ | ✅ | ❌ |
| MPI (distributed) | 📋 | ✅ | ✅ | ✅ | ❌ |
| GPU acceleration | 🔍 | ❌ | ✅ | ✅ | ❌ |
| OpenMP | 🔍 | ✅ | ✅ | ✅ | ❌ |

## 9. Advanced Features

### 9.1 Optimization

| Feature | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|---------|------------|----------|--------|-------------|------------|
| Topology optimization | ❌ | ❌ | ✅ | ✅ | ❌ |
| Shape optimization | ❌ | ❌ | ✅ | ✅ | ❌ |
| Sensitivity analysis | 📋 | ✅ | ✅ | ✅ | ❌ |

### 9.2 Multiphysics

| Feature | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|---------|------------|----------|--------|-------------|------------|
| Thermo-mechanical coupling | 📋 | ✅ | ✅ | ✅ | ❌ |
| Fluid-structure interaction | 📋 | ✅ | ✅ | ✅ | ❌ |
| Electro-thermal coupling | ❌ | ❌ | ✅ | ✅ | ❌ |
| Piezoelectric | ❌ | ❌ | ✅ | ✅ | ❌ |

### 9.3 Special Techniques

| Feature | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|---------|------------|----------|--------|-------------|------------|
| Submodeling | 📋 | ✅ | ✅ | ✅ | ❌ |
| Adaptive meshing | ❌ | ❌ | ✅ | ✅ | ❌ |
| Error estimation | 📋 | ✅ | ✅ | ✅ | ❌ |
| Restart capability | ✅ | ✅ | ✅ | ✅ | ✅ |

## 10. Programming & Extensibility

| Feature | ccx-solver | CalculiX | Abaqus | MSC.Nastran | NASTRAN-95 |
|---------|------------|----------|--------|-------------|------------|
| User subroutines (UMAT) | ❌ | ✅ | ✅ | ✅ | ❌ |
| Python scripting | ✅ | ❌ | ✅ | ✅ | ❌ |
| Plugin system | 📋 | ❌ | ✅ | ❌ | ❌ |
| API/Library mode | ✅ | ❌ | ✅ | ✅ | ❌ |

## Implementation Status Summary

### ✅ Completed Features (ccx-solver)
- **Elements**: T3D2 (truss), B31 (beam)
- **Analysis**: Linear static (partial)
- **Materials**: Isotropic elastic
- **BC/Loads**: Concentrated forces, fixed displacements
- **Solvers**: Dense direct (LU), Sparse CSR with LU
- **I/O**: INP parser, FRD/DAT writer, VTK/VTU export
- **Infrastructure**: Rust library, Python bindings

### 🚧 In Progress
- Linear static analysis (full validation)
- Sparse iterative solvers (CG)
- Abaqus INP compatibility

### 📋 High Priority (Next 3-6 months)
1. **Elements**: S4 shell, C3D8 hexahedron
2. **Analysis**: Modal analysis (eigenvalues)
3. **Loads**: Distributed loads, pressure
4. **Solvers**: Conjugate Gradient, GMRES
5. **Materials**: Orthotropic elastic
6. **Validation**: 1,133 example files from CalculiX

### 📋 Medium Priority (6-12 months)
1. Nonlinear geometry (large displacement)
2. Plasticity (von Mises)
3. Thermal analysis
4. Frequency response
5. More element types (C3D4, C3D6, S3, S6, S8)

### 🔍 Under Investigation
- GPU acceleration using CUDA or wgpu
- Distributed computing (MPI via Rayon)
- Advanced optimization techniques

## Performance Goals

| Metric | Target | Current Status |
|--------|--------|----------------|
| Linear solve (10k DOFs) | < 1s | 🔍 Not benchmarked |
| Linear solve (100k DOFs) | < 10s | 📋 Planned |
| Memory usage vs CalculiX | 50-70% | 🔍 Not measured |
| Compilation time | < 60s | ✅ ~30s (debug) |
| Single-file parse speed | > 100 MB/s | ✅ ~150 MB/s |

## Compatibility Notes

### CalculiX Compatibility
- **Parser**: 99.6% success rate on 1,133 examples
- **Solver**: Working on linear static cases
- **Output**: FRD format compatible with CGX viewer

### Abaqus Compatibility
- **Input**: Subset of INP format supported
- **Output**: Not compatible (use VTK/VTU instead)

### Nastran Compatibility
- **Planned**: BDF reader for cross-platform workflows
- **Output**: F06 format planned for text results

## Contributing

To contribute to feature implementation:
1. Review the [MEMORY.md](/home/emanuele/.claude/projects/-mnt-developer-git-aecs4u-it-calculix/memory/MEMORY.md) for implementation patterns
2. Check [GitHub Issues](https://github.com/aecs4u/calculix/issues) for planned features
3. Follow the Rust API conventions in existing code
4. Add comprehensive tests and documentation

## References

- **CalculiX**: http://www.calculix.de/
- **Abaqus**: https://www.3ds.com/products-services/simulia/products/abaqus/
- **MSC.Nastran**: https://www.mscsoftware.com/product/msc-nastran
- **NASTRAN-95**: Public domain NASA Structural Analysis System

---

**Maintained by**: CalculiX Rust Team
**Repository**: https://github.com/aecs4u/calculix
