#!/usr/bin/env bash
# Start the CalculiX Rust Solver Web Interface
#
# This script starts the FastAPI webapp for running ccx-cli commands
# and viewing validation results.

set -euo pipefail

# Get the repository root directory
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEBAPP_DIR="${REPO_ROOT}/webapp"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Parse command line arguments
PORT="${1:-8000}"
HOST="${2:-0.0.0.0}"
RELOAD="${3:-}"

# Check if webapp directory exists
if [[ ! -d "${WEBAPP_DIR}" ]]; then
    print_error "Webapp directory not found: ${WEBAPP_DIR}"
    exit 1
fi

# Check if main.py exists
if [[ ! -f "${WEBAPP_DIR}/main.py" ]]; then
    print_error "main.py not found in: ${WEBAPP_DIR}"
    exit 1
fi

# Check if ccx-cli binary exists
CCX_CLI="${REPO_ROOT}/target/release/ccx-cli"
if [[ ! -f "${CCX_CLI}" ]]; then
    print_warning "ccx-cli binary not found at: ${CCX_CLI}"
    print_status "You can build it with: cargo build --release --package ccx-cli"
fi

# Check for Python dependencies
if ! command -v python3 &> /dev/null && ! command -v python &> /dev/null; then
    print_error "Python 3 is required but not installed"
    exit 1
fi

PYTHON_CMD=$(command -v python3 || command -v python)

# Check if uvicorn is installed
if ! "${PYTHON_CMD}" -m uvicorn --version &> /dev/null; then
    print_error "uvicorn is not installed"
    print_status "Install dependencies with:"
    print_status "  cd ${WEBAPP_DIR}"
    print_status "  pip install -r requirements.txt"
    print_status "  # or with uv:"
    print_status "  uv pip install -r requirements.txt"
    exit 1
fi

# Check if FastAPI is installed
if ! "${PYTHON_CMD}" -c "import fastapi" 2> /dev/null; then
    print_error "FastAPI is not installed"
    print_status "Install dependencies with:"
    print_status "  cd ${WEBAPP_DIR}"
    print_status "  pip install -r requirements.txt"
    exit 1
fi

# Print configuration
echo "========================================="
echo " CalculiX Rust Solver Web Interface"
echo "========================================="
print_status "Webapp directory: ${WEBAPP_DIR}"
print_status "Host: ${HOST}"
print_status "Port: ${PORT}"
if [[ "${RELOAD}" == "--reload" ]]; then
    print_status "Auto-reload: enabled"
else
    print_status "Auto-reload: disabled"
fi
echo "========================================="
echo

# Change to webapp directory
cd "${WEBAPP_DIR}"

# Build the uvicorn command
UVICORN_CMD="${PYTHON_CMD} -m uvicorn main:app --host ${HOST} --port ${PORT}"

if [[ "${RELOAD}" == "--reload" ]]; then
    UVICORN_CMD="${UVICORN_CMD} --reload"
fi

# Start the webapp
print_success "Starting webapp..."
print_status "Access the webapp at: http://localhost:${PORT}"
print_status "API docs at: http://localhost:${PORT}/docs"
print_status "Press Ctrl+C to stop"
echo

# Execute uvicorn
exec ${UVICORN_CMD}
