"""
Nastran file conversion utilities for validation API

Provides BDF → INP conversion and OP2 result reading via pyNastran
"""

import sys
from pathlib import Path
from typing import Dict, List, Any, Optional

# Add ccx-io python path
CCX_IO_PATH = Path(__file__).parent.parent.parent / "ccx-io" / "python"
if CCX_IO_PATH.exists():
    sys.path.insert(0, str(CCX_IO_PATH))

try:
    from nastran_io import read_bdf, read_op2, get_bdf_stats, PYNASTRAN_AVAILABLE
except ImportError:
    PYNASTRAN_AVAILABLE = False
    print("WARNING: nastran_io module not found. Nastran support disabled.")


class BdfToInpConverter:
    """Convert Nastran BDF to CalculiX INP format"""

    @staticmethod
    def convert_file(bdf_path: str, inp_path: str) -> Dict[str, Any]:
        """
        Convert a BDF file to INP format

        Args:
            bdf_path: Path to input BDF file
            inp_path: Path to output INP file

        Returns:
            Conversion statistics
        """
        if not PYNASTRAN_AVAILABLE:
            raise RuntimeError("pyNastran is not available. Install with: pip install pyNastran")

        # Read BDF
        bdf_reader = read_bdf(bdf_path)
        bdf_data = bdf_reader.to_dict()

        # Convert to INP
        inp_content = BdfToInpConverter._convert_to_inp(bdf_data)

        # Write INP
        with open(inp_path, 'w') as f:
            f.write(inp_content)

        return {
            'num_nodes': len(bdf_data['nodes']),
            'num_elements': len(bdf_data['elements']),
            'num_materials': len(bdf_data['materials']),
            'num_properties': len(bdf_data['properties']),
        }

    @staticmethod
    def _convert_to_inp(bdf_data: Dict[str, Any]) -> str:
        """Convert BDF data dictionary to INP format string"""
        lines = []

        # Header
        lines.append("** CalculiX Input File")
        lines.append("** Converted from Nastran BDF")
        lines.append("**")
        lines.append("")

        # Nodes
        lines.append("*NODE")
        for node_id in sorted(bdf_data['nodes'].keys()):
            node = bdf_data['nodes'][node_id]
            lines.append(f"{node['id']}, {node['x']:.6e}, {node['y']:.6e}, {node['z']:.6e}")

        lines.append("")

        # Elements by type
        elements_by_type: Dict[str, List[Dict]] = {}
        for elem in bdf_data['elements'].values():
            elem_type = elem['elem_type']
            if elem_type not in elements_by_type:
                elements_by_type[elem_type] = []
            elements_by_type[elem_type].append(elem)

        for nastran_type, elements in elements_by_type.items():
            ccx_type = BdfToInpConverter._map_element_type(nastran_type)
            if ccx_type:
                lines.append(f"*ELEMENT, TYPE={ccx_type}")
                for elem in elements:
                    node_str = ", ".join(str(nid) for nid in elem['nodes'])
                    lines.append(f"{elem['id']}, {node_str}")
                lines.append("")

        # Materials
        for mat_id in sorted(bdf_data['materials'].keys()):
            mat = bdf_data['materials'][mat_id]
            lines.append(f"*MATERIAL, NAME={mat['name']}")

            if mat['elastic_modulus'] is not None and mat['poissons_ratio'] is not None:
                lines.append("*ELASTIC")
                lines.append(f"{mat['elastic_modulus']:.6e}, {mat['poissons_ratio']:.6e}")

            if mat['density'] is not None:
                lines.append("*DENSITY")
                lines.append(f"{mat['density']:.6e}")

            lines.append("")

        # Element and node sets
        lines.append("*ELSET, ELSET=ALL")
        elem_ids = ", ".join(str(eid) for eid in sorted(bdf_data['elements'].keys()))
        lines.append(elem_ids)
        lines.append("")

        lines.append("*NSET, NSET=ALL")
        node_ids = ", ".join(str(nid) for nid in sorted(bdf_data['nodes'].keys()))
        lines.append(node_ids)
        lines.append("")

        return "\n".join(lines)

    @staticmethod
    def _map_element_type(nastran_type: str) -> Optional[str]:
        """Map Nastran element type to CalculiX type"""
        mapping = {
            "CROD": "T3D2",
            "CONROD": "T3D2",
            "CBAR": "B31",
            "CBEAM": "B31",
            "CQUAD4": "S4",
            "CTRIA3": "S3",
            "CHEXA": "C3D8",
            "CHEXA8": "C3D8",
            "CTETRA": "C3D4",
            "CTETRA4": "C3D4",
            "CPENTA": "C3D6",
            "CPENTA6": "C3D6",
        }
        return mapping.get(nastran_type)


class Op2ResultReader:
    """Read Nastran OP2 binary result files"""

    @staticmethod
    def read_results(op2_path: str) -> Dict[str, Any]:
        """
        Read results from an OP2 file

        Args:
            op2_path: Path to OP2 file

        Returns:
            Dictionary with displacements, stresses, eigenvalues, etc.
        """
        if not PYNASTRAN_AVAILABLE:
            raise RuntimeError("pyNastran is not available. Install with: pip install pyNastran")

        op2_reader = read_op2(op2_path)
        return op2_reader.to_dict()

    @staticmethod
    def extract_frequencies(op2_path: str) -> List[float]:
        """
        Extract natural frequencies from modal analysis OP2

        Args:
            op2_path: Path to OP2 file

        Returns:
            List of frequencies in Hz
        """
        data = Op2ResultReader.read_results(op2_path)
        eigenvalues = data.get('eigenvalues', [])

        # Convert eigenvalues to frequencies (eigenvalue = ω², f = ω/(2π))
        import math
        frequencies = []
        for lam in eigenvalues:
            if lam > 0:
                omega = math.sqrt(lam)
                freq_hz = omega / (2 * math.pi)
                frequencies.append(freq_hz)
            else:
                frequencies.append(0.0)

        return frequencies


def is_pynastran_available() -> bool:
    """Check if pyNastran is available"""
    return PYNASTRAN_AVAILABLE


if __name__ == '__main__':
    # Test availability
    if is_pynastran_available():
        print("✓ pyNastran is available")
    else:
        print("✗ pyNastran is NOT available")
        print("Install with: pip install pyNastran")
