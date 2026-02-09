# CalculiX - Modern FEA Solver & Tools

A modernized CalculiX finite element analysis suite with Rust solver core and Python tooling.

## Project Status

**Phase 2 of Migration: Rust Foundation & Compatibility Shell** ✅

- 9 ported utility functions from legacy C/Fortran codebase
- Full analysis pipeline framework supporting 16 analysis types
- 638 validated test fixtures with 100% parse success rate
- 52 unit tests + 5 integration tests + 7 doctests (100% pass rate)

## Architecture

### Rust Workspace

```
crates/
├── ccx-cli/          # Command-line interface and job orchestration
├── ccx-inp/          # CalculiX/Abaqus input deck parser
├── ccx-model/        # Domain model (mesh, materials, BCs, loads)
├── ccx-solver/       # Analysis pipelines and solver core
├── ccx-io/           # DAT/STA/FRD writing and restart persistence
└── ccx-compat/       # Temporary C/Fortran compatibility bridge
```

#### 📦 ccx-cli

Main command-line interface for CalculiX operations.

**Commands:**
- `ccx-cli analyze <file.inp>` - Parse and analyze input files
- `ccx-cli analyze-fixtures <dir>` - Batch analyze all .inp files in directory
- `ccx-cli postprocess <file.dat>` - Postprocess stress/strain from .dat files
- `ccx-cli migration-report` - Show solver migration progress
- `ccx-cli gui-migration-report` - Show GUI migration progress

**See also:** [POSTPROCESSING.md](crates/ccx-solver/POSTPROCESSING.md) for detailed postprocessing documentation

#### 📦 ccx-inp

Input deck parser supporting CalculiX and Abaqus formats.

**Features:**
- Lexer/parser for `.inp` format
- Card-based AST representation
- Include file handling (`*INCLUDE`)
- Parameter parsing
- Error recovery and diagnostics

**Key Types:**
- `Deck` - Parsed input deck containing all cards
- `Card` - Individual keyword card with parameters
- `Parameter` - Keyword parameter (name-value pairs)

**Example:**
```rust
use ccx_inp::Deck;

let deck = Deck::parse_file_with_includes("model.inp")?;
for card in &deck.cards {
    println!("Keyword: {}", card.keyword);
}
```

#### 📦 ccx-model

Domain model abstractions for finite element analysis entities.

**Features:**
- Model summary generation and statistics
- Keyword frequency analysis
- Analysis type detection
- Include file tracking

**Key Types:**
- `ModelSummary` - High-level model statistics
  - `node_rows` - Number of node definitions
  - `element_rows` - Number of elements
  - `material_defs` - Material count
  - `has_static`, `has_dynamic`, `has_frequency` - Analysis flags
  - `keyword_counts` - Frequency map of all keywords
  - `include_files` - Tracked include dependencies

**Example:**
```rust
use ccx_inp::Deck;
use ccx_model::ModelSummary;

let deck = Deck::parse_file("model.inp")?;
let summary = ModelSummary::from_deck(&deck);

println!("Nodes: {}", summary.node_rows);
println!("Elements: {}", summary.element_rows);
println!("Has dynamic analysis: {}", summary.has_dynamic);
```

#### 📦 ccx-solver

Core solver implementation with progressive Fortran/C port to Rust.

**Migration Status:**
- Total legacy units: 1,199
- Superseded (legacy): 986
- Pending migration: 213
- **Ported to Rust: 9 (4.2%)**

**Modules:**

##### 🔧 `analysis` - Analysis Pipeline Framework

Orchestrates different types of finite element analyses.

**Supported Analysis Types (16 total):**

