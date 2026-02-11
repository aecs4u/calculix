#!/bin/bash
# Comprehensive validation of ccx-solver
# Tests fixtures and categorizes results by element type

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="scratch/validation_${TIMESTAMP}.txt"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "CalculiX Rust Solver - Comprehensive Validation" | tee "$RESULTS_FILE"
echo "================================================" | tee -a "$RESULTS_FILE"
echo "Timestamp: $TIMESTAMP" | tee -a "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"

# Categorize test cases by element type
declare -a TRUSS_TESTS=("truss.inp" "truss2.inp")
declare -a BEAM_TESTS=("beam8f.inp" "beamcr4.inp" "beamcr2.inp" "beamcr.inp" "beamd.inp" "beamb.inp" "beamf.inp")
declare -a SHELL_TESTS=("shell1.inp" "shell2.inp" "shell3.inp" "shell4.inp" "shell5.inp")
declare -a SOLID_TESTS=("c3d8.inp" "achtel.inp" "achtelp.inp")

passed=0
failed=0
skipped=0

test_fixture() {
    local name="$1"
    local inp_file="tests/fixtures/solver/$name"

    if [ ! -f "$inp_file" ]; then
        echo -e "  ${YELLOW}SKIP${NC} $name (not found)" | tee -a "$RESULTS_FILE"
        ((skipped++))
        return
    fi

    local output=$(timeout 10s cargo run --quiet --package ccx-solver --bin ccx-solver -- solve "$inp_file" 2>&1)
    local exit_code=$?

    if [ $exit_code -eq 0 ] && echo "$output" | grep -q "Status: SUCCESS"; then
        local dofs=$(echo "$output" | grep "DOFs:" | awk '{print $2}')
        local equations=$(echo "$output" | grep "Equations:" | awk '{print $2}')
        local solved=$(echo "$output" | grep -q "\[SOLVED\]" && echo "✓" || echo "")
        echo -e "  ${GREEN}PASS${NC} $name (DOFs: $dofs, Eq: $equations $solved)" | tee -a "$RESULTS_FILE"
        ((passed++))
    else
        local error=$(echo "$output" | grep -E "error:|Unknown element" | head -1 | cut -c1-60)
        echo -e "  ${RED}FAIL${NC} $name: $error" | tee -a "$RESULTS_FILE"
        ((failed++))
    fi
}

echo -e "${BLUE}Testing Truss Elements (T3D2)${NC}" | tee -a "$RESULTS_FILE"
for test in "${TRUSS_TESTS[@]}"; do
    test_fixture "$test"
done
echo "" | tee -a "$RESULTS_FILE"

echo -e "${BLUE}Testing Beam Elements (B31)${NC}" | tee -a "$RESULTS_FILE"
for test in "${BEAM_TESTS[@]}"; do
    test_fixture "$test"
done
echo "" | tee -a "$RESULTS_FILE"

echo -e "${BLUE}Testing Shell Elements (S4)${NC}" | tee -a "$RESULTS_FILE"
for test in "${SHELL_TESTS[@]}"; do
    test_fixture "$test"
done
echo "" | tee -a "$RESULTS_FILE"

echo -e "${BLUE}Testing Solid Elements (C3D8)${NC}" | tee -a "$RESULTS_FILE"
for test in "${SOLID_TESTS[@]}"; do
    test_fixture "$test"
done
echo "" | tee -a "$RESULTS_FILE"

# Summary
echo "================================================" | tee -a "$RESULTS_FILE"
echo "Summary:" | tee -a "$RESULTS_FILE"
echo "  Passed:  $passed" | tee -a "$RESULTS_FILE"
echo "  Failed:  $failed" | tee -a "$RESULTS_FILE"
echo "  Skipped: $skipped" | tee -a "$RESULTS_FILE"
total=$((passed + failed))
if [ $total -gt 0 ]; then
    pass_rate=$(awk "BEGIN {printf \"%.1f\", ($passed/$total)*100}")
    echo "  Pass rate: $pass_rate%" | tee -a "$RESULTS_FILE"
fi
echo "" | tee -a "$RESULTS_FILE"
echo "Full results saved to: $RESULTS_FILE"

exit 0
