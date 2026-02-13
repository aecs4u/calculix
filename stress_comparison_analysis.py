#!/usr/bin/env python3
"""
Compare stress outputs between reference and our implementation.
"""

import re

def parse_stress_line(line):
    """Parse a stress output line."""
    # Skip empty lines
    if not line.strip():
        return None

    # Replace 'E' notation variants
    line = line.replace('E+', 'E').replace('E-', 'E-')

    parts = line.split()
    if len(parts) >= 8:
        try:
            return {
                'elem': int(parts[0]),
                'ip': int(parts[1]),
                'sxx': float(parts[2]),
                'syy': float(parts[3]),
                'szz': float(parts[4]),
                'sxy': float(parts[5]),
                'sxz': float(parts[6]),
                'syz': float(parts[7])
            }
        except (ValueError, IndexError):
            return None
    return None

def read_stresses(filename):
    """Read stresses from DAT file."""
    stresses = []
    with open(filename, 'r') as f:
        in_stress_section = False
        for line in f:
            if 'stresses (elem, integ.pnt.' in line:
                in_stress_section = True
                continue
            if in_stress_section:
                # Stop when we hit volume section
                if 'volume' in line.lower():
                    break
                # Skip empty lines but don't stop
                if not line.strip():
                    continue
                # Try to parse the stress line
                stress = parse_stress_line(line)
                if stress:
                    stresses.append(stress)
    return stresses

def main():
    ref_file = 'validation/solver/simplebeam.dat.ref'
    our_file = 'tests/fixtures/solver/simplebeam.dat'

    ref_stresses = read_stresses(ref_file)
    our_stresses = read_stresses(our_file)

    print("=" * 80)
    print("STRESS COMPARISON: Reference vs Our Implementation")
    print("=" * 80)
    print(f"\nTotal integration points: Ref={len(ref_stresses)}, Ours={len(our_stresses)}")

    # Compare first 10 points in detail
    print("\n" + "=" * 80)
    print("DETAILED COMPARISON (First 10 Integration Points)")
    print("=" * 80)

    for i in range(min(10, len(ref_stresses), len(our_stresses))):
        ref = ref_stresses[i]
        our = our_stresses[i]

        print(f"\n--- Integration Point {ref['ip']} ---")
        print(f"{'Component':<8} {'Reference':>12} {'Ours':>12} {'Ratio':>8} {'Diff':>12}")
        print("-" * 60)

        for comp in ['sxx', 'syy', 'szz', 'sxy', 'sxz', 'syz']:
            ref_val = ref[comp]
            our_val = our[comp]

            if abs(ref_val) > 1e-6:
                ratio = our_val / ref_val if abs(ref_val) > 1e-10 else 0.0
                diff = our_val - ref_val
                print(f"{comp:<8} {ref_val:>12.2f} {our_val:>12.2f} {ratio:>8.2f}× {diff:>12.2f}")
            else:
                print(f"{comp:<8} {ref_val:>12.2f} {our_val:>12.2f} {'N/A':>8} {our_val:>12.2f}")

    # Statistical analysis
    print("\n" + "=" * 80)
    print("STATISTICAL ANALYSIS (All 50 Points)")
    print("=" * 80)

    for comp in ['sxx', 'syy', 'szz', 'sxy', 'sxz', 'syz']:
        ref_vals = [s[comp] for s in ref_stresses]
        our_vals = [s[comp] for s in our_stresses[:len(ref_stresses)]]

        # Filter out near-zero reference values for ratio calculation
        ratios = [our/ref for ref, our in zip(ref_vals, our_vals) if abs(ref) > 1.0]

        if ratios:
            avg_ratio = sum(ratios) / len(ratios)
            min_ratio = min(ratios)
            max_ratio = max(ratios)

            print(f"\n{comp}:")
            print(f"  Ref range: [{min(ref_vals):.1f}, {max(ref_vals):.1f}]")
            print(f"  Our range: [{min(our_vals):.1f}, {max(our_vals):.1f}]")
            print(f"  Ratio: avg={avg_ratio:.3f}×, min={min_ratio:.3f}×, max={max_ratio:.3f}×")
        else:
            print(f"\n{comp}: All reference values near zero")

    # Key observations
    print("\n" + "=" * 80)
    print("KEY OBSERVATIONS")
    print("=" * 80)

    # Check for zero values where ref is non-zero
    print("\n1. Zero values in our output where reference is non-zero:")
    zero_count = {'sxy': 0, 'syz': 0}
    for i in range(len(our_stresses)):
        if abs(ref_stresses[i]['sxy']) > 1.0 and abs(our_stresses[i]['sxy']) < 0.01:
            zero_count['sxy'] += 1
        if abs(ref_stresses[i]['syz']) > 1.0 and abs(our_stresses[i]['syz']) < 0.01:
            zero_count['syz'] += 1

    print(f"   sxy: {zero_count['sxy']}/50 points are zero (ref non-zero)")
    print(f"   syz: {zero_count['syz']}/50 points are zero (ref non-zero)")

    # Check sign differences
    print("\n2. Sign differences:")
    sign_diff = {comp: 0 for comp in ['sxx', 'syy', 'szz', 'sxy', 'sxz', 'syz']}
    for i in range(len(our_stresses)):
        for comp in sign_diff.keys():
            ref_val = ref_stresses[i][comp]
            our_val = our_stresses[i][comp]
            if abs(ref_val) > 10 and abs(our_val) > 10:  # Only for significant values
                if (ref_val * our_val) < 0:  # Opposite signs
                    sign_diff[comp] += 1

    for comp, count in sign_diff.items():
        if count > 0:
            print(f"   {comp}: {count}/50 points have opposite signs")

if __name__ == '__main__':
    main()
