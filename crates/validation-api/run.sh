#!/bin/bash
# Run the CalculiX Validation API

set -e

cd "$(dirname "$0")"

echo "🦀 CalculiX Rust Solver Validation API"
echo "========================================"
echo ""

# Check if database exists
if [ ! -f "validation_results.db" ]; then
    echo "📦 Database not found. Initializing..."
    python3 scripts/populate_db.py
    echo ""
fi

echo "🚀 Starting FastAPI server..."
echo "   Dashboard: http://localhost:8000"
echo "   API Docs:  http://localhost:8000/docs"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Run the server
python3 -m uvicorn app.main:app --reload --host 0.0.0.0 --port 8000
