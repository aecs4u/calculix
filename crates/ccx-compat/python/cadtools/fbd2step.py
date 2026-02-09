"""Wrapper for fbd2step: Convert CGX FBD format to STEP CAD files

Based on cgxCadTools by Pascal Mossier
"""

import os
import subprocess
import shutil
from pathlib import Path
from typing import Optional


class Fbd2StepError(Exception):
    """Exception raised when FBD to STEP conversion fails"""
    pass


def find_fbd2step_binary() -> Optional[str]:
    """Find fbd2step binary in PATH or common locations"""
    # Check PATH first
    binary = shutil.which("fbd2step")
    if binary:
        return binary

    # Check common installation locations
    common_paths = [
        "/usr/local/bin/fbd2step",
        "/usr/bin/fbd2step",
        str(Path.home() / "bin" / "fbd2step"),
        "../../../calculix_migration_tooling/cgxCadTools/FbdReader/bin/fbd2step",
    ]

    for path in common_paths:
        if os.path.isfile(path) and os.access(path, os.X_OK):
            return path

    return None


def convert_fbd_to_step(
    input_file: str,
    output_file: Optional[str] = None,
    verbose: bool = False,
) -> str:
    """Convert CGX FBD file to STEP CAD format

    Args:
        input_file: Path to input FBD file
        output_file: Path to output STEP file (default: output.step)
        verbose: Enable verbose output

    Returns:
        Path to generated STEP file

    Raises:
        Fbd2StepError: If conversion fails
        FileNotFoundError: If binary or input file not found
    """
    # Find binary
    binary = find_fbd2step_binary()
    if not binary:
        raise FileNotFoundError(
            "fbd2step binary not found. Please install cgxCadTools or add to PATH."
        )

    # Verify input file exists
    if not os.path.isfile(input_file):
        raise FileNotFoundError(f"Input file not found: {input_file}")

    # Build command
    cmd = [binary]

    if verbose:
        cmd.append("-v")

    cmd.append(input_file)

    if output_file:
        cmd.extend(["-o", output_file])

    # Run conversion
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=True,
        )

        # Determine output file
        final_output = output_file if output_file else "output.step"

        if not os.path.isfile(final_output):
            raise Fbd2StepError(f"Output file not created: {final_output}")

        if verbose and result.stdout:
            print(result.stdout)

        return final_output

    except subprocess.CalledProcessError as e:
        error_msg = f"Conversion failed:\n{e.stderr}"
        if e.stdout:
            error_msg += f"\nOutput:\n{e.stdout}"
        raise Fbd2StepError(error_msg)
    except subprocess.SubprocessError as e:
        raise Fbd2StepError(f"Failed to run fbd2step: {e}")


# CLI interface
if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(
        description="Convert CGX FBD files to STEP CAD format"
    )
    parser.add_argument("input", help="Input FBD file")
    parser.add_argument(
        "-o", "--output", help="Output STEP file (default: output.step)"
    )
    parser.add_argument("-v", "--verbose", action="store_true",
                       help="Enable verbose output")

    args = parser.parse_args()

    try:
        output = convert_fbd_to_step(
            args.input,
            args.output,
            verbose=args.verbose,
        )
        print(f"✓ Conversion successful: {output}")
    except Exception as e:
        print(f"✗ Error: {e}")
        exit(1)
