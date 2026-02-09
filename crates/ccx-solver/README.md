# ccx-solver

Migration-stage Rust solver module for CalculiX CCX.

## Features

- **Build-time catalog**: Automatically scans and catalogs 1,199 legacy source files from `ccx_2.23/src`
- **Progressive porting**: Incrementally migrate C and Fortran routines to safe Rust
- **Migration tracking**: Built-in reporting of porting progress
- **Test fixtures**: 638 validated CalculiX input files for testing
- **Comprehensive tests**: 24 unit tests covering ported functionality

## Ported Routines (5 total)

### C Utilities
- [`compare.c`](src/ported/compare.rs) - String comparison utility
- [`strcmp1.c`](src/ported/strcmp1.rs) - String comparison with special null handling

### Fortran Utilities
- [`bsort.f`](src/ported/bsort.rs) - Bin sorting for spatial data structures
- [`cident.f`](src/ported/cident.rs) - Fortran-style binary search with string comparison
- [`insertsortd.f`](src/ported/insertsortd.rs) - Insertion sort for small f64 arrays

## Usage

### Check Migration Progress
```bash
cargo run --bin ccx_solver -- migration-report
```

Output:
```
legacy_units_total: 1199
ported_units: 5
superseded_fortran_units: 986
pending_units: 213
ported_list: compare.c, strcmp1.c, superseded/bsort.f, superseded/cident.f, superseded/insertsortd.f
```

### Analyze Input Files
```bash
# Analyze a single input file
cargo run --bin ccx_solver -- analyze tests/fixtures/solver/beamcr4.inp

# Analyze all fixtures in a directory
cargo run --bin ccx_solver -- analyze-fixtures tests/fixtures/solver
```

## Development

### Running Tests
```bash
# Run all tests
cargo test --package ccx-solver

# Run with output
cargo test --package ccx-solver -- --nocapture
```

### Building
```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release
```

## Migration Strategy

See [PORTING.md](PORTING.md) for detailed porting guidelines and strategy.

### Current Phase: Utility Functions
Porting foundational utilities used throughout the codebase. This includes:
- String manipulation functions
- Sorting and searching algorithms
- Mathematical utilities
- Data structure helpers

### Next Steps
1. Port more utility functions (nident, dsort, etc.)
2. Port core data structures (element definitions, nodes, materials)
3. Implement element stiffness matrix assembly
4. Create basic solver entry point for linear static analysis
5. Add integration tests using test fixtures

## Architecture

```
ccx-solver/
├── build.rs              # Scans legacy source and generates catalog
├── src/
│   ├── lib.rs           # Migration tracking and reporting API
│   ├── main.rs          # CLI for migration reports and analysis
│   └── ported/          # Ported Rust implementations
│       ├── mod.rs
│       ├── bsort.rs
│       ├── cident.rs
│       ├── compare.rs
│       ├── insertsortd.rs
│       ├── strcmp1.rs
│       └── superseded_fortran.rs
└── PORTING.md           # Porting guidelines

```

## Testing

The crate includes comprehensive testing:
- **24 unit tests** covering all ported functionality
- **3 doctests** ensuring documentation examples work
- **638 test fixtures** for integration testing
- 100% test pass rate

## License

This is a port of CalculiX CCX 2.23, which is licensed under GPLv2.
See the original source files for copyright information.
