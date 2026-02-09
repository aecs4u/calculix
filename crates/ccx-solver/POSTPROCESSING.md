# CalculiX Postprocessing Module

This module provides stress and strain postprocessing capabilities for CalculiX .dat files, ported from the original [CCXStressReader](https://github.com/QuantenTunnel/CCXStressReader) Python tool by Henning Richter.

## Overview

The postprocessing module reads element variable output at integration points from CalculiX .dat files and computes:

- **Von Mises equivalent stress** (σ_v)
- **Total effective strain** (ε_eff)
- **Equivalent plastic strain** (PEEQ)
- **Statistical summaries** (min/max/mean)

Element variable output at integration points is generally more accurate than output extrapolated to nodes, particularly in stress analyses with non-linear material behaviour.

## Usage

### Command-Line Interface

The easiest way to use the postprocessing functionality is through the CLI:

```bash
ccx-cli postprocess results.dat
```

This will:
1. Read element variable output from `results.dat`
2. Compute Mises stress, effective strain, and PEEQ for each integration point
3. Calculate statistics (min/max/mean)
4. Write results to `results_IntPtOutput.txt`
5. Print summary statistics to console

### Example Output

```
Reading element variable output from: results.dat
Found 4 integration points

Statistics:
  Mises stress:       min=5.5245e1  max=9.5099e1  mean=7.3003e1
  Effective strain:   min=3.6830e-4  max=6.3399e-4  mean=4.8669e-4
  Plastic strain:     min=0.0000e0  max=2.0000e-4  mean=7.5000e-5
Results successfully written to file 'results_IntPtOutput.txt'
```

### Output File Format

The generated `*_IntPtOutput.txt` file contains:

```
     Elem.    Int.Pt.         MISES              EEQ             PEEQ
           1        1           6.5590e1        4.3726e-4         0.0000e0
           1        2           7.6079e1        5.0719e-4        1.0000e-4
           2        1           5.5245e1        3.6830e-4         0.0000e0

     Minimum                         5.5245e1        3.6830e-4         0.0000e0
     Maximum                         9.5099e1        6.3399e-4        2.0000e-4
     Mean (arith.)                   7.3003e1        4.8669e-4        7.5000e-5
```

## Programmatic API

### Basic Usage

```rust
use ccx_solver::{
    read_dat_file,
    process_integration_points,
    compute_statistics,
    write_results
};

// Read .dat file
let data = read_dat_file("results.dat")?;

// Process integration points
let results = process_integration_points(&data);

// Compute statistics
let stats = compute_statistics(&results);

// Write results to file
write_results("results.dat", &results, &stats)?;

println!("Mises stress: min={:.4e}, max={:.4e}, mean={:.4e}",
         stats.mises_min, stats.mises_max, stats.mises_mean);
```

### Computing Individual Values

```rust
use ccx_solver::{StressState, StrainState, compute_mises_stress, compute_effective_strain};

// Compute Mises stress from stress tensor
let stress = StressState {
    sxx: 100.0, syy: 50.0, szz: 30.0,
    sxy: 10.0, sxz: 5.0, syz: 3.0,
};
let mises = compute_mises_stress(&stress);

// Compute effective strain from strain tensor
let strain = StrainState {
    exx: 0.001, eyy: 0.0005, ezz: 0.0003,
    exy: 0.0001, exz: 0.00005, eyz: 0.00003,
};
let eeq = compute_effective_strain(&strain);
```

## Enabling Element Variable Output in CalculiX

To generate the required .dat file, add the following to your CalculiX .inp file:

```
*EL PRINT, ELSET=Eall
S, E, PEEQ
```

This activates element variable output to the .dat file for:
- `S` - Stress components (sxx, syy, szz, sxy, sxz, syz)
- `E` - Strain components (exx, eyy, ezz, exy, exz, eyz)
- `PEEQ` - Equivalent plastic strain

## Formulas

### Von Mises Equivalent Stress

```
σ_v = sqrt(0.5 * ((σ_xx - σ_yy)² + (σ_yy - σ_zz)² + (σ_zz - σ_xx)²)
           + 3 * (τ_xy² + τ_xz² + τ_yz²))
```

Where:
- σ_xx, σ_yy, σ_zz are normal stress components
- τ_xy, τ_xz, τ_yz are shear stress components

### Total Effective Strain

```
ε_eff = (2/3) * sqrt(0.5 * ((ε_xx - ε_yy)² + (ε_yy - ε_zz)² + (ε_zz - ε_xx)²)
                     + 3 * (γ_xy² + γ_xz² + γ_yz²))
```

Where:
- ε_xx, ε_yy, ε_zz are normal strain components
- γ_xy, γ_xz, γ_yz are shear strain components

## API Documentation

### Data Structures

- **`StressState`** - Stress tensor components at an integration point
- **`StrainState`** - Strain tensor components at an integration point
- **`IntegrationPointData`** - Raw data from .dat file for one integration point
- **`IntegrationPointResult`** - Computed results (Mises, EEQ, PEEQ) for one point
- **`ResultStatistics`** - Statistical summary (min/max/mean) for all points

### Functions

- **`read_dat_file(path)`** - Parse .dat file and extract element variable output
- **`compute_mises_stress(stress)`** - Compute von Mises stress from stress tensor
- **`compute_effective_strain(strain)`** - Compute effective strain from strain tensor
- **`process_integration_points(data)`** - Compute results for all integration points
- **`compute_statistics(results)`** - Calculate min/max/mean statistics
- **`write_results(path, results, stats)`** - Write formatted output file

## Testing

The module includes comprehensive tests:

- **10 unit tests** - Testing individual computation functions
- **5 integration tests** - Testing complete workflows
- **2 doctests** - Testing examples in documentation

Run tests with:

```bash
cargo test -p ccx-solver postprocess
cargo test -p ccx-solver --test postprocess_dat
```

## Compatibility

This Rust implementation provides identical functionality to the original CCXStressReader Python tool:

- Same input format (.dat files)
- Same output format (_IntPtOutput.txt files)
- Same computational formulas
- Same statistical measures

**Advantages over Python version:**
- ✅ **Type safety** - Compile-time error checking
- ✅ **Performance** - Native code execution
- ✅ **No runtime dependencies** - Standalone binary
- ✅ **Integration** - Direct integration with Rust solver
- ✅ **Memory safety** - No null pointer errors

## Examples

### Example 1: Basic Postprocessing

```bash
# Run CalculiX analysis
ccx example

# Postprocess results
ccx-cli postprocess example.dat

# View results
cat example_IntPtOutput.txt
```

### Example 2: Programmatic Access

```rust
use ccx_solver::{read_dat_file, process_integration_points};

fn analyze_stress(dat_file: &str) -> Result<f64, String> {
    let data = read_dat_file(dat_file)?;
    let results = process_integration_points(&data);

    // Find maximum Mises stress
    let max_stress = results.iter()
        .map(|r| r.mises)
        .fold(f64::NEG_INFINITY, f64::max);

    Ok(max_stress)
}
```

## License

This module is part of the CalculiX Rust solver project and follows the same license.

The original CCXStressReader Python tool was created by Henning Richter and released under the GNU Lesser General Public License v2.1.

## References

1. **CCXStressReader** - Original Python tool: https://github.com/QuantenTunnel/CCXStressReader
2. **CalculiX** - G. Dhondt, K. Wittig: http://www.calculix.de/
3. **Von Mises Yield Criterion** - https://en.wikipedia.org/wiki/Von_Mises_yield_criterion
4. **Effective Strain** - Theory of plasticity and strain hardening

## See Also

- [SOLVER_STATUS.md](SOLVER_STATUS.md) - Current solver capabilities
- [TEST_COVERAGE.md](TEST_COVERAGE.md) - Comprehensive test report
- [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md) - Future plans
