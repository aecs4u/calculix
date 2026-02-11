"""
Python wrapper around pyNastran for reading BDF and OP2 files

This module provides a simplified interface to pyNastran that returns
data in a JSON-serializable format for easy consumption by Rust via PyO3.
"""

import json
from pathlib import Path
from typing import Dict, List, Any, Optional

try:
    from pyNastran.bdf.bdf import BDF
    from pyNastran.op2.op2 import OP2
    PYNASTRAN_AVAILABLE = True
except ImportError:
    PYNASTRAN_AVAILABLE = False
    print("WARNING: pyNastran not installed. Install with: pip install pyNastran")


class BdfReader:
    """Wrapper for reading Nastran BDF files"""

    def __init__(self, filepath: str):
        if not PYNASTRAN_AVAILABLE:
            raise ImportError("pyNastran is required but not installed")

        self.filepath = Path(filepath)
        if not self.filepath.exists():
            raise FileNotFoundError(f"BDF file not found: {filepath}")

        self.model = BDF(debug=False)
        self.model.read_bdf(str(self.filepath))

    def to_dict(self) -> Dict[str, Any]:
        """Convert BDF data to dictionary"""
        return {
            'nodes': self._extract_nodes(),
            'elements': self._extract_elements(),
            'materials': self._extract_materials(),
            'properties': self._extract_properties(),
        }

    def to_json(self) -> str:
        """Convert BDF data to JSON string"""
        return json.dumps(self.to_dict())

    def _extract_nodes(self) -> Dict[int, Dict[str, Any]]:
        """Extract node data"""
        nodes = {}
        for nid, node in self.model.nodes.items():
            pos = node.get_position()
            nodes[nid] = {
                'id': nid,
                'x': float(pos[0]),
                'y': float(pos[1]),
                'z': float(pos[2]),
            }
        return nodes

    def _extract_elements(self) -> Dict[int, Dict[str, Any]]:
        """Extract element data"""
        elements = {}
        for eid, elem in self.model.elements.items():
            elements[eid] = {
                'id': eid,
                'elem_type': elem.type,
                'nodes': [int(nid) for nid in elem.node_ids],
                'property_id': elem.pid if hasattr(elem, 'pid') else 0,
            }
        return elements

    def _extract_materials(self) -> Dict[int, Dict[str, Any]]:
        """Extract material data"""
        materials = {}
        for mid, mat in self.model.materials.items():
            mat_data = {
                'id': mid,
                'name': f"MAT{mid}",
                'elastic_modulus': None,
                'poissons_ratio': None,
                'density': None,
            }

            # MAT1 cards (isotropic)
            if hasattr(mat, 'e'):
                mat_data['elastic_modulus'] = float(mat.e) if mat.e is not None else None
            if hasattr(mat, 'nu'):
                mat_data['poissons_ratio'] = float(mat.nu) if mat.nu is not None else None
            if hasattr(mat, 'rho'):
                mat_data['density'] = float(mat.rho) if mat.rho is not None else None

            materials[mid] = mat_data

        return materials

    def _extract_properties(self) -> Dict[int, Dict[str, Any]]:
        """Extract property data"""
        properties = {}
        for pid, prop in self.model.properties.items():
            prop_data = {
                'id': pid,
                'property_type': prop.type,
                'material_id': prop.mid if hasattr(prop, 'mid') else 0,
                'thickness': None,
                'area': None,
            }

            # Shell thickness
            if hasattr(prop, 't'):
                prop_data['thickness'] = float(prop.t) if prop.t is not None else None

            # Rod/beam area
            if hasattr(prop, 'A'):
                prop_data['area'] = float(prop.A) if prop.A is not None else None

            properties[pid] = prop_data

        return properties


