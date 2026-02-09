# CalculiX Validation API - Makefile

.PHONY: help install export-results generate-report view-report populate-db run-api clean

help:
	@echo "CalculiX Rust Solver Validation API"
	@echo "===================================="
	@echo ""
	@echo "Available targets:"
	@echo "  make install          - Install Python dependencies"
	@echo "  make export-results   - Export test results to JSON"
	@echo "  make generate-report  - Generate standalone HTML report"
	@echo "  make view-report      - Open HTML report in browser"
	@echo "  make populate-db      - Initialize database (requires dependencies)"
	@echo "  make run-api          - Start FastAPI server (requires dependencies)"
	@echo "  make quick-report     - Quick: export + generate + view"
	@echo "  make clean            - Remove generated files"
	@echo ""

install:
	@echo "📦 Installing dependencies..."
	pip install -e .
	@echo "✅ Installation complete"

export-results:
	@echo "🧪 Exporting test results..."
	python3 scripts/export_test_results.py

generate-report:
	@echo "📊 Generating HTML report..."
	python3 scripts/generate_html_report.py

view-report: generate-report
	@echo "🌐 Opening report in browser..."
	@xdg-open validation_report.html 2>/dev/null || open validation_report.html 2>/dev/null || echo "Open validation_report.html manually"

quick-report: export-results generate-report
	@echo "✅ Report ready: validation_report.html"
	@echo ""
	@echo "   To view: make view-report"
	@echo "   Or open: file://$(PWD)/validation_report.html"

populate-db:
	@echo "🗄️  Populating database..."
	python3 scripts/populate_db.py

run-api: populate-db
	@echo "🚀 Starting FastAPI server..."
	@echo "   Dashboard: http://localhost:8000"
	@echo "   API Docs:  http://localhost:8000/docs"
	python3 -m uvicorn app.main:app --reload

clean:
	@echo "🧹 Cleaning generated files..."
	rm -f test_results.json validation_report.html validation_results.db
	rm -rf __pycache__ app/__pycache__ scripts/__pycache__
	rm -rf *.egg-info
	@echo "✅ Clean complete"

# Development targets
test:
	@echo "🧪 Running tests..."
	pytest tests/ -v

format:
	@echo "🎨 Formatting code..."
	black app/ scripts/

lint:
	@echo "🔍 Linting code..."
	ruff app/ scripts/

check: format lint test
	@echo "✅ All checks passed"