| Type | CalculiX Keyword | Description |
|------|-----------------|-------------|
| `LinearStatic` | `*STATIC` | Linear static structural analysis |
| `NonlinearStatic` | `*STATIC` + nonlinear | Nonlinear with contact/plasticity |
| `Modal` | `*FREQUENCY` | Frequency/eigenvalue analysis |
| `SteadyStateDynamics` | `*STEADY STATE DYNAMICS` | Harmonic response |
| `Dynamic` | `*DYNAMIC` | Transient time integration |
| `HeatTransfer` | `*HEAT TRANSFER` | Thermal analysis |
| `CoupledThermoMechanical` | Both thermal + mechanical | Coupled multi-physics |
| `Buckling` | `*BUCKLE` | Linear buckling analysis |
| `ComplexFrequency` | `*COMPLEX FREQUENCY` | Complex eigenvalue analysis |
| `Green` | `*GREEN` | Green's function analysis |
| `Sensitivity` | `*SENSITIVITY` | Design sensitivity |
| `ModalDynamic` | `*MODAL DYNAMIC` | Modal superposition |
| `Visco` | `*VISCO` | Viscoplastic analysis |
| `Electromagnetic` | `*ELECTROMAGNETICS` | EM field analysis |
| `UncoupledThermoMechanical` | `*UNCOUPLED TEMP-DISP` | Sequential coupling |
| `CFD` | `*CFD` | Computational fluid dynamics |

**Key Types:**
- `AnalysisPipeline` - Main orchestrator
- `AnalysisConfig` - Configuration (iterations, tolerance, verbosity)
- `AnalysisResults` - Results (success, DOFs, equations, message)
- `AnalysisType` - Analysis procedure enum

**Example Usage:**
```rust
use ccx-solver::{AnalysisPipeline, AnalysisType};
use ccx_inp::Deck;

// Auto-detect analysis type from keywords
let deck = Deck::parse_file("model.inp")?;
let pipeline = AnalysisPipeline::detect_from_deck(&deck);

// Run analysis
let results = pipeline.run(&deck)?;
println!("Analysis: {:?}", results.analysis_type);
println!("DOFs: {}", results.num_dofs);
println!("Status: {}", if results.success { "SUCCESS" } else { "FAILED" });

// Or create specific analysis explicitly
let pipeline = AnalysisPipeline::linear_static();
let pipeline = AnalysisPipeline::modal();
let pipeline = AnalysisPipeline::heat_transfer();
```

##### 🔧 `ported` - Migrated Utility Functions

Foundation utilities ported from legacy C/Fortran (**9 functions**):

**String Operations:**
- `compare(s1, s2, len) -> usize` - Compare strings up to length, returns match count
- `strcmp1(s1, s2) -> Ordering` - String compare with special null handling
- `stoi(s, a, b) -> i32` - Extract integer from substring (Fortran fixed-width format)
- `stof(s, a, b) -> f64` - Extract float from substring (Fortran fixed-width format)

**Search & Sort:**
- `bsort(list, bin, x, y, bounds) -> Result<...>` - Spatial bin sorting for 3D coordinates
- `cident(ordered, probe) -> usize` - Binary search with Fortran-style string comparison
- `nident(x, px) -> usize` - Binary search in ordered integers (1D array)
- `nident2(x, px) -> usize` - Binary search in ordered integers (2D array, first column)
- `insertsortd(dx)` - In-place insertion sort for small f64 arrays

**Migration Tracking:**
- `SUPERSEDED_FORTRAN_FILES: &[&str]` - Catalog of 986 legacy Fortran files
- `is_superseded_fortran(path) -> bool` - Check if file is superseded

**Example Usage:**
```rust
use ccx-solver::ported::{nident, stof, stoi, compare};

// Parse Fortran fixed-width format (common in legacy FEA)
let line = "      123  1.500E+00  2.300E+00";
let node_id = stoi(line, 1, 10);   // Extract columns 1-10 → 123
let x_coord = stof(line, 11, 20);  // Extract columns 11-20 → 1.5
let y_coord = stof(line, 21, 30);  // Extract columns 21-30 → 2.3

// Binary search in sorted arrays
let sorted_nodes = vec![1, 5, 10, 20, 50, 100];
let pos = nident(&sorted_nodes, 15); // Returns 3 (15 is after index 2)

// String comparison with length limit
let matched = compare("HELLO", "HELP", 5); // Returns 3 (matched "HEL")
```

