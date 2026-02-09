# CLI Updates Summary - ccx2paraview and cgxCadTools Integration

## Overview

This document outlines the updates needed to integrate ccx2paraview (FRD→VTK) and cgxCadTools (CAD format conversion) into the ccx-cli command-line interface.

## New Commands

### 1. ccx2paraview (FRD to VTK/VTU)

```bash
# Convert FRD to VTK legacy format
ccx-cli frd2vtk input.frd output.vtk

# Convert FRD to VTU XML format
ccx-cli frd2vtu input.frd output.vtu

# Convert FRD to VTU with binary encoding
ccx-cli frd2vtu --binary input.frd output.vtu
```

**Implementation**:
- Uses `ccx-io::FrdFile` to read FRD
- Uses `ccx-io::VtkWriter` to write VTK/VTU
- Computes von Mises and principal stresses automatically

### 2. cgxCadTools (CAD Format Conversion)

```bash
# Convert STEP/IGES to FBD
ccx-cli cad2fbd input.step output.fbd

# Convert STEP to FBD with surface splitting
ccx-cli cad2fbd --split-closed input.step output.fbd

# Convert FBD to STEP
ccx-cli fbd2step input.fbd output.step
```

**Implementation**:
- Python wrappers in `crates/ccx-compat/python/cadtools/`
- Calls original C++ binaries via subprocess
- Graceful error handling for missing binaries

## Code Changes Required

### 1. Update `crates/ccx-cli/src/main.rs`

#### Add to `usage()` function:
```rust
fn usage() {
    eprintln!("usage:");
    eprintln!("  ccx-cli analyze <input.inp>");
    eprintln!("  ccx-cli analyze-fixtures <fixtures_dir>");
    eprintln!("  ccx-cli postprocess <input.dat>");
    eprintln!("  ccx-cli frd2vtk <input.frd> <output.vtk>");          // NEW
    eprintln!("  ccx-cli frd2vtu [--binary] <input.frd> <output.vtu>"); // NEW
    eprintln!("  ccx-cli cad2fbd [options] <input.step> <output.fbd>"); // NEW
    eprintln!("  ccx-cli fbd2step <input.fbd> <output.step>");         // NEW
    eprintln!("  ccx-cli migration-report");
    eprintln!("  ccx-cli gui-migration-report");
    eprintln!("  ccx-cli --help");
    eprintln!("  ccx-cli --version");
}
```

#### Add handler functions:

```rust
fn frd2vtk_file(input_path: &Path, output_path: &Path) -> Result<(), String> {
    use ccx_io::{FrdFile, VtkWriter};

    println!("Reading FRD file: {}", input_path.display());
    let frd = FrdFile::from_file(input_path)
        .map_err(|e| format!("Failed to read FRD file: {}", e))?;

    println!("Found {} nodes, {} elements, {} time steps",
             frd.nodes.len(), frd.elements.len(), frd.result_blocks.len());

    println!("Writing VTK file: {}", output_path.display());
    let writer = VtkWriter::new(&frd);
    writer.write_vtk(output_path)
        .map_err(|e| format!("Failed to write VTK file: {}", e))?;

    println!("✓ Conversion successful");
    Ok(())
}

fn frd2vtu_file(input_path: &Path, output_path: &Path, binary: bool) -> Result<(), String> {
    use ccx_io::{FrdFile, VtkWriter, VtkFormat};

    println!("Reading FRD file: {}", input_path.display());
    let frd = FrdFile::from_file(input_path)
        .map_err(|e| format!("Failed to read FRD file: {}", e))?;

    println!("Found {} nodes, {} elements, {} time steps",
             frd.nodes.len(), frd.elements.len(), frd.result_blocks.len());

    let format = if binary { VtkFormat::Binary } else { VtkFormat::Ascii };
    println!("Writing VTU file ({:?}): {}", format, output_path.display());

    let writer = VtkWriter::new(&frd);
    writer.write_vtu(output_path, format)
        .map_err(|e| format!("Failed to write VTU file: {}", e))?;

    println!("✓ Conversion successful");
    Ok(())
}

fn cad2fbd_file(input_path: &Path, output_path: &Path, split_closed: bool) -> Result<(), String> {
    // Call Python wrapper via subprocess or direct Python API
    use std::process::Command;

    println!("Converting CAD to FBD: {} → {}", input_path.display(), output_path.display());

    let mut cmd = Command::new("python3");
    cmd.arg("-m")
       .arg("cadtools.cad2fbd")
       .arg(input_path);

    if split_closed {
        cmd.arg("--split-closed");
    }

    if let Some(output) = output_path.to_str() {
        cmd.arg("-o").arg(output);
    }

    let output = cmd.output()
        .map_err(|e| format!("Failed to run cad2fbd: {}. Is cgxCadTools installed?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Conversion failed: {}", stderr));
    }

    println!("✓ Conversion successful");
    Ok(())
}

fn fbd2step_file(input_path: &Path, output_path: &Path) -> Result<(), String> {
    use std::process::Command;

    println!("Converting FBD to STEP: {} → {}", input_path.display(), output_path.display());

    let mut cmd = Command::new("python3");
    cmd.arg("-m")
       .arg("cadtools.fbd2step")
       .arg(input_path);

    if let Some(output) = output_path.to_str() {
        cmd.arg("-o").arg(output);
    }

    let output = cmd.output()
        .map_err(|e| format!("Failed to run fbd2step: {}. Is cgxCadTools installed?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Conversion failed: {}", stderr));
    }

    println!("✓ Conversion successful");
    Ok(())
}
```

