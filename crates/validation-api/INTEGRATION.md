# Integration with CalculiX Rust Solver

This document describes how the Validation API integrates with the main CalculiX Rust solver project.

## Overview

The Validation API provides:
- **Database storage** for test results and validation data
- **Web dashboard** for visualizing solver progress
- **REST API** for programmatic access
- **KPI tracking** over time
- **Historical trends** for test coverage and performance

## Quick Start

```bash
cd crates/validation-api

# Install dependencies (one-time)
pip install -e .

# Initialize and run
./run.sh
```

Visit http://localhost:8000 to see the dashboard.

## Data Flow

```
┌──────────────────┐
│  Rust Tests      │  cargo test --workspace
│  (193 tests)     │  └─> Test results
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  populate_db.py  │  Parse and store results
│                  │  └─> SQLite database
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  FastAPI Server  │  Web UI + REST API
│  localhost:8000  │  └─> Dashboard, Charts, KPIs
└──────────────────┘
```

## What Gets Tracked

### Test Modules (10 modules)
- elements (21 tests)
- assembly (10 tests)
- materials (13 tests)
- mesh_builder (9 tests)
- bc_builder (9 tests)
- boundary_conditions (7 tests)
- sets (6 tests)
- analysis (13 tests)
- mesh (9 tests)
- ported (46 tests)

### Examples (2 examples)
- simple_truss (2-node bar, analytical validation)
- three_bar_truss (triangular truss, equilibrium check)

### Validation Results
- Computed vs analytical comparisons
- Relative error tracking
- Pass/fail status
- Tolerance checking

### KPIs
- Total tests: 193
- Pass rate: 100%
- Lines of code: ~3,520
- Supported elements: T3D2
- Average test time: <1ms

## API Usage

### Get Current Statistics

```bash
curl http://localhost:8000/api/stats/dashboard | jq
```

Response:
```json
{
  "total_tests": 193,
  "passing_tests": 193,
  "failing_tests": 0,
  "pass_rate": 100.0,
  "total_examples": 2,
  "validated_examples": 2,
  "total_modules": 10,
  "avg_test_time_ms": 0.3,
  "supported_elements": ["T3D2"],
  "lines_of_code": 3520
}
```

### Record New Test Run

```bash
curl -X POST http://localhost:8000/api/test-runs \
  -H "Content-Type: application/json" \
  -d '{
    "test_case_id": 1,
    "passed": true,
    "execution_time_ms": 1.2,
    "git_commit": "abc1234"
  }'
```

### Record Validation Result

```bash
curl -X POST http://localhost:8000/api/validation-results \
  -H "Content-Type: application/json" \
  -d '{
    "example_id": 1,
    "metric_name": "displacement_x",
    "computed_value": 0.004762,
    "analytical_value": 0.004762,
    "relative_error": 0.0000001,
    "passed": true,
    "tolerance": 0.000001
  }'
```

## Updating Database After Tests

After running `cargo test`, update the database:

```bash
cd crates/validation-api
python scripts/populate_db.py
```

This will:
1. Count current tests
2. Record pass/fail status
3. Update KPIs
4. Add validation results
5. Track git commit

## Dashboard Features

### Main Dashboard (`/`)
- **Test Statistics**: Total tests, pass rate, module count
- **Recent Runs**: Last 10 test executions
- **Recent Validations**: Latest validation results
- **KPI Cards**: Key metrics at a glance
- **Implementation Status**: Progress by component

### Test Modules (`/modules`)
- Module-by-module breakdown
- Pass rates per module
- Test counts
- Status badges

### Examples (`/examples`)
- Example problem details
- Validation accuracy
- Error percentages
- Element type coverage

## CI/CD Integration

### GitHub Actions

Add to `.github/workflows/test.yml`:

```yaml
- name: Run tests
  run: cargo test --workspace

- name: Update validation database
  run: |
    cd crates/validation-api
    python scripts/populate_db.py

- name: Deploy dashboard
  run: |
    # Deploy to your hosting platform
    # or commit the database to track history
```

## Monitoring & Alerts

The API can be extended to:
- Send email alerts on test failures
- Detect performance regressions
- Track coverage trends
- Generate weekly reports

## File Locations

- Database: `crates/validation-api/validation_results.db`
- Templates: `crates/validation-api/app/templates/`
- Scripts: `crates/validation-api/scripts/`
- Configuration: `crates/validation-api/pyproject.toml`

## Development

### Add New Test Module

1. Run `populate_db.py` to create module
2. Add test cases via API:
   ```bash
   curl -X POST http://localhost:8000/api/test-cases \
     -H "Content-Type: application/json" \
     -d '{
       "module_id": 1,
       "name": "new_test",
       "description": "Test description",
       "test_type": "unit"
     }'
   ```

### Add New Example

```bash
curl -X POST http://localhost:8000/api/examples \
  -H "Content-Type: application/json" \
  -d '{
    "name": "cantilever_beam",
    "description": "Cantilever beam with end load",
    "element_type": "B31",
    "num_nodes": 10,
    "num_elements": 9,
    "num_dofs": 60
  }'
```

## Future Enhancements

1. **Real-time Updates**: WebSocket for live test results
2. **Charts**: Historical trends with Chart.js
3. **Regression Detection**: Automated performance alerts
4. **Coverage Reports**: Integration with tarpaulin/llvm-cov
5. **Multi-Project**: Support for multiple solver versions
6. **Authentication**: User management and access control

## Troubleshooting

### Database not found

```bash
cd crates/validation-api
python scripts/populate_db.py
```

### API won't start

Check dependencies:
```bash
pip install -e .
```

### Tests not updating

Verify the database path in `populate_db.py` matches your current directory.

## Links

- **API Docs**: http://localhost:8000/docs
- **Main Dashboard**: http://localhost:8000
- **Test Coverage**: [../../crates/ccx-solver/TEST_COVERAGE.md](../../crates/ccx-solver/TEST_COVERAGE.md)
- **Solver Status**: [../../crates/ccx-solver/SOLVER_STATUS.md](../../crates/ccx-solver/SOLVER_STATUS.md)