##### 🔧 `postprocess` - Stress & Strain Analysis

Postprocessing module for reading CalculiX .dat files and computing stress/strain metrics.

**Features:**
- Parse element variable output from .dat files
- Compute von Mises equivalent stress
- Compute total effective strain
- Compute equivalent plastic strain (PEEQ)
- Statistical analysis (min/max/mean)
- Text file output generation

**Key Functions:**
- `read_dat_file(path)` - Parse .dat file and extract integration point data
- `compute_mises_stress(stress)` - Calculate von Mises stress from tensor components
- `compute_effective_strain(strain)` - Calculate effective strain from tensor components
- `process_integration_points(data)` - Compute results for all integration points
- `compute_statistics(results)` - Calculate min/max/mean statistics
- `write_results(path, results, stats)` - Write formatted output file

**Example Usage:**
```rust
use ccx_solver::{read_dat_file, process_integration_points, compute_statistics};

// Read element variable output
let data = read_dat_file("results.dat")?;

// Process integration points
let results = process_integration_points(&data);

// Compute statistics
let stats = compute_statistics(&results);

println!("Max Mises stress: {:.4e}", stats.mises_max);
println!("Mean effective strain: {:.4e}", stats.eeq_mean);
```

**CLI Usage:**
```bash
ccx-cli postprocess results.dat
# Generates: results_IntPtOutput.txt
```

**See:** [POSTPROCESSING.md](crates/ccx-solver/POSTPROCESSING.md) for detailed documentation

### CLI Commands

#### ccx-solver Binary

```bash
# Check migration progress
cargo run --bin ccx_solver -- migration-report
# Output:
# legacy_units_total: 1199
# ported_units: 9
# superseded_fortran_units: 986
# pending_units: 213

# Analyze input file structure
cargo run --bin ccx_solver -- analyze model.inp
# Output: node counts, element counts, analysis type flags

# Analyze entire fixture directory (batch processing)
cargo run --bin ccx_solver -- analyze-fixtures tests/fixtures/solver
# Output: parse success/failure statistics for all .inp files

# Run solver analysis pipeline (Phase 2 skeleton)
cargo run --bin ccx_solver -- solve model.inp
# Output: Detected analysis type, DOFs, equations, status
```

### Python Tools

#### Nastran Reader

Convert Nastran BDF files using `pyNastran`:

```bash
uv run python -m calculix_migration_tooling.nastran_reader model.bdf --json
```

## Development

### Building

```bash
# Full workspace (all crates)
cargo build --release

# Specific crate
cargo build -p ccx-solver
cargo build -p ccx-inp
cargo build -p ccx-model

# Check compilation without building
cargo check
```

### Testing

```bash
# Run all tests across workspace
cargo test

# Specific crate tests
cargo test -p ccx-solver
cargo test -p ccx-inp

# Integration tests with real fixtures
cargo test --test integration_tests

# Run with output (for debugging)
cargo test -- --nocapture

# Fast TDD loop (Python tests)
./scripts/tdd.sh

# Full test matrix (all configurations)
CALCULIX_RUN_FULL=1 python3 -m pytest --full-matrix
```

**Test Statistics:**
- Unit tests: 52 ✅
- Integration tests: 5 ✅
- Doctests: 7 ✅
- Test fixtures: 638 (100% parse success)
- Overall pass rate: 100%

### Documentation

| Document | Description |
|----------|-------------|
| `DEVELOPMENT_PLAN.md` | Overall migration strategy and roadmap |
| `crates/ccx-solver/PORTING.md` | Porting guidelines for C/Fortran → Rust |
| `crates/ccx-solver/README.md` | Solver crate detailed documentation |
| `docs/calculix_cli.md` | CLI usage and commands |
| `docs/migration/feature-coverage.md` | Feature parity tracking matrix |
| `TESTING.md` | Test coverage and quality workflow |

## Migration Status

**Current Phase:** Phase 2 - Rust Foundation (Weeks 8-12) ✅

