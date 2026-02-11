# ccx-io

I/O utilities for CalculiX with Nastran format support via pyNastran.

## Features

- **CalculiX Output**: DAT/STA/FRD writers for CalculiX format
- **FRD Reader**: Parse CalculiX result files
- **VTK Export**: ParaView visualization support
- **Nastran BDF Reader** (optional): Read Nastran Bulk Data Files via pyNastran
- **Nastran OP2 Reader** (optional): Read binary output files for result validation
- **BDF → INP Converter** (optional): Convert Nastran models to CalculiX format
- **Postprocessing**: von Mises stress, principal stresses/strains

## Requirements

### Core Features
- Rust 1.70+

### Nastran Support (optional)
- Python 3.8+
- pyNastran: `pip install pyNastran`

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
# Core features only
ccx-io = { path = "crates/ccx-io" }

# With Nastran support
ccx-io = { path = "crates/ccx-io", features = ["nastran"] }
```

## Usage

### Reading BDF Files

```rust
#[cfg(feature = "nastran")]
use ccx_io::NastranReader;

let reader = NastranReader::new()?;
let bdf_data = reader.read_bdf("model.bdf")?;

println!("Nodes: {}", bdf_data.nodes.len());
println!("Elements: {}", bdf_data.elements.len());
```

### Converting BDF to INP

```rust
#[cfg(feature = "nastran")]
use ccx_io::{NastranReader, BdfToInpConverter};

let reader = NastranReader::new()?;
let bdf_data = reader.read_bdf("model.bdf")?;

let mut converter = BdfToInpConverter::new();
let inp_content = converter.convert(&bdf_data)?;

std::fs::write("model.inp", inp_content)?;
```

### Reading OP2 Results

```rust
#[cfg(feature = "nastran")]
use ccx_io::NastranReader;

let reader = NastranReader::new()?;
let op2_data = reader.read_op2("results.op2")?;

for (node_id, disp) in &op2_data.displacements {
    println!("Node {}: dx={}, dy={}, dz={}",
             node_id, disp.dx, disp.dy, disp.dz);
}
```

### FRD Output (Core Feature)

```rust
use ccx_io::{write_frd_stub, FrdFile};

// Write basic FRD header
write_frd_stub("results.frd", &mesh)?;

// Read FRD file
let frd = FrdFile::parse("results.frd")?;
```

## Architecture

The Nastran support uses a two-tier architecture:

1. **Python Layer** (`python/nastran_io.py`): Wraps pyNastran, provides JSON serialization
2. **Rust Layer** (`src/nastran.rs`): PyO3 interface, deserializes to Rust structs

This design keeps the heavy I/O work in Python (where pyNastran excels) while providing a clean Rust API.

## Supported Element Types

| Nastran | CalculiX | Description |
|---------|----------|-------------|
| CROD    | T3D2     | 2-node truss |
| CBAR    | B31      | 2-node beam |
| CQUAD4  | S4       | 4-node shell |
| CTRIA3  | S3       | 3-node shell |
| CHEXA   | C3D8     | 8-node solid |
| CTETRA  | C3D4     | 4-node solid |

## Testing

```bash
# Core features only
cargo test --package ccx-io

# With Nastran support (requires pyNastran)
cargo test --package ccx-io --features nastran
```

## Python Environment Setup

For Nastran support:

```bash
# Install pyNastran
pip install pyNastran

# Test installation
python crates/ccx-io/python/nastran_io.py
```

## License

GPL-3.0 (same as CalculiX)
