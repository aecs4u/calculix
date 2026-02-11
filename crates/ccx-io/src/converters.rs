//! Format conversion utilities
//!
//! Provides converters between different FEA file formats:
//! - BDF → INP (Nastran to CalculiX)
//! - OP2 → DAT (Nastran results to CalculiX format)

use crate::error::{IoError, Result};
use crate::nastran::{BdfData, Element, Material, Node, Property};
use std::collections::HashMap;

/// Converter from Nastran BDF to CalculiX INP format
pub struct BdfToInpConverter {
    node_map: HashMap<i32, i32>,
    element_map: HashMap<i32, i32>,
}

impl BdfToInpConverter {
    /// Create a new converter
    pub fn new() -> Self {
        Self {
            node_map: HashMap::new(),
            element_map: HashMap::new(),
        }
    }

    /// Convert BDF data to INP format string
    ///
    /// # Arguments
    /// * `bdf_data` - Parsed BDF data
    ///
    /// # Returns
    /// INP file content as string
    pub fn convert(&mut self, bdf_data: &BdfData) -> Result<String> {
        let mut inp = String::new();

        // Header
        inp.push_str("** CalculiX Input File\n");
        inp.push_str("** Converted from Nastran BDF\n");
        inp.push_str("**\n");

        // Nodes
        inp.push_str("*NODE\n");
        let mut sorted_nodes: Vec<_> = bdf_data.nodes.iter().collect();
        sorted_nodes.sort_by_key(|(id, _)| *id);

        for (id, node) in sorted_nodes {
            inp.push_str(&format!("{}, {:.6e}, {:.6e}, {:.6e}\n",
                id, node.x, node.y, node.z));
            self.node_map.insert(*id, *id);
        }

        // Elements by type
        let mut elements_by_type: HashMap<String, Vec<(&i32, &Element)>> = HashMap::new();
        for (id, elem) in &bdf_data.elements {
            elements_by_type.entry(elem.elem_type.clone())
                .or_insert_with(Vec::new)
                .push((id, elem));
        }

        for (elem_type, elements) in elements_by_type.iter() {
            let ccx_type = self.map_element_type(elem_type)?;

            inp.push_str(&format!("*ELEMENT, TYPE={}\n", ccx_type));

            for (id, elem) in elements {
                inp.push_str(&format!("{}", id));
                for node_id in &elem.nodes {
                    inp.push_str(&format!(", {}", node_id));
                }
                inp.push_str("\n");
                self.element_map.insert(**id, **id);
            }
        }

        // Materials
        for (mat_id, material) in &bdf_data.materials {
            inp.push_str(&format!("*MATERIAL, NAME={}\n", material.name));

            if let (Some(e), Some(nu)) = (material.elastic_modulus, material.poissons_ratio) {
                inp.push_str("*ELASTIC\n");
                inp.push_str(&format!("{:.6e}, {:.6e}\n", e, nu));
            }

            if let Some(rho) = material.density {
                inp.push_str("*DENSITY\n");
                inp.push_str(&format!("{:.6e}\n", rho));
            }
        }

        // Element sets (optional - could group by property)
        inp.push_str("*ELSET, ELSET=ALL\n");
        for id in bdf_data.elements.keys() {
            inp.push_str(&format!("{}, ", id));
        }
        inp.push_str("\n");

        // Node sets
        inp.push_str("*NSET, NSET=ALL\n");
        for id in bdf_data.nodes.keys() {
            inp.push_str(&format!("{}, ", id));
        }
        inp.push_str("\n");

        Ok(inp)
    }

    /// Map Nastran element type to CalculiX element type
    fn map_element_type(&self, nastran_type: &str) -> Result<String> {
        match nastran_type {
            // Rod elements
            "CROD" | "CONROD" => Ok("T3D2".to_string()),

            // Beam elements
            "CBAR" | "CBEAM" => Ok("B31".to_string()),

            // Shell elements
            "CQUAD4" => Ok("S4".to_string()),
            "CTRIA3" => Ok("S3".to_string()),

            // Solid elements
            "CHEXA" | "CHEXA8" => Ok("C3D8".to_string()),
            "CTETRA" | "CTETRA4" => Ok("C3D4".to_string()),
            "CPENTA" | "CPENTA6" => Ok("C3D6".to_string()),

            // Unsupported
            _ => Err(IoError::UnsupportedElement(
                format!("Nastran element type '{}' not supported", nastran_type)
            )),
        }
    }

    /// Get conversion statistics
    pub fn stats(&self) -> ConversionStats {
        ConversionStats {
            num_nodes_converted: self.node_map.len(),
            num_elements_converted: self.element_map.len(),
        }
    }
}

impl Default for BdfToInpConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ConversionStats {
    pub num_nodes_converted: usize,
    pub num_elements_converted: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_type_mapping() {
        let converter = BdfToInpConverter::new();

        assert_eq!(converter.map_element_type("CROD").unwrap(), "T3D2");
        assert_eq!(converter.map_element_type("CBAR").unwrap(), "B31");
        assert_eq!(converter.map_element_type("CQUAD4").unwrap(), "S4");
        assert_eq!(converter.map_element_type("CHEXA").unwrap(), "C3D8");

        assert!(converter.map_element_type("UNKNOWN").is_err());
    }

    #[test]
    fn test_simple_conversion() {
        let mut converter = BdfToInpConverter::new();

        let mut nodes = HashMap::new();
        nodes.insert(1, Node { id: 1, x: 0.0, y: 0.0, z: 0.0 });
        nodes.insert(2, Node { id: 2, x: 1.0, y: 0.0, z: 0.0 });

        let mut elements = HashMap::new();
        elements.insert(1, Element {
            id: 1,
            elem_type: "CROD".to_string(),
            nodes: vec![1, 2],
            property_id: 1,
        });

        let bdf_data = BdfData {
            nodes,
            elements,
            materials: HashMap::new(),
            properties: HashMap::new(),
        };

        let inp = converter.convert(&bdf_data).unwrap();

        assert!(inp.contains("*NODE"));
        assert!(inp.contains("*ELEMENT, TYPE=T3D2"));
        assert!(inp.contains("1, 0.000000e0, 0.000000e0, 0.000000e0"));

        let stats = converter.stats();
        assert_eq!(stats.num_nodes_converted, 2);
        assert_eq!(stats.num_elements_converted, 1);
    }
}
