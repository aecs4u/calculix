#!/bin/bash
# Quick validation of ccx-solver against test fixtures

set -e

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "CalculiX Rust Solver - Quick Validation"
echo "========================================"
echo ""

# Test fixtures known to work with current implementation
TEST_CASES=(
    "truss.inp"
    "beam8f.inp"
    "beamcr4.inp"
    "oneel.inp"
    "oneeltruss.inp"
)

passed=0
failed=0

for test_case in "${TEST_CASES[@]}"; do
    inp_file="tests/fixtures/solver/$test_case"

    if [ ! -f "$inp_file" ]; then
        echo -e "${YELLOW}SKIP${NC} $test_case (file not found)"
        continue
    fi

    echo -n "Testing $test_case ... "

    if timeout 10s cargo run --quiet --package ccx-solver --bin ccx-solver -- solve "$inp_file" > /dev/null 2>&1; then
        echo -e "${GREEN}PASS${NC}"
        ((passed++))
    else
        echo -e "${RED}FAIL${NC}"
        ((failed++))
    fi
done

echo ""
echo "Results: $passed passed, $failed failed"

[ $failed -eq 0 ]
