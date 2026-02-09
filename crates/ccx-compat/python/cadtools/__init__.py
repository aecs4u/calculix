"""CAD format conversion tools for CalculiX CGX

Python wrappers for cgxCadTools binaries:
- cad2fbd: Convert STEP/IGES to CGX FBD format
- fbd2step: Convert FBD to STEP format
"""

from .cad2fbd import convert_cad_to_fbd
from .fbd2step import convert_fbd_to_step

__all__ = ["convert_cad_to_fbd", "convert_fbd_to_step"]
__version__ = "0.1.0"
