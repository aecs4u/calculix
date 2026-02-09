# CalculiX Rust Solver Validation API

A FastAPI application for tracking and visualizing validation results for the CalculiX Rust solver.

## Features

- **Real-time Dashboard**: Overview of test coverage, validation results, and KPIs
- **Test Module Tracking**: Monitor test status across all modules
- **Example Validation**: Track validation against analytical solutions
- **KPI Tracking**: Historical performance metrics
- **REST API**: Programmatic access to all data
- **Database Storage**: Persistent storage of all test results

## Quick Start

### Prerequisites

- Python 3.10+
- SQLite3

### Installation

```bash
# Install dependencies
pip install -e .

# Or with uv (recommended)
uv pip install -e .
```

### Initialize Database

```bash
# Create database and populate with current test results
python scripts/populate_db.py
```

### Run the API

```bash
# Development server
uvicorn app.main:app --reload

# Production server
uvicorn app.main:app --host 0.0.0.0 --port 8000 --workers 4
```

Visit: http://localhost:8000

## API Endpoints

### Web Interface

- `GET /` - Dashboard
- `GET /modules` - Test modules overview
- `GET /examples` - Example problems overview
- `GET /docs` - Interactive API documentation

### REST API

#### Test Modules

- `GET /api/modules` - List all test modules
- `POST /api/modules` - Create new module

#### Test Cases

- `GET /api/test-cases?module_id={id}` - List test cases
- `POST /api/test-cases` - Create new test case

#### Test Runs

- `GET /api/test-runs?test_case_id={id}&limit={n}` - Get test history
- `POST /api/test-runs` - Record test run

#### Examples

- `GET /api/examples` - List example problems
- `POST /api/examples` - Create new example

#### Validation Results

- `GET /api/validation-results?example_id={id}&limit={n}` - Get validation history
- `POST /api/validation-results` - Record validation result

#### KPIs

- `GET /api/kpis?limit={n}` - Get KPI history
- `POST /api/kpis` - Record KPI snapshot
- `GET /api/kpis/latest` - Get latest KPI

#### Statistics

- `GET /api/stats/dashboard` - Get dashboard statistics

## Data Models

### TestModule

Test module/category (e.g., "elements", "assembly").

### TestCase

Individual test case with description and type (unit, integration, end-to-end).

### TestRun

Execution result for a test case (passed/failed, execution time, error message).

### Example

Example problem with input file, element type, and mesh info.

### ValidationResult

Validation comparing computed vs analytical results (metric, values, error, tolerance).

### KPI

Key Performance Indicator snapshot (total tests, pass rate, LOC, element types).

## Database Schema

```
test_modules
├─ test_cases
   └─ test_runs

examples
└─ validation_results

kpis
```

## Usage Examples

### Record Test Run

```python
import requests

response = requests.post('http://localhost:8000/api/test-runs', json={
    'test_case_id': 1,
    'passed': True,
    'execution_time_ms': 1.5,
    'git_commit': 'abc1234'
})
```

### Record Validation Result

```python
response = requests.post('http://localhost:8000/api/validation-results', json={
    'example_id': 1,
    'metric_name': 'displacement_x',
    'computed_value': 0.004762,
    'analytical_value': 0.004762,
    'relative_error': 0.0000001,
    'passed': True,
    'tolerance': 0.000001,
    'git_commit': 'abc1234'
})
```

### Get Dashboard Stats

```bash
curl http://localhost:8000/api/stats/dashboard | jq
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
- name: Run tests and update validation database
  run: |
    cargo test --workspace --no-fail-fast -- --format json > test-results.json
    python scripts/update_from_cargo_test.py test-results.json

- name: Post results to validation API
  run: |
    curl -X POST http://validation-api/api/test-runs \
      -H "Content-Type: application/json" \
      -d @test-results-payload.json
```

## KPIs Tracked

| KPI | Description |
|-----|-------------|
| Total Tests | Number of test cases |
| Passing Tests | Tests passing in latest run |
| Test Coverage | Percentage of passing tests |
| Element Types | Number of supported element types |
| Lines of Code | Total Rust LOC in ccx-solver |
| Avg Test Time | Mean execution time |

## Dashboard Screenshots

### Overview
- Test pass rate with trend
- Module statistics
- Recent test runs
- Recent validations

### Test Modules
- Per-module pass rates
- Test counts
- Status badges

### Examples
- Validation accuracy
- Error percentages
- Element type coverage

## Development

### Run Tests

```bash
pytest tests/
```

### Code Quality

```bash
# Format
black app/

# Lint
ruff app/
```

### Database Migrations

```bash
# Create migration
alembic revision --autogenerate -m "description"

# Apply migrations
alembic upgrade head
```

## File Structure

```
validation-api/
├── app/
│   ├── __init__.py
│   ├── main.py           # FastAPI application
│   ├── database.py       # SQLAlchemy models
│   ├── schemas.py        # Pydantic schemas
│   └── templates/        # Jinja2 HTML templates
│       ├── base.html
│       ├── dashboard.html
│       ├── modules.html
│       └── examples.html
├── scripts/
│   ├── populate_db.py    # Initialize database
│   └── update_from_cargo_test.py  # Parse cargo test output
├── pyproject.toml
├── README.md
└── validation_results.db # SQLite database
```

## Architecture

```
┌─────────────┐
│ Rust Solver │
│ (cargo test)│
└──────┬──────┘
       │ Test Results
       ▼
┌─────────────┐
│  populate   │
│  database   │
└──────┬──────┘
       │
       ▼
┌─────────────┐     ┌──────────────┐
│  SQLite DB  │◄────│  FastAPI     │
│             │     │  Application │
└─────────────┘     └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  Web UI      │
                    │  REST API    │
                    └──────────────┘
```

## Future Enhancements

- [ ] Real-time test execution monitoring
- [ ] Email alerts for test failures
- [ ] Performance regression detection
- [ ] Historical trend charts
- [ ] Export reports (PDF, CSV)
- [ ] Multi-project support
- [ ] User authentication
- [ ] Test retry mechanism
- [ ] Benchmark tracking

## Contributing

1. Add new endpoints in `app/main.py`
2. Define schemas in `app/schemas.py`
3. Update database models in `app/database.py`
4. Create templates in `app/templates/`
5. Run tests and update documentation

## License

Same as CalculiX Rust Solver project.

## Support

For issues or questions:
- Check the [API docs](http://localhost:8000/docs)
- Review the [main solver README](../../README.md)
- See [test coverage report](../../crates/ccx-solver/TEST_COVERAGE.md)
