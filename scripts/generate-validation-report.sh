#!/bin/bash
# Generate CalculiX Rust Solver Validation Report
# This script runs all tests and generates a comprehensive validation report

set -e

cd "$(dirname "$0")/.."

echo "🦀 CalculiX Rust Solver - Validation Report Generator"
echo "======================================================"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Must run from project root"
    exit 1
fi

# Step 1: Run tests
echo "📝 Step 1: Running all tests..."
echo "   This may take a moment..."
cargo test --workspace --quiet 2>&1 | tail -5
echo "   ✅ Tests completed"
echo ""

# Step 2: Export test results
echo "📊 Step 2: Exporting test results..."
cd crates/validation-api
python3 scripts/export_test_results.py
echo ""

# Step 3: Generate HTML report
echo "🌐 Step 3: Generating HTML report..."
python3 scripts/generate_html_report.py
cd ../..
echo ""

# Show summary
echo "✅ Validation report generated successfully!"
echo ""
echo "📄 Files created:"
echo "   - crates/validation-api/test_results.json    (Test data)"
echo "   - crates/validation-api/validation_report.html (HTML report)"
echo ""
echo "🌐 To view the report:"
echo "   Open: crates/validation-api/validation_report.html"
echo ""
echo "📊 For interactive dashboard:"
echo "   cd crates/validation-api"
echo "   make run-api"
echo "   Visit: http://localhost:8000"
echo ""