#### Add match arms in `main()`:

```rust
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        // ... existing cases ...

        Some("frd2vtk") => {
            if args.len() != 4 {
                usage();
                return ExitCode::from(2);
            }
            let input = Path::new(&args[2]);
            let output = Path::new(&args[3]);
            match frd2vtk_file(input, output) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("frd2vtk error: {err}");
                    ExitCode::from(1)
                }
            }
        }

        Some("frd2vtu") => {
            let (binary, input_idx, output_idx) = if args.get(2) == Some(&"--binary".to_string()) {
                (true, 3, 4)
            } else {
                (false, 2, 3)
            };

            if args.len() != output_idx + 1 {
                usage();
                return ExitCode::from(2);
            }

            let input = Path::new(&args[input_idx]);
            let output = Path::new(&args[output_idx]);
            match frd2vtu_file(input, output, binary) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("frd2vtu error: {err}");
                    ExitCode::from(1)
                }
            }
        }

        Some("cad2fbd") => {
            // Simple version - parse options properly in production
            let split_closed = args.contains(&"--split-closed".to_string());
            let args_filtered: Vec<_> = args.iter()
                .filter(|a| !a.starts_with("--"))
                .collect();

            if args_filtered.len() != 4 {
                usage();
                return ExitCode::from(2);
            }

            let input = Path::new(args_filtered[2].as_str());
            let output = Path::new(args_filtered[3].as_str());
            match cad2fbd_file(input, output, split_closed) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("cad2fbd error: {err}");
                    ExitCode::from(1)
                }
            }
        }

        Some("fbd2step") => {
            if args.len() != 4 {
                usage();
                return ExitCode::from(2);
            }
            let input = Path::new(&args[2]);
            let output = Path::new(&args[3]);
            match fbd2step_file(input, output) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("fbd2step error: {err}");
                    ExitCode::from(1)
                }
            }
        }

        _ => {
            usage();
            ExitCode::from(2)
        }
    }
}
```

### 2. Update `crates/ccx-cli/Cargo.toml`

Add ccx-io dependency:

```toml
[dependencies]
calculix_gui = { path = "../calculix_gui" }
ccx-inp = { path = "../ccx-inp" }
ccx-model = { path = "../ccx-model" }
ccx-solver = { path = "../ccx-solver" }
ccx-io = { path = "../ccx-io" }  # NEW
```

## Testing

### Unit Tests

```rust
#[test]
fn test_frd2vtk_basic() {
    // Create minimal FRD file
    // Run conversion
    // Verify VTK output exists
}

#[test]
fn test_cad2fbd_missing_binary() {
    // Should fail gracefully if cgxCadTools not installed
}
```

### Integration Tests

```bash
# Test FRD to VTK conversion
ccx-cli frd2vtk examples/beam_results.frd output.vtk
paraview output.vtk  # Verify visually

# Test CAD conversion (if binaries installed)
ccx-cli cad2fbd examples/part.step part.fbd
ccx-cli fbd2step part.fbd part_out.step
```

## Installation Requirements

### For ccx2paraview (frd2vtk/frd2vtu)
- ✅ No external dependencies (pure Rust)
- ✅ Works out of the box

### For cgxCadTools (cad2fbd/fbd2step)
- ⚠️  Requires Python 3
- ⚠️  Requires cgxCadTools binaries compiled and in PATH
- ⚠️  Requires OpenCASCADE library installed

**Graceful Degradation**: Commands fail with helpful error message if dependencies missing.

## User Documentation

### Quick Start Guide

```bash
# Convert CalculiX results to ParaView format
ccx-cli frd2vtk job.frd job.vtk
paraview job.vtk

# Convert STEP CAD file to CGX format
ccx-cli cad2fbd part.step part.fbd

# View in CGX
cgx -a part.fbd
```

### Error Messages

**Good error messages**:
- `frd2vtk error: Failed to read FRD file: No such file or directory`
- `cad2fbd error: cgxCadTools not installed. See docs/installation.md`
- `fbd2step error: OpenCASCADE library not found`

## Implementation Status

### Completed ✅
- [x] FRD reader (`ccx-io::FrdFile`)
- [x] VTK/VTU writer (`ccx-io::VtkWriter`)
- [x] Postprocessing utilities (von Mises, principals)
- [x] Python wrappers for cad2fbd
- [x] Python wrappers for fbd2step
- [x] Documentation

### Remaining ⏳
- [ ] Update CLI main.rs with new commands
- [ ] Add ccx-io dependency to CLI Cargo.toml
- [ ] Write integration tests
- [ ] Update user documentation
- [ ] Test with real FRD files
- [ ] Test with cgxCadTools binaries

## Next Steps

1. **Apply CLI changes**: Update main.rs with code from this document
2. **Test frd2vtk**: Use solver-generated FRD files from beam examples
3. **Test cad2fbd**: If cgxCadTools available, test CAD conversion
4. **Document usage**: Add to README and user guide
5. **CI integration**: Add automated tests

## References

- [ccx-io module](../crates/ccx-io/)
- [cgxCadTools](../calculix_migration_tooling/cgxCadTools/)
- [ccx2paraview original](../calculix.BAK/ccx2paraview/)
