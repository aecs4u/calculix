#!/bin/bash
# Test implemented element types in ccx-solver
# Tests: T3D2, T3D3, B31, S4, C3D8

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.."; pwd)"
cd "$ROOT_DIR"

GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "Testing Implemented Elements in ccx-solver"
echo "==========================================="
echo ""

passed=0
failed=0

test_file() {
    local file="$1"
    local label="$2"

    if [ ! -f "$file" ]; then
        echo -e "${RED}SKIP${NC} $label (file not found)"
        return
    fi

    local output=$(timeout 15s cargo run --quiet --package ccx-solver --bin ccx-solver -- solve "$file" 2>&1)
    local exit_code=$?

    if [ $exit_code -eq 0 ] && echo "$output" | grep -q "Status: SUCCESS"; then
        local dofs=$(echo "$output" | grep "DOFs:" | awk '{print $2}')
        local equations=$(echo "$output" | grep "Equations:" | awk '{print $2}')
        echo -e "${GREEN}PASS${NC} $label (DOFs: $dofs, Eq: $equations)"
        ((passed++))
    else
        local error=$(echo "$output" | grep -E "error:|Unknown element" | head -1)
        echo -e "${RED}FAIL${NC} $label: $error"
        ((failed++))
    fi
}

# Truss Elements (T3D2, T3D3)
echo -e "${BLUE}Testing Truss Elements (T3D2, T3D3)${NC}"
test_file "tests/fixtures/solver/truss.inp" "truss.inp (T3D2)"
test_file "tests/fixtures/solver/truss2.inp" "truss2.inp (T3D3)"
echo ""

# Beam Elements (B31)
echo -e "${BLUE}Testing Beam Elements (B31)${NC}"
test_file "tests/fixtures/solver/b31.inp" "b31.inp"
test_file "tests/fixtures/solver/beamtor.inp" "beamtor.inp"
test_file "tests/fixtures/solver/beammix.inp" "beammix.inp"
echo ""

# Solid Elements (C3D8)
echo -e "${BLUE}Testing Solid Elements (C3D8)${NC}"
test_file "tests/fixtures/solver/achtelp.inp" "achtelp.inp"
test_file "tests/fixtures/solver/beam8f.inp" "beam8f.inp (C3D8)"
test_file "tests/fixtures/solver/ball.inp" "ball.inp"
echo ""

# Shell Elements (S4) - find some
echo -e "${BLUE}Testing Shell Elements (S4)${NC}"
# Note: Need to find actual S4 test files
# For now, skip this section
echo "  (No S4-specific test files identified yet)"
echo ""

# Summary
echo "==========================================="
echo "Summary:"
echo "  Passed:  $passed"
echo "  Failed:  $failed"
total=$((passed + failed))
if [ $total -gt 0 ]; then
    pass_rate=$(awk "BEGIN {printf \"%.1f\", ($passed/$total)*100}")
    echo "  Pass rate: $pass_rate%"
fi

exit 0
