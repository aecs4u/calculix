# CalculiX Rust Solver Webapp

FastAPI webapp that now includes both:
- the ccx-cli runner UI/API (`fixtures`, `analyze`, `solve`, `validate`)
- the former `validation-api` dashboard and REST endpoints (`modules`, `examples`, KPI/test APIs, Nastran APIs)

## Setup

```bash
cd webapp
pip install -r requirements.txt
```

Build `ccx-cli` (for run/analyze/solve/validate pages):

```bash
cargo build --release --package ccx-cli
```

## Run

```bash
cd webapp
python -m uvicorn main:app --reload
```

Open `http://localhost:8000`.

## Main Routes

### Web pages
- `/` home
- `/fixtures` fixture browser
- `/run` run ccx-cli commands
- `/status` system status
- `/validation/dashboard` validation overview
- `/dashboard` extended validation dashboard
- `/modules`, `/examples` validation pages

### ccx-cli APIs
- `GET /health`
- `GET /version`
- `GET /api/fixtures`
- `POST /analyze/{fixture_name}`
- `POST /solve/{fixture_name}`
- `POST /validate`
- `POST /api/run-command`

### Validation APIs (merged from former validation-api)
- `GET/POST /api/modules`
- `GET/POST /api/test-cases`
- `GET/POST /api/test-runs`
- `GET/POST /api/examples`
- `GET/POST /api/validation-results`
- `GET/POST /api/kpis`
- `GET /api/kpis/latest`
- `GET /api/stats/dashboard`

### Nastran APIs
- `GET /api/nastran/status`
- `POST /api/nastran/convert/bdf-to-inp`
- `GET /api/nastran/download/{filename}`
- `POST /api/nastran/read/op2`
- `POST /api/nastran/extract/frequencies`

## Utilities

- `webapp/scripts/populate_db.py`
- `webapp/scripts/import_examples.py`
- `webapp/scripts/export_test_results.py`
- `webapp/scripts/generate_html_report.py`
- `webapp/Makefile`
- `webapp/run.sh`
