# calculix
Calculix

## Testing

- Fast TDD loop: `./scripts/tdd.sh`
- Full matrix: `CALCULIX_RUN_FULL=1 python3 -m pytest --full-matrix`
- Coverage workflow: see `TESTING.md`

## Nastran Reader

Use `pyNastran`-based reader for Nastran analysis decks:

- `uv run python -m calculix_migration_tooling.nastran_reader path/to/model.bdf --json`
