"""Meshio wrapper for CalculiX I/O operations.

This module provides mesh format conversion via meshio:
- Read: VTK, VTU, XDMF, STL, OFF, OBJ, PLY, Gmsh, ANSYS, Abaqus, etc.
- Write: VTK, VTU, XDMF, STL, OFF, OBJ, PLY, Gmsh, ANSYS, Abaqus, etc.
- Convert: Any supported format to any other format
- Extract: Mesh statistics, element types, node coordinates

Homepage: https://github.com/nschloe/meshio
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

try:
    import meshio
    MESHIO_AVAILABLE = True
except ImportError:
    MESHIO_AVAILABLE = False


def check_meshio_available() -> bool:
    """Check if meshio is available."""
    return MESHIO_AVAILABLE


def get_supported_formats() -> Dict[str, List[str]]:
    """Get list of formats supported by meshio.
    
    Returns:
        dict with 'read' and 'write' keys containing lists of extensions
    """
    if not MESHIO_AVAILABLE:
        return {"read": [], "write": []}
    
    return {
        "read": list(meshio.extension_to_filetype.keys()),
        "write": list(meshio._writer_map.keys()) if hasattr(meshio, '_writer_map') else []
    }


def read_mesh(file_path: str | Path) -> Dict[str, Any]:
    """Read a mesh file and return structured information.
    
    Args:
        file_path: Path to mesh file (any meshio-supported format)
        
    Returns:
        dict with mesh data:
        - num_points: int
        - num_cells: int
        - point_data_fields: list of str
        - cell_data_fields: list of str
        - cell_types: list of str (element types)
        - bounds: [xmin, xmax, ymin, ymax, zmin, zmax]
        
    Raises:
        ImportError: If meshio is not installed
        FileNotFoundError: If file doesn't exist
        ValueError: If file format is not supported
    """
    if not MESHIO_AVAILABLE:
        raise ImportError("meshio is not installed. Install with: pip install meshio")
    
    file_path = Path(file_path)
    if not file_path.exists():
        raise FileNotFoundError(f"File not found: {file_path}")
    
    mesh = meshio.read(file_path)
    
    # Compute bounds
    points = mesh.points
    bounds = [
        float(points[:, 0].min()), float(points[:, 0].max()),
        float(points[:, 1].min()), float(points[:, 1].max()),
        float(points[:, 2].min()), float(points[:, 2].max()),
    ]
    
    # Extract cell types
    cell_types = [cell_block.type for cell_block in mesh.cells]
    
    # Count total cells
    num_cells = sum(len(cell_block.data) for cell_block in mesh.cells)
    
    return {
        "num_points": len(mesh.points),
        "num_cells": num_cells,
        "point_data_fields": list(mesh.point_data.keys()),
        "cell_data_fields": list(mesh.cell_data.keys()),
        "cell_types": cell_types,
        "bounds": bounds,
    }


def convert_mesh(input_path: str | Path, output_path: str | Path,
                 cell_type: Optional[str] = None) -> Dict[str, Any]:
    """Convert mesh from one format to another.
    
    Args:
        input_path: Input mesh file
        output_path: Output mesh file
        cell_type: Optional filter for specific cell type (e.g., "triangle", "tetra")
        
    Returns:
        dict with conversion info:
        - success: bool
        - input_format: str
        - output_format: str
        - num_points: int
        - num_cells: int
        
    Raises:
        ImportError: If meshio is not installed
        FileNotFoundError: If input file doesn't exist
    """
    if not MESHIO_AVAILABLE:
        raise ImportError("meshio is not installed. Install with: pip install meshio")
    
    input_path = Path(input_path)
    output_path = Path(output_path)
    
    if not input_path.exists():
        raise FileNotFoundError(f"Input file not found: {input_path}")
    
    # Read mesh
    mesh = meshio.read(input_path)
    
    # Filter by cell type if requested
    if cell_type:
        mesh = mesh.cells_dict([cell_type]) if hasattr(mesh, 'cells_dict') else mesh
    
    # Write mesh
    meshio.write(output_path, mesh)
    
    # Count cells
    num_cells = sum(len(cell_block.data) for cell_block in mesh.cells)
    
    return {
        "success": True,
        "input_format": input_path.suffix,
        "output_format": output_path.suffix,
        "num_points": len(mesh.points),
        "num_cells": num_cells,
    }


def extract_point_data(file_path: str | Path, field_name: str) -> Dict[str, Any]:
    """Extract point data field from mesh.
    
    Args:
        file_path: Path to mesh file
        field_name: Name of point data field to extract
        
    Returns:
        dict with:
        - field_name: str
        - num_points: int
        - min_value: float
        - max_value: float
        - mean_value: float
        
    Raises:
        ImportError: If meshio is not installed
        KeyError: If field doesn't exist
    """
    if not MESHIO_AVAILABLE:
        raise ImportError("meshio is not installed. Install with: pip install meshio")
    
    mesh = meshio.read(file_path)
    
    if field_name not in mesh.point_data:
        raise KeyError(f"Field '{field_name}' not found. Available: {list(mesh.point_data.keys())}")
    
    data = mesh.point_data[field_name]
    
    # Handle vector/tensor data (take magnitude)
    if len(data.shape) > 1:
        import numpy as np
        data = np.linalg.norm(data, axis=1)
    
    return {
        "field_name": field_name,
        "num_points": len(data),
        "min_value": float(data.min()),
        "max_value": float(data.max()),
        "mean_value": float(data.mean()),
    }


def get_mesh_info(file_path: str | Path, verbose: bool = False) -> Dict[str, Any]:
    """Get comprehensive mesh information.
    
    Args:
        file_path: Path to mesh file
        verbose: Include detailed cell-by-cell info
        
    Returns:
        dict with comprehensive mesh metadata
    """
    if not MESHIO_AVAILABLE:
        raise ImportError("meshio is not installed. Install with: pip install meshio")
    
    mesh = meshio.read(file_path)
    
    info = {
        "file": str(file_path),
        "num_points": len(mesh.points),
        "num_cells": sum(len(cell_block.data) for cell_block in mesh.cells),
        "point_data": list(mesh.point_data.keys()),
        "cell_data": list(mesh.cell_data.keys()),
        "field_data": mesh.field_data if hasattr(mesh, 'field_data') else {},
        "cell_sets": list(mesh.cell_sets.keys()) if hasattr(mesh, 'cell_sets') else [],
        "point_sets": list(mesh.point_sets.keys()) if hasattr(mesh, 'point_sets') else [],
    }
    
    if verbose:
        info["cells"] = [
            {
                "type": cell_block.type,
                "count": len(cell_block.data),
            }
            for cell_block in mesh.cells
        ]
    
    return info


def cli_main():
    """Command-line interface for meshio wrapper."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Meshio wrapper for mesh format conversion and inspection"
    )
    subparsers = parser.add_subparsers(dest="command", help="Command to execute")
    
    # info command
    info_parser = subparsers.add_parser("info", help="Get mesh information")
    info_parser.add_argument("file", help="Mesh file to inspect")
    info_parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    
    # convert command
    convert_parser = subparsers.add_parser("convert", help="Convert mesh format")
    convert_parser.add_argument("input", help="Input mesh file")
    convert_parser.add_argument("output", help="Output mesh file")
    convert_parser.add_argument("--cell-type", help="Filter by cell type")
    
    # formats command
    formats_parser = subparsers.add_parser("formats", help="List supported formats")
    
    # extract command
    extract_parser = subparsers.add_parser("extract", help="Extract point data field")
    extract_parser.add_argument("file", help="Mesh file")
    extract_parser.add_argument("field", help="Field name to extract")
    
    args = parser.parse_args()
    
    if not MESHIO_AVAILABLE:
        print("ERROR: meshio is not installed", file=sys.stderr)
        print("Install with: pip install meshio", file=sys.stderr)
        sys.exit(1)
    
    try:
        if args.command == "info":
            result = get_mesh_info(args.file, verbose=args.verbose)
            print(json.dumps(result, indent=2))
            
        elif args.command == "convert":
            result = convert_mesh(args.input, args.output, cell_type=args.cell_type)
            print(json.dumps(result, indent=2))
            print(f"✓ Converted {args.input} → {args.output}")
            
        elif args.command == "formats":
            result = get_supported_formats()
            print(json.dumps(result, indent=2))
            
        elif args.command == "extract":
            result = extract_point_data(args.file, args.field)
            print(json.dumps(result, indent=2))
            
        else:
            parser.print_help()
            
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    cli_main()
