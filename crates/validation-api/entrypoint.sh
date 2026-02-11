#!/bin/sh
set -eu

PORT="${PORT:-8080}"

# Seed the runtime sqlite file on first start when using the default Cloud Run path.
if [ "${DATABASE_URL:-}" = "sqlite:////tmp/validation_results.db" ]; then
  if [ ! -f /tmp/validation_results.db ] && [ -f /app/validation_results.db ]; then
    cp /app/validation_results.db /tmp/validation_results.db
  fi
fi

exec python -m uvicorn app.main:app --host 0.0.0.0 --port "${PORT}"
