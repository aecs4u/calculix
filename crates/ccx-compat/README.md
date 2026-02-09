# ccx-compat: CalculiX Compatibility Layer

This crate provides compatibility wrappers for legacy CalculiX tools that haven't been ported to Rust.

## Supported Tools

### cgxCadTools
CAD format converters for CalculiX CGX (C4W format):

- **cad2fbd**: Convert STEP/IGES CAD files to CGX FBD format
- **fbd2step**: Convert CGX FBD format back to STEP

**Implementation**: Python/shell wrapper around original C++ binaries

**Dependencies**:
- OpenCASCADE Technology (6.9.1+)
- Original cgxCadTools binaries

## Integration Strategy

### Option 1: Binary Wrapper (Current)
- Keep original C++ tools as external binaries
- Provide Python wrappers for CLI integration
- Minimal porting effort, maximum compatibility

### Option 2: Rust Bindings (Future)
- Use `opencascade-sys` or similar Rust bindings
- Port C++ code to Rust with OCC bindings
- Better integration, no external dependencies

### Option 3: Pure Rust CAD (Long-term)
- Use pure Rust CAD libraries (truck, etc.)
- Full native implementation
- Currently not mature enough for production

## Current Implementation

We use **Option 1** for now with Python wrappers in `python/cadtools/`.

## Usage

```bash
# Convert STEP to FBD
ccx-cli cad2fbd input.step output.fbd

# Convert FBD to STEP
ccx-cli fbd2step input.fbd output.step
```

## Directory Structure

```
ccx-compat/
├── src/
│   └── lib.rs         # Rust interface (future)
├── python/
│   └── cadtools/
│       ├── __init__.py
│       ├── cad2fbd.py   # STEP/IGES → FBD converter wrapper
│       └── fbd2step.py  # FBD → STEP converter wrapper
└── README.md
```

## Installation

### Prerequisites

1. **OpenCASCADE Technology**
   ```bash
   # Ubuntu/Debian
   sudo apt-get install opencascade-dev

   # macOS
   brew install opencascade
   ```

2. **cgxCadTools binaries**
   ```bash
   # Build from source (included in calculix_migration_tooling/)
   cd calculix_migration_tooling/cgxCadTools/CadReader
   make

   # Install binaries to PATH
   sudo cp bin/cad2fbd /usr/local/bin/
   ```

### Python Package

```bash
# Install with uv
uv pip install -e crates/ccx-compat/python/
```

## Development

### Adding New CAD Tools

1. Create wrapper in `python/cadtools/`
2. Add CLI command in `ccx-cli`
3. Document usage in this README

### Testing

```bash
# Test Python wrappers
pytest crates/ccx-compat/python/tests/

# Test Rust interface (when available)
cargo test --package ccx-compat
```

## License

GNU General Public License v3.0 (to match cgxCadTools license)

## References

- [cgxCadTools Documentation](../../../calculix_migration_tooling/cgxCadTools/README)
- [OpenCASCADE Documentation](https://www.opencascade.com/doc/)
