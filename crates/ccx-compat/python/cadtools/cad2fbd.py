"""Wrapper for cad2fbd: Convert STEP/IGES CAD files to CGX FBD format

Based on cgxCadTools by Pascal Mossier
Requires OpenCASCADE Technology 6.9.1 or later
"""

import os
import subprocess
import shutil
from pathlib import Path
from typing import Optional, List


class Cad2FbdError(Exception):
    """Exception raised when CAD conversion fails"""
    pass


def find_cad2fbd_binary() -> Optional[str]:
    """Find cad2fbd binary in PATH or common locations"""
    # Check PATH first
    binary = shutil.which("cad2fbd")
    if binary:
        return binary

    # Check common installation locations
    common_paths = [
        "/usr/local/bin/cad2fbd",
        "/usr/bin/cad2fbd",
        str(Path.home() / "bin" / "cad2fbd"),
        "../../../calculix_migration_tooling/cgxCadTools/CadReader/bin/cad2fbd",
    ]

    for path in common_paths:
        if os.path.isfile(path) and os.access(path, os.X_OK):
            return path

    return None


def convert_cad_to_fbd(
    input_file: str,
    output_file: Optional[str] = None,
    split_closed: bool = False,
    split_discontinuous: bool = False,
    no_split: bool = True,
    split_all: bool = False,
    convert_planes: bool = False,
    convert_cylinders: bool = False,
    convert_cones: bool = False,
    convert_spheres: bool = False,
    convert_torus: bool = False,
    verbose: bool = False,
    additional_flags: Optional[List[str]] = None,
) -> str:
    """Convert STEP or IGES CAD file to CGX FBD format

    Args:
        input_file: Path to input CAD file (.step, .stp, .iges, .igs)
        output_file: Path to output FBD file (default: result.fbd)
        split_closed: Split closed surfaces like cylinders
        split_discontinuous: Split C1-discontinuous surfaces
        no_split: No splitting (default), correct only invalid NURBS
        split_all: Use all splitting criteria
        convert_planes: Convert planes to NURBS surfaces
        convert_cylinders: Convert cylinders to NURBS surfaces
        convert_cones: Convert cones to NURBS surfaces
        convert_spheres: Convert spheres to NURBS surfaces
        convert_torus: Convert toroidal shapes to NURBS surfaces
        verbose: Display topology tree
        additional_flags: Additional command-line flags

    Returns:
        Path to generated FBD file

    Raises:
        Cad2FbdError: If conversion fails
        FileNotFoundError: If binary or input file not found
    """
    # Find binary
    binary = find_cad2fbd_binary()
    if not binary:
        raise FileNotFoundError(
            "cad2fbd binary not found. Please install cgxCadTools or add to PATH."
        )

    # Verify input file exists
    if not os.path.isfile(input_file):
        raise FileNotFoundError(f"Input file not found: {input_file}")

    # Build command
    cmd = [binary]

    # Add flags
    if split_closed:
        cmd.append("-scl")
    elif split_discontinuous:
        cmd.append("-sco")
    elif split_all:
        cmd.append("-sa")
    elif no_split:
        cmd.append("-ns")

    if convert_planes:
        cmd.append("-pln")
    if convert_cylinders:
        cmd.append("-cyl")
    if convert_cones:
        cmd.append("-con")
    if convert_spheres:
        cmd.append("-sph")
    if convert_torus:
        cmd.append("-tor")

    if verbose:
        cmd.append("-v")

    if additional_flags:
        cmd.extend(additional_flags)

    # Add input file
    cmd.append(input_file)

    # Run conversion
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=False,  # Don't raise on non-zero exit (cad2fbd sometimes exits with error but produces valid output)
        )

        # Check for output file (default is result.fbd)
        default_output = "result.fbd"
        if not os.path.isfile(default_output):
            # If no output file, conversion failed
            error_msg = f"Conversion failed:\n{result.stderr}"
            if result.stdout:
                error_msg += f"\nOutput:\n{result.stdout}"
            raise Cad2FbdError(error_msg)

        # Move to desired output location if specified
        final_output = output_file if output_file else default_output
        if output_file and output_file != default_output:
            import shutil as sh
            sh.move(default_output, output_file)

        return final_output

    except subprocess.SubprocessError as e:
        raise Cad2FbdError(f"Failed to run cad2fbd: {e}")


# CLI interface
if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(
        description="Convert STEP/IGES CAD files to CGX FBD format"
    )
    parser.add_argument("input", help="Input CAD file (.step, .stp, .iges, .igs)")
    parser.add_argument(
        "-o", "--output", help="Output FBD file (default: result.fbd)"
    )
    parser.add_argument("-scl", "--split-closed", action="store_true",
                       help="Split closed surfaces")
    parser.add_argument("-sco", "--split-discontinuous", action="store_true",
                       help="Split C1-discontinuous surfaces")
    parser.add_argument("-sa", "--split-all", action="store_true",
                       help="Use all splitting criteria")
    parser.add_argument("-pln", "--convert-planes", action="store_true",
                       help="Convert planes to NURBS")
    parser.add_argument("-cyl", "--convert-cylinders", action="store_true",
                       help="Convert cylinders to NURBS")
    parser.add_argument("-v", "--verbose", action="store_true",
                       help="Display topology tree")

    args = parser.parse_args()

    try:
        output = convert_cad_to_fbd(
            args.input,
            args.output,
            split_closed=args.split_closed,
            split_discontinuous=args.split_discontinuous,
            split_all=args.split_all,
            convert_planes=args.convert_planes,
            convert_cylinders=args.convert_cylinders,
            verbose=args.verbose,
        )
        print(f"✓ Conversion successful: {output}")
    except Exception as e:
        print(f"✗ Error: {e}")
        exit(1)
