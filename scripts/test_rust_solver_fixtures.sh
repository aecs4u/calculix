#!/usr/bin/env bash
# Test Rust ccx-solver on multiple fixtures from tests/fixtures/solver/

set -euo pipefail

SOLVER_BIN="${1:-target/release/ccx-cli}"
FIXTURES_DIR="${2:-tests/fixtures/solver}"

if [[ ! -x "$SOLVER_BIN" ]]; then
    echo "Error: Solver binary not found or not executable: $SOLVER_BIN" >&2
    echo "Build with: cargo build --release --package ccx-cli" >&2
    exit 1
fi

if [[ ! -d "$FIXTURES_DIR" ]]; then
    echo "Error: Fixtures directory not found: $FIXTURES_DIR" >&2
    exit 1
fi

echo "========================================"
echo " Rust Solver Fixture Test Suite"
echo "========================================"
echo "Solver: $SOLVER_BIN"
echo "Fixtures: $FIXTURES_DIR"
echo "========================================"
echo

# Select a diverse set of test cases
test_cases=(
    "achtel2.inp"      # C3D8 hex elements
    "achtel9.inp"      # C3D8 with different BC
    "achtel29.inp"     # C3D8 variant
    "beam8b.inp"       # Beam elements
    "beam8p.inp"       # Beam with loads
)

passed=0
failed=0
skipped=0

for inp_file in "${test_cases[@]}"; do
    inp_path="$FIXTURES_DIR/$inp_file"

    if [[ ! -f "$inp_path" ]]; then
        echo "⊘ SKIP: $inp_file (file not found)"
        ((skipped++))
        continue
    fi

    echo -n "Testing $inp_file... "

    # Run analyze command (parsing only)
    if output=$("$SOLVER_BIN" analyze "$inp_path" 2>&1); then
        # Check if it reports structural analysis
        if echo "$output" | grep -q "has_static: true"; then
            echo "✓ PASS (static analysis)"
            ((passed++))
        elif echo "$output" | grep -q "has_frequency: true"; then
            echo "✓ PASS (modal analysis)"
            ((passed++))
        else
            echo "⊘ SKIP (unsupported analysis type)"
            ((skipped++))
        fi
    else
        echo "✗ FAIL"
        echo "  Error: $output"
        ((failed++))
    fi
done

echo
echo "========================================"
echo "           RESULTS SUMMARY"
echo "========================================"
echo "Total:   ${#test_cases[@]}"
echo "Passed:  $passed"
echo "Failed:  $failed"
echo "Skipped: $skipped"
echo "========================================"

if [[ $failed -gt 0 ]]; then
    exit 1
fi
