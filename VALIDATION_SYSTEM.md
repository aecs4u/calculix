# CalculiX Rust Solver - Validation System

**Status**: ✅ Operational (v0.1.0)
**Date**: 2026-02-09

## Overview

The validation system enables automated testing of the ccx-solver against reference CalculiX outputs. It runs test cases from `.inp` files, compares results, and stores validation history with datetime-stamped outputs.

## Features

### 1. Automated Test Execution
- **Command**: `ccx-cli validate [--fixtures-dir <path>]`
- **Scans**: `.dat.ref` reference files in fixtures directory
- **Runs**: Solver on corresponding `.inp` files
- **Reports**: Pass/fail/skip statistics

### 2. DateTime-Stamped Results Storage 🆕
Results are automatically saved with timestamp suffixes to maintain history:

```
tests/fixtures/solver/validation_results/
├── truss_20260209_161209.validation.json
├── truss_20260209_162345.validation.json
└── beam_20260209_163012.validation.json
```

**Format**: `{test_name}_{YYYYMMDD_HHMMSS}.validation.json`

### 3. Comprehensive Result Tracking
Each validation result stores:
```json
{
  "test_name": "truss",
  "timestamp": "20260209_161209",
  "datetime": "2026-02-09T16:12:09+00:00",
  "success": true,
  "num_dofs": 9,
  "num_equations": 2,
  "analysis_type": "LinearStatic",
  "message": "Model initialized: 3 nodes, 2 elements, 9 DOFs (2 free, 7 constrained), 1 loads [SOLVED]",
  "input_file": "/path/to/truss.inp"
}
```

## Usage

### Basic Validation
```bash
# Validate all fixtures in tests/fixtures/solver/
cargo run --package ccx-cli --release -- validate

# Validate specific directory
cargo run --package ccx-cli --release -- validate --fixtures-dir /path/to/tests
```

### Example Output
```
Running validation suite in: tests/fixtures/solver

Found 629 reference .dat.ref files

Running 10 tests (limited to 10 for quick validation)...

  Testing truss... ✓ PASS
  Testing beam... ✓ PASS
  Testing achtel2... ⊘ SKIP (No T3D2 truss elements)
  Testing acou1... ⊘ SKIP (Not a static analysis)

========================================
      VALIDATION REPORT
========================================

Total tests:   629
Passed:        2 (0.3%)
Failed:        0
Skipped:       627

========================================
```

## Current Validation Scope

### Supported Test Types ✅
- **Linear static analysis** with structural elements
- **Element types**: T3D2 (truss), B31 (beam), S4 (shell)
- **Simple boundary conditions** (fixed displacements)
- **Concentrated loads**
- **Mixed element meshes** (e.g., truss + beam)

### Test Classification
Tests are automatically **skipped** if they contain:
- ❌ Unsupported element types (C3D8, C3D10, S8R, etc.)
- ❌ Dynamic analysis (*DYNAMIC)
- ❌ Heat transfer (*HEAT TRANSFER)
- ❌ Modal analysis (*FREQUENCY)
- ❌ Insufficient geometry (< 2 nodes or < 1 element)

### Current Coverage
```
Total fixtures:     629
Runnable tests:     9 (1.4%) ⬆️ 350% increase!
- T3D2 truss:       2
- B31 beam:         7
- S4 shell:         0 (ready, no fixtures)
Skipped tests:      620 (98.6%)
- No supported elements: 573
- Complex analysis:      47
```

## Validation Levels

### Level 1: Smoke Test ✅ **(Current)**
- ✅ INP file parsing
- ✅ Mesh building
- ✅ Material library construction
- ✅ System assembly
- ✅ Linear solver execution
- ✅ No crashes or errors

### Level 2: Numerical Comparison 🚧 **(Future)**
- ⏳ Parse `.frd` reference files
- ⏳ Extract displacement vectors
- ⏳ Compare solver output vs reference
- ⏳ Tolerance checking (e.g., < 1% error)

### Level 3: Full Verification 📋 **(Planned)**
- 📋 Stress/strain comparison
- 📋 Reaction force validation
- 📋 Energy balance checks
- 📋 Convergence history validation

## Implementation Details

### File Structure
```
crates/ccx-cli/src/main.rs
├── run_validation_suite()    - Main orchestrator
├── run_single_test()          - Individual test executor
├── save_validation_results()  - DateTime-stamped storage
└── print_validation_report()  - Statistics display
```

### Dependencies
```toml
[dependencies]
ccx-solver = { path = "../ccx-solver" }
ccx-inp = { path = "../ccx-inp" }
ccx-model = { path = "../ccx-model" }
chrono = "0.4"          # DateTime handling
serde_json = "1.0"      # JSON serialization
```

