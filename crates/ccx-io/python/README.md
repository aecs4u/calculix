# Python Integration for ccx-io

This directory contains Python modules for extended I/O capabilities in the CalculiX Rust project.

## Modules

### `meshio_wrapper.py` - Universal Mesh Format Conversion

Provides mesh format conversion and inspection via [meshio](https://github.com/nschloe/meshio).

**Supported Formats** (40+):
- **Read/Write**: VTK, VTU, XDMF, STL, OFF, OBJ, PLY, Gmsh (.msh), ANSYS (.ans), Abaqus (.inp), Nastran (.bdf/.nas), Exodus (.e), MED, Medit (.mesh), etc.

**Installation**:
```bash
pip install meshio numpy
# Or with all dependencies:
pip install -r requirements.txt
```

**Usage via ccx-cli**:

```bash
# Get mesh information
ccx-cli meshio-info mesh.vtu

# Convert between formats
ccx-cli meshio-convert input.stl output.vtu
ccx-cli meshio-convert model.msh result.vtk

# List supported formats
ccx-cli meshio-formats
```

**Direct Python Usage**:
```python
from meshio_wrapper import read_mesh, convert_mesh, get_mesh_info

# Read mesh info
info = read_mesh("model.vtu")
print(f"Points: {info['num_points']}, Cells: {info['num_cells']}")
print(f"Cell types: {info['cell_types']}")

# Convert format
convert_mesh("input.stl", "output.vtu")

# Detailed inspection
full_info = get_mesh_info("mesh.msh", verbose=True)
```

**CLI Tool**:
```bash
python meshio_wrapper.py info mesh.vtu --verbose
python meshio_wrapper.py convert input.stl output.vtu
python meshio_wrapper.py formats
python meshio_wrapper.py extract mesh.vtu displacement
```

### `nastran_io.py` - Nastran BDF/OP2 Support (Optional)

Nastran input/output via pyNastran (requires `nastran` feature in Cargo.toml).

**Installation**:
```bash
pip install pyNastran
```

## Integration with Rust

The Python modules are invoked from Rust via `std::process::Command`. See `crates/ccx-cli/src/main.rs` for implementation:

- `run_meshio_info()` → `meshio_wrapper.py info`
- `run_meshio_convert()` → `meshio_wrapper.py convert`
- `run_meshio_formats()` → `meshio_wrapper.py formats`

## Testing

```bash
# Test meshio wrapper
pytest python/test_meshio_wrapper.py

# Manual test
python python/meshio_wrapper.py info ../../../tests/fixtures/mesh/example.vtu
```

## Requirements

- **Python**: 3.7+
- **meshio**: 5.3.0+
- **numpy**: 1.20.0+ (meshio dependency)
- **pyNastran**: 1.4.0+ (optional, for Nastran support)

## Supported Mesh Formats

| Format | Extension | Read | Write |
|--------|-----------|------|-------|
| VTK Legacy | `.vtk` | ✓ | ✓ |
| VTK XML | `.vtu`, `.vtp`, `.vts` | ✓ | ✓ |
| XDMF | `.xdmf`, `.xmf` | ✓ | ✓ |
| STL | `.stl` | ✓ | ✓ |
| Gmsh | `.msh` | ✓ | ✓ |
| ANSYS | `.ans` | ✓ | ✓ |
| Abaqus | `.inp` | ✓ | ✓ |
| Nastran | `.bdf`, `.nas` | ✓ | ✓ |
| Exodus | `.e`, `.exo` | ✓ | ✓ |
| MED | `.med` | ✓ | ✓ |
| OFF | `.off` | ✓ | ✓ |
| OBJ | `.obj` | ✓ | ✓ |
| PLY | `.ply` | ✓ | ✓ |
| ... | +30 more | ... | ... |

Full list: https://github.com/nschloe/meshio#file-formats

## Architecture

```
ccx-cli (Rust)
    ↓ std::process::Command
python meshio_wrapper.py [command] [args]
    ↓ import meshio
meshio library (reads/writes formats)
    ↓ I/O
mesh files (VTK, STL, Gmsh, etc.)
```

## Error Handling

- **Missing Python**: `Python not found. Please install Python 3.7+`
- **Missing meshio**: `meshio is not installed. Install with: pip install meshio`
- **Unsupported format**: `ValueError: File format ... not supported`
- **File not found**: `FileNotFoundError: File not found: ...`

All errors are propagated to Rust with clear error messages.

## Performance

Meshio is optimized for large meshes (millions of elements). Typical performance:

- **Read**: 1M triangles in ~1 second
- **Write**: 1M triangles in ~1.5 seconds
- **Convert**: 1M triangles STL → VTU in ~2 seconds

## License

GPL-3.0 (matches CalculiX license)

## Contributing

See the main project CONTRIBUTING.md for guidelines.
