# ccx_2.23 Legacy Build Scripts

Scripts are provided to build legacy CalculiX CrunchiX executables using the original files in:

- `calculix_migration_tooling/ccx_2.23/src/Makefile`
- `calculix_migration_tooling/ccx_2.23/src/Makefile_MT`
- `calculix_migration_tooling/ccx_2.23/src/Makefile_i8`
- `calculix_migration_tooling/ccx_2.23/src/Makefile_MFront`

## Scripts

- `scripts/build_ccx223_executables.sh` (main orchestrator)
- `scripts/build_ccx223_default.sh`
- `scripts/build_ccx223_mt.sh`
- `scripts/build_ccx223_i8.sh`
- `scripts/build_ccx223_mfront.sh`

## Quick Usage

```bash
# Build default executable (ccx_2.23)
scripts/build_ccx223_default.sh

# Build i8 executable (ccx_2.23_i8)
scripts/build_ccx223_i8.sh

# Build all variants
scripts/build_ccx223_executables.sh --variant all --jobs 8
```

Build outputs are copied to:

- `dist/ccx_2.23/bin/`

## Notes

- Dependencies (SPOOLES, ARPACK, PaStiX, etc.) must be installed with paths expected by the legacy Makefiles, or those Makefiles must be adjusted.
- The legacy source directory is `ccx_2.23/src` (if you meant `scr`, this project uses `src`).