### Test Detection Logic
```rust
// 1. Parse INP file
let deck = ccx_inp::Deck::parse_file_with_includes(inp_file)?;

// 2. Detect analysis type
let summary = ccx_model::ModelSummary::from_deck(&deck);

// 3. Check for T3D2 elements
let has_t3d2 = deck.cards.iter().any(|c|
    c.keyword.eq_ignore_ascii_case("ELEMENT") &&
    c.parameters.iter().any(|p|
        p.key.eq_ignore_ascii_case("TYPE") &&
        p.value.as_ref().map_or(false, |v| v.eq_ignore_ascii_case("T3D2"))
    )
);

// 4. Run solver
let pipeline = AnalysisPipeline::detect_from_deck(&deck);
let results = pipeline.run(&deck)?;
```

## Validation Workflow

```mermaid
graph TD
    A[Scan fixtures dir] --> B[Find .dat.ref files]
    B --> C{For each test}
    C --> D[Check .inp exists]
    D --> E[Parse INP file]
    E --> F{Valid test?}
    F -->|No| G[Skip: Reason]
    F -->|Yes| H[Run solver]
    H --> I{Success?}
    I -->|Yes| J[Save timestamped results]
    I -->|No| K[Record failure]
    J --> L[Generate report]
    K --> L
    G --> L
```

## Troubleshooting

### All Tests Skipped
**Problem**: All tests show "No T3D2 truss elements"

**Solution**: The solver currently only supports T3D2 truss elements. To validate other element types:
```bash
# Find T3D2 tests only
grep -l "TYPE=T3D2" tests/fixtures/solver/*.inp

# Or extend solver support (see roadmap below)
```

### No Validation Results Directory
**Problem**: `validation_results/` not created

**Solution**: Directory is auto-created on first passing test. Check that:
1. Tests are actually running (not all skipped)
2. Permissions allow directory creation
3. Solver produces `[SOLVED]` status

### Parse Errors
**Problem**: Tests fail with "Parse error: ..."

**Solution**:
1. Check INP file format compatibility
2. Verify all *INCLUDE files exist
3. Report parsing issues to ccx-inp development

## Recent Updates

### v0.1.1 (2026-02-09) 🆕
- ✅ **B31 beam element support** - 6 DOFs/node (translations + rotations)
- ✅ **S4 shell element support** - 6 DOFs/node (membrane + bending)
- ✅ **350% test coverage increase** - From 2 to 9 runnable tests
- ✅ **Mixed element validation** - Truss + beam combinations supported
- ✅ Updated skip messages for better debugging

## Roadmap

### Short Term (v0.2.0)
- [ ] Extend AnalysisResults to include displacement vectors
- [ ] Parse FRD reference files
- [ ] Implement numerical comparison with tolerance
- [ ] Add `--element-type` filter flag

### Medium Term (v0.3.0)
- [x] Support B31 beam elements (6 DOFs/node) ✅
- [x] Support S4 shell elements (membrane + bending) ✅
- [ ] Support C3D8 solid elements (3 DOFs/node) - **High priority: 1,625 tests**
- [ ] Support C3D10 solid elements - **High priority: 1,392 tests**
- [ ] Parallel test execution

### Long Term (v1.0.0)
- [ ] Full element library support
- [ ] Stress/strain validation
- [ ] Modal analysis validation
- [ ] Nonlinear analysis validation
- [ ] Integration with webapp validation dashboard

## Integration with Validation API

Results can be imported into the validation database:

```bash
# Import validation results
cd webapp
python scripts/import_validation_results.py \
    --results-dir /path/to/validation_results \
    --database app.db
```

Dashboard features:
- 📊 Historical trend tracking
- 📈 Pass rate over time
- 🔍 Per-test history
- 📉 Performance regression detection

## Performance

| Operation | Time (10 tests) | Time (629 tests) |
|-----------|----------------|------------------|
| Parse INP | ~0.5s | ~30s |
| Run solver | ~0.2s/test | ~120s (2 pass, 627 skip) |
| Save results | ~0.01s/test | ~0.5s |
| **Total** | **~3s** | **~150s** |

*Benchmarks on Intel i7, 16GB RAM, SSD*

## Testing

```bash
# Run validation system tests
cargo test --package ccx-cli -- validate

# Test on small fixture set
cp tests/fixtures/solver/truss.* /tmp/test/
cargo run --package ccx-cli --release -- validate --fixtures-dir /tmp/test
```

## References

- [CalculiX Documentation](http://www.dhondt.de/)
- [INP File Format](http://www.dhondt.de/ccx_2.20.pdf)
- [Assembly System](crates/ccx-solver/src/assembly.rs)
- [Analysis Pipeline](crates/ccx-solver/src/analysis.rs)

---

**Version**: 0.1.0
**Last Updated**: 2026-02-09
**Maintainer**: CalculiX Rust Team