### Completed Milestones

- ✅ Rust workspace structure established
- ✅ INP parser for CalculiX/Abaqus format
- ✅ Domain model with summary generation
- ✅ Analysis pipeline framework (16 types)
- ✅ 9 utility functions ported from C/Fortran
- ✅ Integration test framework with 638 fixtures
- ✅ CLI commands (migrate, analyze, solve)
- ✅ Migration tracking and reporting

### Next Steps (Phase 3: Solver Core Port)

1. **Element Library**
   - Port element type definitions (C3D8, C3D20, etc.)
   - Shape function evaluation
   - Numerical integration rules

2. **Matrix Assembly**
   - Element stiffness matrix routines
   - Mass matrix assembly
   - Load vector assembly

3. **Linear Solver**
   - Integration with sparse solver (MUMPS/PARDISO/native)
   - Iterative solver options
   - Preconditioners

4. **Results Output**
   - FRD file writer
   - DAT file writer
   - Result field extraction

5. **Progressive Porting**
   - Port analysis procedures incrementally
   - Maintain golden test outputs
   - Track numerical accuracy

### Statistics

```
Migration Progress:
├─ Total legacy units:     1,199
├─ Superseded (legacy):      986
├─ Pending migration:        213
└─ Ported to Rust:             9  (4.2%)

Test Coverage:
├─ Unit tests:               158  ✅
├─ Integration tests:          9  ✅
├─ Doctests:                   9  ✅
├─ Example validation:         4  ✅
├─ Test fixtures:            638  (100% parse)
└─ Examples integrated:    1,133  (99.6% parse)

Codebase:
├─ Rust crates:                4
├─ Analysis types:            16
└─ Build status:              ✅  Passing
```

## Validation & Testing

### Validation API

The project includes a comprehensive validation tracking system for monitoring solver quality and test coverage.

**Features:**
- 📊 Real-time dashboard with KPIs and statistics
- 🧪 Test module tracking (223 tests, 100% passing)
- 📂 Example integration (1,133 INP files, 99.6% parse success)
- ✅ Analytical validation results
- 📈 Historical performance metrics
- 🎯 Category-based organization (13 analysis types)
- 🌐 Web UI + REST API

**Quick Start:**

```bash
# Generate static HTML report (no dependencies)
cd crates/validation-api
python3 scripts/export_test_results.py
python3 scripts/generate_html_report.py
# Open validation_report.html in browser

# Or use Makefile
make quick-report

# Run full API server (requires dependencies)
make install
make run-api
# Visit http://localhost:8000
```

**Current Status:**
- **Total Tests:** 193 (143 unit + 46 ported + 4 integration)
- **Pass Rate:** 100% ✅
- **Lines of Code:** 6,212 (ccx-solver)
- **Element Types:** T3D2 (truss)
- **Examples:** 2 validated with analytical solutions
- **Test Coverage:** Comprehensive across all modules

**Documentation:**
- [Validation API README](crates/validation-api/README.md)
- [Integration Guide](crates/validation-api/INTEGRATION.md)
- [Test Coverage Report](crates/ccx-solver/TEST_COVERAGE.md)
- [Solver Status](crates/ccx-solver/SOLVER_STATUS.md)

**API Endpoints:**
```bash
# Get dashboard statistics
curl http://localhost:8000/api/stats/dashboard | jq

# List test modules
curl http://localhost:8000/api/modules | jq

# Get validation results
curl http://localhost:8000/api/validation-results | jq
```

**GitHub Actions:**
The validation report is automatically generated on every push and PR. See `.github/workflows/validation-report.yml`.

## Contributing

See `DEVELOPMENT_PLAN.md` for the overall roadmap and phase gates.

When porting new functions:
1. Read `crates/ccx-solver/PORTING.md` for guidelines
2. Port function with full tests and documentation
3. Add to `PORTED_UNITS` in `src/lib.rs`
4. Verify migration report updates correctly
5. Ensure all tests pass

## License

CalculiX is licensed under GPLv2. See individual source files for copyright information.
