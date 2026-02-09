# calculix_cli documentation

`ccx-cli` is the workspace command-line entrypoint for:

- INP deck analysis
- Solver migration progress reporting
- GUI migration progress reporting

## Build and run

From repository root:

```bash
cargo run -p ccx-cli -- --help
```

Or build first:

```bash
cargo build -p ccx-cli
./target/debug/ccx-cli --help
```

## Command reference

### `analyze <input.inp>`

Parses a single deck (including `*INCLUDE` expansion) and prints a compact model summary.

```bash
cargo run -p ccx-cli -- analyze tests/fixtures/solver/ax6.inp
```

Example output:

```text
total_cards: 12
total_data_lines: 124
node_rows: 85
element_rows: 32
material_defs: 1
has_step: true
has_static: true
has_dynamic: false
has_frequency: false
has_heat_transfer: false
unique_keywords: 12
```

### `analyze-fixtures <fixtures_dir>`

Recursively scans a directory for `.inp` files and validates parseability.

```bash
cargo run -p ccx-cli -- analyze-fixtures tests/fixtures/solver
```

Example output:

```text
fixtures_root: tests/fixtures/solver
total_inp: 638
parse_ok: 638
parse_failed: 0
```

If any deck fails, each failure is reported as:

```text
parse_error: <path>: line <n>: <message>
```

### `migration-report`

Prints migration status for legacy solver source units.

```bash
cargo run -p ccx-cli -- migration-report
```

Example output:

```text
legacy_units_total: 1199
ported_units: 5
superseded_fortran_units: 986
pending_units: 213
language_C: 185
language_Fortran: 986
language_Header: 9
language_Other: 19
ported_list: compare.c, strcmp1.c, superseded/bsort.f, superseded/cident.f, superseded/insertsortd.f
```

### `gui-migration-report`

Prints migration status for the legacy CGX GUI source units.

```bash
cargo run -p ccx-cli -- gui-migration-report
```

### `--help` and `--version`

```bash
cargo run -p ccx-cli -- --help
cargo run -p ccx-cli -- --version
```
