#!/bin/bash
# Run solver on fixtures and write output files to scratch/solver/

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

OUTPUT_DIR="scratch/solver"
mkdir -p "$OUTPUT_DIR"

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo "Running CalculiX Rust Solver - Writing Output Files"
echo "===================================================="
echo "Output directory: $OUTPUT_DIR"
echo ""

# Test cases that should work
TEST_CASES=(
    "truss.inp"
    "beam8f.inp"
    "beamcr4.inp"
    "beamcr2.inp"
    "beamcr.inp"
    "beamd.inp"
    "beamb.inp"
    "beamf.inp"
    "shell1.inp"
    "shell2.inp"
    "shell4.inp"
    "achtelp.inp"
)

passed=0
failed=0

for test_case in "${TEST_CASES[@]}"; do
    inp_file="tests/fixtures/solver/$test_case"
    base_name="${test_case%.inp}"

    if [ ! -f "$inp_file" ]; then
        echo -e "${RED}SKIP${NC} $test_case (not found)"
        continue
    fi

    echo -n "Processing $test_case ... "

    # Copy INP file to output directory for reference
    cp "$inp_file" "$OUTPUT_DIR/"

    # Run solver (will be modified to write .dat files)
    if timeout 15s cargo run --quiet --package ccx-solver --bin ccx-solver -- solve "$inp_file" > "$OUTPUT_DIR/${base_name}.log" 2>&1; then
        # Check if solver succeeded
        if grep -q "Status: SUCCESS" "$OUTPUT_DIR/${base_name}.log"; then
            # Create a placeholder .out file for now (will be .dat later)
            cat > "$OUTPUT_DIR/${base_name}.out" <<EOF
CalculiX Rust Solver Output
===========================
Input: $test_case
Analysis: Linear Static

$(grep "DOFs:" "$OUTPUT_DIR/${base_name}.log")
$(grep "Equations:" "$OUTPUT_DIR/${base_name}.log")
$(grep "Message:" "$OUTPUT_DIR/${base_name}.log")

Status: Completed successfully
EOF
            echo -e "${GREEN}PASS${NC} (output written)"
            ((passed++))
        else
            echo -e "${RED}FAIL${NC} (solver error)"
            ((failed++))
        fi
    else
        echo -e "${RED}FAIL${NC} (timeout or error)"
        ((failed++))
    fi
done

echo ""
echo "===================================================="
echo "Summary:"
echo "  Processed: $passed files"
echo "  Failed: $failed files"
echo "  Output location: $OUTPUT_DIR/"
echo ""
echo "Generated files:"
ls -lh "$OUTPUT_DIR/" | grep -E "\.(inp|out|log)$" | wc -l
echo " files written to $OUTPUT_DIR/"

exit 0
