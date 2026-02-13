#!/usr/bin/env python3
"""
Test beam coordinate transformation to understand the mapping.

Beam: from (0,0,0) → (0,0,5) → (0,0,10)
Beam axis: Z direction
Normal: (1,0,0) = X direction

Expected local coords:
- ex (local x = beam axis) = (0, 0, 1) = global Z
- ey (local y = normal) = (1, 0, 0) = global X
- ez (local z) = ex × ey = (0, 1, 0) = global Y
"""

import numpy as np

def main():
    print("=" * 80)
    print("BEAM COORDINATE SYSTEM ANALYSIS")
    print("=" * 80)

    # Beam nodes
    node1 = np.array([0.0, 0.0, 0.0])
    node3 = np.array([0.0, 0.0, 10.0])

    # Beam direction (local x = beam axis)
    beam_vec = node3 - node1
    length = np.linalg.norm(beam_vec)
    ex = beam_vec / length

    print(f"\nBeam geometry:")
    print(f"  Node 1: {node1}")
    print(f"  Node 3: {node3}")
    print(f"  Length: {length}")
    print(f"  ex (local x, beam axis): {ex}")

    # Normal direction from INP (defines local y)
    normal = np.array([1.0, 0.0, 0.0])
    print(f"  Specified normal: {normal}")

    # For beam along Z, ey should be X direction
    global_x = np.array([1.0, 0.0, 0.0])
    global_z = np.array([0.0, 0.0, 1.0])

    # Choose ey perpendicular to beam axis
    if abs(np.dot(ex, global_z)) > 0.9:
        # Beam nearly along Z, use X for ey
        ey = global_x
        print(f"  ey (local y): {ey} (along global X)")
    else:
        # General case
        temp = np.cross(ex, global_z)
        ey = temp / np.linalg.norm(temp)
        print(f"  ey (local y): {ey}")

    # Local z completes right-handed system
    ez = np.cross(ex, ey)
    ez = ez / np.linalg.norm(ez)
    print(f"  ez (local z): {ez} (perpendicular)")

    # Build rotation matrix R = [ex | ey | ez]
    R = np.column_stack([ex, ey, ez])
    print(f"\nRotation matrix R (local → global):")
    print(R)

    # Test transformation
    print("\n" + "=" * 80)
    print("STRESS TRANSFORMATION TEST")
    print("=" * 80)

    # Local stress state (beam theory)
    # Main stress along beam axis (local x)
    sxx_local = 1000.0  # Axial + bending
    syy_local = -300.0  # Transverse (Poisson + anticlastic)
    szz_local = -300.0  # Transverse
    sxy_local = -100.0  # Coupling
    sxz_local = 50.0    # Shear
    syz_local = 0.0

    print("\nLocal stress tensor (in beam coordinates):")
    stress_local = np.array([
        [sxx_local, sxy_local, sxz_local],
        [sxy_local, syy_local, syz_local],
        [sxz_local, syz_local, szz_local]
    ])
    print(stress_local)
    print(f"  sxx_local = {sxx_local} (main stress along beam)")
    print(f"  syy_local = {syy_local} (transverse)")
    print(f"  szz_local = {szz_local} (transverse)")
    print(f"  sxy_local = {sxy_local} (coupling)")

    # Transform to global: σ_global = R * σ_local * R^T
    stress_global = R @ stress_local @ R.T

    print("\nGlobal stress tensor:")
    print(stress_global)
    print(f"  sxx_global = {stress_global[0,0]:.1f}")
    print(f"  syy_global = {stress_global[1,1]:.1f}")
    print(f"  szz_global = {stress_global[2,2]:.1f} (should be main - beam along Z!)")
    print(f"  sxy_global = {stress_global[0,1]:.1f}")
    print(f"  sxz_global = {stress_global[0,2]:.1f}")
    print(f"  syz_global = {stress_global[1,2]:.1f}")

    print("\n" + "=" * 80)
    print("EXPECTED vs ACTUAL")
    print("=" * 80)
    print("\nSince beam is along Z-axis:")
    print(f"  Expected: szz_global ≈ sxx_local = {sxx_local}")
    print(f"  Actual:   szz_global = {stress_global[2,2]:.1f}")
    print(f"  Match: {'✓' if abs(stress_global[2,2] - sxx_local) < 1 else '✗'}")

    print("\nTransverse stresses should be in X,Y directions:")
    print(f"  sxx_global = {stress_global[0,0]:.1f} (should be transverse)")
    print(f"  syy_global = {stress_global[1,1]:.1f} (should be transverse)")

    # Compare with reference pattern
    print("\n" + "=" * 80)
    print("REFERENCE PATTERN (IP 1)")
    print("=" * 80)
    ref_sxx = -135.52
    ref_syy = -44.82
    ref_szz = 468.51
    ref_sxy = -44.81

    print(f"Reference:")
    print(f"  sxx = {ref_sxx} (transverse)")
    print(f"  syy = {ref_syy} (transverse)")
    print(f"  szz = {ref_szz} (main - along beam!)")
    print(f"  sxy = {ref_sxy} (≈ syy)")

    print(f"\nPattern observations:")
    print(f"  |szz| > |sxx| > |syy|: {abs(ref_szz) > abs(ref_sxx) > abs(ref_syy)}")
    print(f"  sxy ≈ syy: {abs(ref_sxy - ref_syy) < 1}")
    print(f"  szz is main stress (beam along Z)")

if __name__ == '__main__':
    main()
