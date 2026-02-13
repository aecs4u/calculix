# Quick Start Guide - Validation API

Get up and running with the CalculiX Rust Solver validation tracking system in 5 minutes.

## Option 1: Static HTML Report (No Dependencies)

**Perfect for:** Quick validation checks, sharing reports, CI/CD

```bash
# From project root
cd webapp

# Export test results
python3 scripts/export_test_results.py

# Generate HTML report
python3 scripts/generate_html_report.py

# Open in browser
open validation_report.html  # macOS
xdg-open validation_report.html  # Linux
```

**What you get:**
- Beautiful standalone HTML page
- Test statistics and KPIs
- Module breakdown with pass rates
- Validation results with error metrics
- No server or dependencies needed

## Option 2: Using Makefile (Recommended)

```bash
cd webapp

# One-command report generation
make quick-report

# View in browser
make view-report
```

## Option 3: Full Interactive API (Advanced)

**Perfect for:** Development, real-time monitoring, API integration

### Prerequisites
- Python 3.10+
- pip or uv

### Installation

```bash
cd webapp

# Install dependencies
pip install -e .
# or
uv pip install -e .
```

### Run Server

```bash
# Initialize database and start server
./run.sh

# Or manually
make run-api
```

Visit:
- **Dashboard**: http://localhost:8000
- **API Docs**: http://localhost:8000/docs
- **Test Modules**: http://localhost:8000/modules
- **Examples**: http://localhost:8000/examples

## From Project Root

Use the convenience script:

```bash
# From calculix/ directory
./scripts/generate-validation-report.sh
```

This will:
1. Run all tests
2. Export results
3. Generate HTML report
4. Show summary

## What's Included

### Static Report
- ✅ Test statistics (193 tests, 100% passing)
- ✅ Module breakdown (10 modules)
- ✅ Example validations (2 examples)
- ✅ KPI cards (pass rate, LOC, timing)
- ✅ Responsive design
- ✅ No dependencies

### Full API
- ✅ Everything in static report
- ✅ REST API endpoints
- ✅ Database persistence
- ✅ Historical tracking
- ✅ Real-time updates
- ✅ Interactive dashboard

## Example Workflows

### Daily Development

```bash
# After making changes
cargo test --workspace
cd webapp
make quick-report
```

### Code Review

```bash
# Generate report for PR
./scripts/generate-validation-report.sh
# Share validation_report.html
```

### CI/CD Integration

```bash
# In GitHub Actions
- run: cargo test --workspace
- run: python3 webapp/scripts/export_test_results.py
- run: python3 webapp/scripts/generate_html_report.py
- uses: actions/upload-artifact@v3
  with:
    name: validation-report
    path: webapp/validation_report.html
```

## API Usage Examples

### Get Dashboard Stats

```bash
curl http://localhost:8000/api/stats/dashboard | jq
```

### List Test Modules

```bash
curl http://localhost:8000/api/modules | jq
```

### Get Validation Results

```bash
curl http://localhost:8000/api/validation-results | jq
```

### Record Test Run

```bash
curl -X POST http://localhost:8000/api/test-runs \
  -H "Content-Type: application/json" \
  -d '{
    "test_case_id": 1,
    "passed": true,
    "execution_time_ms": 1.5,
    "git_commit": "abc1234"
  }'
```

## Troubleshooting

### "ModuleNotFoundError" when running API

Install dependencies:
```bash
pip install -e .
```

### Static report generation fails

Make sure you're in the right directory:
```bash
cd webapp
python3 scripts/export_test_results.py
```

### Database not found

Initialize it:
```bash
python3 scripts/populate_db.py
```

## Next Steps

- **Read the docs**: [README.md](README.md)
- **Integration guide**: [INTEGRATION.md](INTEGRATION.md)
- **Test coverage**: [../ccx-solver/TEST_COVERAGE.md](../ccx-solver/TEST_COVERAGE.md)
- **Solver status**: [../ccx-solver/SOLVER_STATUS.md](../ccx-solver/SOLVER_STATUS.md)

## Support

For issues:
1. Check the [README](README.md)
2. Review [INTEGRATION.md](INTEGRATION.md)
3. See the [main project README](../../README.md)

## Summary

**Fastest way to get started:**

```bash
cd webapp
make quick-report
```

**For full functionality:**

```bash
cd webapp
make install
./run.sh
```

**For project-wide report:**

```bash
./scripts/generate-validation-report.sh
```

That's it! You now have a comprehensive validation tracking system for the CalculiX Rust solver. 🚀
