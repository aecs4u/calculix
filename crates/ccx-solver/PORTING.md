# CalculiX Solver Migration - Porting Guide

## Overview

This document describes the strategy and guidelines for porting the legacy CalculiX CCX 2.23 solver from C/Fortran to Rust.

## Migration Status

Current status as of 2026-02-08:
- **Total legacy units**: 1,199 source files
- **Superseded Fortran**: 986 files (marked as legacy/unused)
- **Pending units**: 213 files
- **Ported units**: 5 files
  - `compare.c` - String comparison utility
  - `strcmp1.c` - String comparison with special null handling
  - `superseded/bsort.f` - Bin sorting for spatial data
  - `superseded/cident.f` - Fortran-style string search
  - `superseded/insertsortd.f` - Insertion sort for small arrays

## Porting Strategy

### Phase 1: Utility Functions (Current)
Port foundational utility functions that are used throughout the codebase:
- String manipulation (compare, strcmp1, etc.)
- Sorting algorithms (bsort, insertsortd, etc.)
- Mathematical utilities
- Memory management wrappers (convert to Rust idioms)

### Phase 2: Data Structures
Port core data structures:
- Element definitions
- Node structures
- Material property containers
- Boundary condition representations

### Phase 3: Core Algorithms
Port computational kernels:
- Element stiffness matrix assembly
- System matrix operations
- Linear solvers
- Time integration schemes

### Phase 4: Analysis Types
Implement analysis capabilities:
- Linear static analysis
- Nonlinear static
- Modal analysis
- Heat transfer
- Dynamic analysis

## Porting Guidelines

### 1. File Organization
```
crates/ccx-solver/src/ported/
├── mod.rs              # Module exports
├── compare.rs          # C utility functions
├── strcmp1.rs          #
├── bsort.rs            # Fortran utility functions
├── cident.rs           #
├── insertsortd.rs      #
└── superseded_fortran.rs  # Catalog of superseded files
```

### 2. Code Structure

Each ported routine should include:
- Module-level documentation with original source reference
- Function documentation with examples
- Comprehensive unit tests
- Doctests for public APIs

Example:
```rust
//! Rust port of `original_file.f`.
//!
//! Brief description of functionality.

/// Function documentation with examples.
///
/// # Arguments
/// * `param1` - Description
///
/// # Examples
/// ```
/// use ccx-solver::ported::function_name;
/// // Example usage
/// ```
pub fn function_name(param1: Type) -> ReturnType {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_case() {
        // Test implementation
    }
}
```

### 3. Naming Conventions

- **Functions**: Use snake_case (Rust convention)
- **Types**: Use PascalCase
- **Constants**: Use SCREAMING_SNAKE_CASE
- Preserve original function names where possible for traceability

### 4. Type Mappings

| Fortran/C Type | Rust Type |
|----------------|-----------|
| `integer` / `ITG` | `i32` or `i64` (context-dependent) |
| `real*8` / `double` | `f64` |
| `real*4` / `float` | `f32` |
| `character` | `&str` or `String` |
| Pointers | References or `Option<&T>` |
| Arrays | Slices `&[T]` or `Vec<T>` |

### 5. Error Handling

- Replace Fortran/C error codes with `Result<T, E>`
- Define custom error types for domain-specific errors
- Use descriptive error variants

Example:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BSortError {
    InvalidDmax,
    InvalidBounds,
    MissingX { index: usize },
    MissingY { index: usize },
    MissingBin { index: usize },
}
```

### 6. Testing Strategy

1. **Unit Tests**: Test individual functions with various inputs
2. **Property Tests**: Use `proptest` for algorithmic correctness
3. **Integration Tests**: Test combinations of ported routines
4. **Regression Tests**: Compare results with legacy solver on fixtures
5. **Doctests**: Ensure examples in documentation work

### 7. Documentation Requirements

Each ported routine must include:
- Source file reference in module doc comment
- Original author attribution where available
- Algorithm description or reference
- Example usage in function docs
- Parameter descriptions
- Return value description

### 8. Tracking Progress

Update `PORTED_UNITS` in `src/lib.rs` when porting new units:
```rust
pub const PORTED_UNITS: &[&str] = &[
    "compare.c",
    "strcmp1.c",
    "superseded/bsort.f",
    // Add new ported units here
];
```

Run migration report to track progress:
```bash
cargo run --bin ccx_solver -- migration-report
```

## Testing Fixtures

The repository includes 638 CalculiX input files in `tests/fixtures/solver/` that can be used for:
- Parser validation
- Model analysis testing
- Integration testing
- Regression testing

Analyze fixtures:
```bash
cargo run --bin ccx_solver -- analyze-fixtures tests/fixtures/solver
```

## Build and Test

```bash
# Run all tests
cargo test

# Build release
cargo build --release

# Check migration status
cargo run --bin ccx_solver -- migration-report

# Analyze a specific fixture
cargo run --bin ccx_solver -- analyze tests/fixtures/solver/beamcr4.inp
```

## Next Steps

1. **Port More Utilities**: Continue porting utility functions (nident, dsort, etc.)
2. **Element Library**: Begin porting element type definitions
3. **Matrix Assembly**: Port matrix assembly routines
4. **Linear Solver Interface**: Create wrapper for linear algebra libraries
5. **Basic Solver Loop**: Implement simple linear static analysis

## Resources

- Original CalculiX Documentation: http://www.dhondt.de/
- CalculiX Source: `calculix_migration_tooling/ccx_2.23/`
- Test Fixtures: `tests/fixtures/solver/`
- Build Script: `crates/ccx-solver/build.rs`

## Contributing

When porting new routines:
1. Read the original source carefully
2. Understand the algorithm and its context
3. Write Rust implementation following guidelines
4. Add comprehensive tests
5. Update PORTED_UNITS list
6. Verify migration report updates correctly
7. Ensure all tests pass