class Op2Reader:
    """Wrapper for reading Nastran OP2 files"""

    def __init__(self, filepath: str):
        if not PYNASTRAN_AVAILABLE:
            raise ImportError("pyNastran is required but not installed")

        self.filepath = Path(filepath)
        if not self.filepath.exists():
            raise FileNotFoundError(f"OP2 file not found: {filepath}")

        self.model = OP2(debug=False)
        self.model.read_op2(str(self.filepath))

    def to_dict(self) -> Dict[str, Any]:
        """Convert OP2 data to dictionary"""
        return {
            'displacements': self._extract_displacements(),
            'stresses': self._extract_stresses(),
            'eigenvalues': self._extract_eigenvalues(),
            'eigenvectors': self._extract_eigenvectors(),
        }

    def to_json(self) -> str:
        """Convert OP2 data to JSON string"""
        return json.dumps(self.to_dict())

    def _extract_displacements(self) -> Dict[int, Dict[str, float]]:
        """Extract displacement data"""
        displacements = {}

        if hasattr(self.model, 'displacements') and self.model.displacements:
            # Get first subcase
            subcase_id = list(self.model.displacements.keys())[0]
            disp_obj = self.model.displacements[subcase_id]

            for nid in disp_obj.node_gridtype[:, 0]:
                idx = disp_obj.node_gridtype[:, 0] == nid
                data = disp_obj.data[0, idx, :].flatten()

                displacements[int(nid)] = {
                    'node_id': int(nid),
                    'dx': float(data[0]),
                    'dy': float(data[1]),
                    'dz': float(data[2]),
                    'rx': float(data[3]) if len(data) > 3 else 0.0,
                    'ry': float(data[4]) if len(data) > 4 else 0.0,
                    'rz': float(data[5]) if len(data) > 5 else 0.0,
                }

        return displacements

    def _extract_stresses(self) -> Dict[int, Dict[str, float]]:
        """Extract stress data"""
        stresses = {}

        # TODO: Implement stress extraction based on element type
        # This is a placeholder for now

        return stresses

    def _extract_eigenvalues(self) -> List[float]:
        """Extract eigenvalues from modal analysis"""
        eigenvalues = []

        if hasattr(self.model, 'eigenvalues') and self.model.eigenvalues:
            subcase_id = list(self.model.eigenvalues.keys())[0]
            eig_obj = self.model.eigenvalues[subcase_id]
            eigenvalues = [float(val) for val in eig_obj.eigenvalues]

        return eigenvalues

    def _extract_eigenvectors(self) -> Dict[int, List[float]]:
        """Extract eigenvectors (mode shapes)"""
        eigenvectors = {}

        if hasattr(self.model, 'eigenvectors') and self.model.eigenvectors:
            subcase_id = list(self.model.eigenvectors.keys())[0]
            eigvec_obj = self.model.eigenvectors[subcase_id]

            for mode_idx in range(eigvec_obj.data.shape[0]):
                mode_data = eigvec_obj.data[mode_idx, :, :].flatten()
                eigenvectors[mode_idx + 1] = [float(val) for val in mode_data]

        return eigenvectors


def read_bdf(filepath: str) -> BdfReader:
    """Read a BDF file and return a reader object"""
    return BdfReader(filepath)


def read_op2(filepath: str) -> Op2Reader:
    """Read an OP2 file and return a reader object"""
    return Op2Reader(filepath)


def get_bdf_stats(filepath: str) -> Dict[str, Any]:
    """Get basic statistics from a BDF file without full parsing"""
    reader = BdfReader(filepath)
    data = reader.to_dict()

    element_types = set()
    for elem in data['elements'].values():
        element_types.add(elem['elem_type'])

    return {
        'num_nodes': len(data['nodes']),
        'num_elements': len(data['elements']),
        'num_materials': len(data['materials']),
        'num_properties': len(data['properties']),
        'element_types': sorted(list(element_types)),
    }


if __name__ == '__main__':
    # Test if pyNastran is available
    if PYNASTRAN_AVAILABLE:
        print("✓ pyNastran is installed and available")
    else:
        print("✗ pyNastran is NOT installed")
        print("Install with: pip install pyNastran")
