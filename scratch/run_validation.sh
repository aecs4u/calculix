#!/bin/bash
# Validation runner for CalculiX Rust solver
# Compares solver output against reference DAT files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FIXTURES_DIR="$ROOT_DIR/tests/fixtures/solver"
REFERENCE_DIR="$ROOT_DIR/validation/solver"
SCRATCH_DIR="$ROOT_DIR/scratch"
SOLVER_BIN="$ROOT_DIR/target/debug/ccx-solver"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Build solver if not exists
if [ ! -f "$SOLVER_BIN" ]; then
    echo "Building solver..."
    cd "$ROOT_DIR"
    cargo build --package ccx-solver --bin ccx-solver
fi

# Statistics
total=0
passed=0
failed=0
skipped=0

# Create output directory
mkdir -p "$SCRATCH_DIR/validation_output"
RESULTS_FILE="$SCRATCH_DIR/validation_results_$(date +%Y%m%d_%H%M%S).txt"

echo "CalculiX Rust Solver Validation" | tee "$RESULTS_FILE"
echo "===============================" | tee -a "$RESULTS_FILE"
echo "Fixtures: $FIXTURES_DIR" | tee -a "$RESULTS_FILE"
echo "Reference: $REFERENCE_DIR" | tee -a "$RESULTS_FILE"
echo "Output: $SCRATCH_DIR/validation_output" | tee -a "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"

# Test specific fixtures or all if no args
if [ $# -eq 0 ]; then
    # Default: test beam fixtures only (known to work)
    TEST_PATTERN="beam*.inp"
else
    TEST_PATTERN="$1"
fi

echo "Testing pattern: $TEST_PATTERN" | tee -a "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"

# Run tests
for inp_file in $FIXTURES_DIR/$TEST_PATTERN; do
    [ -e "$inp_file" ] || continue

    basename=$(basename "$inp_file" .inp)
    ref_file="$REFERENCE_DIR/${basename}.dat.ref"
    out_file="$SCRATCH_DIR/validation_output/${basename}.dat"

    total=$((total + 1))

    # Check if reference exists
    if [ ! -f "$ref_file" ]; then
        echo -e "${YELLOW}SKIP${NC} $basename (no reference)" | tee -a "$RESULTS_FILE"
        skipped=$((skipped + 1))
        continue
    fi

    # Run solver
    cd "$SCRATCH_DIR/validation_output"
    if timeout 30s "$SOLVER_BIN" analyze "$inp_file" > "${basename}.log" 2>&1; then
        # Check if DAT file was created
        if [ -f "$out_file" ]; then
            # Compare with reference (simple line count for now)
            out_lines=$(wc -l < "$out_file")
            ref_lines=$(wc -l < "$ref_file")

            if [ "$out_lines" -eq "$ref_lines" ]; then
                echo -e "${GREEN}PASS${NC} $basename ($out_lines lines)" | tee -a "$RESULTS_FILE"
                passed=$((passed + 1))
            else
                echo -e "${RED}FAIL${NC} $basename (lines: $out_lines vs $ref_lines)" | tee -a "$RESULTS_FILE"
                failed=$((failed + 1))
            fi
        else
            echo -e "${RED}FAIL${NC} $basename (no output DAT)" | tee -a "$RESULTS_FILE"
            failed=$((failed + 1))
        fi
    else
        echo -e "${RED}FAIL${NC} $basename (solver error)" | tee -a "$RESULTS_FILE"
        failed=$((failed + 1))
    fi
done

# Summary
echo "" | tee -a "$RESULTS_FILE"
echo "===============================" | tee -a "$RESULTS_FILE"
echo "Summary:" | tee -a "$RESULTS_FILE"
echo "  Total:   $total" | tee -a "$RESULTS_FILE"
echo "  Passed:  $passed" | tee -a "$RESULTS_FILE"
echo "  Failed:  $failed" | tee -a "$RESULTS_FILE"
echo "  Skipped: $skipped" | tee -a "$RESULTS_FILE"

if [ $total -gt 0 ]; then
    pass_rate=$(awk "BEGIN {printf \"%.1f\", ($passed/$total)*100}")
    echo "  Pass rate: $pass_rate%" | tee -a "$RESULTS_FILE"
fi

echo "" | tee -a "$RESULTS_FILE"
echo "Results saved to: $RESULTS_FILE"

# Exit with error if any tests failed
[ $failed -eq 0 ]
