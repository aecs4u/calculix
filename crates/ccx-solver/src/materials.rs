//! Material properties for finite element analysis.

use ccx_io::inp::{Card, Deck};
use std::collections::HashMap;

/// Material model type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaterialModel {
    /// Linear elastic isotropic
    #[default]
    LinearElastic,
    /// Plastic (elasto-plastic)
    Plastic,
    /// Hyperelastic (rubber-like)
    Hyperelastic,
    /// Viscoplastic
    Viscoplastic,
}

/// A material definition
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Material {
    /// Material name
    pub name: String,
    /// Material model type
    pub model: MaterialModel,
    /// Young's modulus (E) [Pa]
    pub elastic_modulus: Option<f64>,
    /// Poisson's ratio (ν) [-]
    pub poissons_ratio: Option<f64>,
    /// Density (ρ) [kg/m³]
    pub density: Option<f64>,
    /// Thermal expansion coefficient [1/K]
    pub thermal_expansion: Option<f64>,
    /// Thermal conductivity [W/(m·K)]
    pub conductivity: Option<f64>,
    /// Specific heat [J/(kg·K)]
    pub specific_heat: Option<f64>,
}

impl Material {
    /// Create a new material with a given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            model: MaterialModel::LinearElastic,
            elastic_modulus: None,
            poissons_ratio: None,
            density: None,
            thermal_expansion: None,
            conductivity: None,
            specific_heat: None,
        }
    }

    /// Check if material has minimum required properties for structural analysis
    pub fn is_valid_for_structural(&self) -> bool {
        self.elastic_modulus.is_some() && self.poissons_ratio.is_some()
    }

    /// Get the shear modulus (G) from E and ν
    pub fn shear_modulus(&self) -> Option<f64> {
        match (self.elastic_modulus, self.poissons_ratio) {
            (Some(e), Some(nu)) => Some(e / (2.0 * (1.0 + nu))),
            _ => None,
        }
    }

    /// Get the bulk modulus (K) from E and ν
    pub fn bulk_modulus(&self) -> Option<f64> {
        match (self.elastic_modulus, self.poissons_ratio) {
            (Some(e), Some(nu)) => Some(e / (3.0 * (1.0 - 2.0 * nu))),
            _ => None,
        }
    }
}

/// Material library containing all materials and their assignments
#[derive(Debug, Clone)]
pub struct MaterialLibrary {
    /// All materials by name
    materials: HashMap<String, Material>,
    /// Element-to-material assignments (element_id -> material_name)
    element_materials: HashMap<i32, String>,
}

impl MaterialLibrary {
    /// Create an empty material library
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
            element_materials: HashMap::new(),
        }
    }

    /// Add a material to the library
    pub fn add_material(&mut self, material: Material) {
        self.materials.insert(material.name.clone(), material);
    }

    /// Get a material by name
    pub fn get_material(&self, name: &str) -> Option<&Material> {
        self.materials.get(name)
    }

    /// Get all material names
    pub fn material_names(&self) -> Vec<String> {
        self.materials.keys().cloned().collect()
    }

    /// Assign a material to an element
    pub fn assign_material(&mut self, element_id: i32, material_name: String) {
        self.element_materials.insert(element_id, material_name);
    }

    /// Get the material for an element
    pub fn get_element_material(&self, element_id: i32) -> Option<&Material> {
        self.element_materials
            .get(&element_id)
            .and_then(|name| self.materials.get(name))
    }

    /// Build material library from a deck
    pub fn build_from_deck(deck: &Deck) -> Result<Self, String> {
        let mut library = Self::new();
        let mut current_material: Option<String> = None;

        for card in &deck.cards {
            match card.keyword.to_uppercase().as_str() {
                "MATERIAL" => {
                    let mat = Self::parse_material(card)?;
                    current_material = Some(mat.name.clone());
                    library.add_material(mat);
                }
                "ELASTIC" => {
                    if let Some(ref mat_name) = current_material {
                        Self::parse_elastic(card, &mut library, mat_name)?;
                    }
                }
                "DENSITY" => {
                    if let Some(ref mat_name) = current_material {
                        Self::parse_density(card, &mut library, mat_name)?;
                    }
                }
                "EXPANSION" => {
                    if let Some(ref mat_name) = current_material {
                        Self::parse_expansion(card, &mut library, mat_name)?;
                    }
                }
                "CONDUCTIVITY" => {
                    if let Some(ref mat_name) = current_material {
                        Self::parse_conductivity(card, &mut library, mat_name)?;
                    }
                }
                "SPECIFIC HEAT" => {
                    if let Some(ref mat_name) = current_material {
                        Self::parse_specific_heat(card, &mut library, mat_name)?;
                    }
                }
                _ => {}
            }
        }

        Ok(library)
    }

    /// Parse a *MATERIAL card
    fn parse_material(card: &Card) -> Result<Material, String> {
        let name_param = card
            .parameters
            .iter()
            .find(|p| p.key.to_uppercase() == "NAME");

        let name = match name_param {
            Some(p) => match &p.value {
                Some(v) => v.clone(),
                None => return Err("MATERIAL parameter missing NAME value".to_string()),
            },
            None => return Err("MATERIAL card missing NAME parameter".to_string()),
        };

        Ok(Material::new(name))
    }

    /// Parse an *ELASTIC card (isotropic)
    fn parse_elastic(
        card: &Card,
        library: &mut MaterialLibrary,
        material_name: &str,
    ) -> Result<(), String> {
        if card.data_lines.is_empty() {
            return Err("ELASTIC card has no data lines".to_string());
        }

        let line = &card.data_lines[0];
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() < 2 {
            return Err(format!(
                "ELASTIC data line needs at least 2 values: {}",
                line
            ));
        }

        let e = parts[0]
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("Invalid elastic modulus: {}", parts[0].trim()))?;

        let nu = parts[1]
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("Invalid Poisson's ratio: {}", parts[1].trim()))?;

        if let Some(material) = library.materials.get_mut(material_name) {
            material.elastic_modulus = Some(e);
            material.poissons_ratio = Some(nu);
        }

        Ok(())
    }

    /// Parse a *DENSITY card
    fn parse_density(
        card: &Card,
        library: &mut MaterialLibrary,
        material_name: &str,
    ) -> Result<(), String> {
        if card.data_lines.is_empty() {
            return Err("DENSITY card has no data lines".to_string());
        }

        let line = &card.data_lines[0];
        let density = line
            .trim()
            .split(',')
            .next()
            .ok_or("DENSITY data line is empty")?
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("Invalid density value: {}", line.trim()))?;

        if let Some(material) = library.materials.get_mut(material_name) {
            material.density = Some(density);
        }

        Ok(())
    }

    /// Parse an *EXPANSION card
    fn parse_expansion(
        card: &Card,
        library: &mut MaterialLibrary,
        material_name: &str,
    ) -> Result<(), String> {
        if card.data_lines.is_empty() {
            return Err("EXPANSION card has no data lines".to_string());
        }

        let line = &card.data_lines[0];
        let alpha = line
            .trim()
            .split(',')
            .next()
            .ok_or("EXPANSION data line is empty")?
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("Invalid thermal expansion value: {}", line.trim()))?;

        if let Some(material) = library.materials.get_mut(material_name) {
            material.thermal_expansion = Some(alpha);
        }

        Ok(())
    }

    /// Parse a *CONDUCTIVITY card
    fn parse_conductivity(
        card: &Card,
        library: &mut MaterialLibrary,
        material_name: &str,
    ) -> Result<(), String> {
        if card.data_lines.is_empty() {
            return Err("CONDUCTIVITY card has no data lines".to_string());
        }

        let line = &card.data_lines[0];
        let k = line
            .trim()
            .split(',')
            .next()
            .ok_or("CONDUCTIVITY data line is empty")?
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("Invalid conductivity value: {}", line.trim()))?;

        if let Some(material) = library.materials.get_mut(material_name) {
            material.conductivity = Some(k);
        }

        Ok(())
    }

    /// Parse a *SPECIFIC HEAT card
    fn parse_specific_heat(
        card: &Card,
        library: &mut MaterialLibrary,
        material_name: &str,
    ) -> Result<(), String> {
        if card.data_lines.is_empty() {
            return Err("SPECIFIC HEAT card has no data lines".to_string());
        }

        let line = &card.data_lines[0];
        let cp = line
            .trim()
            .split(',')
            .next()
            .ok_or("SPECIFIC HEAT data line is empty")?
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("Invalid specific heat value: {}", line.trim()))?;

        if let Some(material) = library.materials.get_mut(material_name) {
            material.specific_heat = Some(cp);
        }

        Ok(())
    }

    /// Get statistics
    pub fn statistics(&self) -> MaterialStatistics {
        let valid_materials = self
            .materials
            .values()
            .filter(|m| m.is_valid_for_structural())
            .count();

        MaterialStatistics {
            num_materials: self.materials.len(),
            num_valid_structural: valid_materials,
            num_element_assignments: self.element_materials.len(),
        }
    }
}

impl Default for MaterialLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Material library statistics
#[derive(Debug, Clone)]
pub struct MaterialStatistics {
    /// Total number of materials defined
    pub num_materials: usize,
    /// Number of materials valid for structural analysis
    pub num_valid_structural: usize,
    /// Number of element-material assignments
    pub num_element_assignments: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_deck(input: &str) -> Deck {
        Deck::parse_str(input).expect("Failed to parse deck")
    }

    #[test]
    fn parses_simple_material() {
        let input = r#"
*MATERIAL, NAME=STEEL
*ELASTIC
210000, 0.3
"#;

        let deck = parse_deck(input);
        let library = MaterialLibrary::build_from_deck(&deck).expect("Failed to build library");

        assert_eq!(library.materials.len(), 1);
        let steel = library.get_material("STEEL").unwrap();
        assert_eq!(steel.elastic_modulus, Some(210000.0));
        assert_eq!(steel.poissons_ratio, Some(0.3));
    }

    #[test]
    fn parses_material_with_density() {
        let input = r#"
*MATERIAL, NAME=ALUMINUM
*ELASTIC
70000, 0.33
*DENSITY
2700
"#;

        let deck = parse_deck(input);
        let library = MaterialLibrary::build_from_deck(&deck).expect("Failed to build library");

        let al = library.get_material("ALUMINUM").unwrap();
        assert_eq!(al.elastic_modulus, Some(70000.0));
        assert_eq!(al.poissons_ratio, Some(0.33));
        assert_eq!(al.density, Some(2700.0));
    }

    #[test]
    fn calculates_shear_modulus() {
        let mut mat = Material::new("TEST".to_string());
        mat.elastic_modulus = Some(210000.0);
        mat.poissons_ratio = Some(0.3);

        let g = mat.shear_modulus().unwrap();
        assert!((g - 80769.23).abs() < 0.01);
    }

    #[test]
    fn calculates_bulk_modulus() {
        let mut mat = Material::new("TEST".to_string());
        mat.elastic_modulus = Some(210000.0);
        mat.poissons_ratio = Some(0.3);

        let k = mat.bulk_modulus().unwrap();
        assert!((k - 175000.0).abs() < 0.01);
    }

    #[test]
    fn validates_material_for_structural() {
        let mut mat = Material::new("TEST".to_string());
        assert!(!mat.is_valid_for_structural());

        mat.elastic_modulus = Some(210000.0);
        assert!(!mat.is_valid_for_structural());

        mat.poissons_ratio = Some(0.3);
        assert!(mat.is_valid_for_structural());
    }

    #[test]
    fn handles_multiple_materials() {
        let input = r#"
*MATERIAL, NAME=STEEL
*ELASTIC
210000, 0.3
*MATERIAL, NAME=CONCRETE
*ELASTIC
30000, 0.2
"#;

        let deck = parse_deck(input);
        let library = MaterialLibrary::build_from_deck(&deck).expect("Failed to build library");

        assert_eq!(library.materials.len(), 2);
        assert!(library.get_material("STEEL").is_some());
        assert!(library.get_material("CONCRETE").is_some());
    }

    #[test]
    fn parses_thermal_properties() {
        let input = r#"
*MATERIAL, NAME=STEEL
*ELASTIC
210000, 0.3
*EXPANSION
1.2e-5
*CONDUCTIVITY
50.0
*SPECIFIC HEAT
450.0
"#;

        let deck = parse_deck(input);
        let library = MaterialLibrary::build_from_deck(&deck).expect("Failed to build library");

        let steel = library.get_material("STEEL").unwrap();
        assert_eq!(steel.thermal_expansion, Some(1.2e-5));
        assert_eq!(steel.conductivity, Some(50.0));
        assert_eq!(steel.specific_heat, Some(450.0));
    }

    #[test]
    fn element_material_assignment() {
        let mut library = MaterialLibrary::new();
        let mut steel = Material::new("STEEL".to_string());
        steel.elastic_modulus = Some(210000.0);
        steel.poissons_ratio = Some(0.3);
        library.add_material(steel);

        library.assign_material(1, "STEEL".to_string());
        library.assign_material(2, "STEEL".to_string());

        let mat1 = library.get_element_material(1).unwrap();
        assert_eq!(mat1.name, "STEEL");

        assert!(library.get_element_material(999).is_none());
    }

    #[test]
    fn statistics() {
        let input = r#"
*MATERIAL, NAME=STEEL
*ELASTIC
210000, 0.3
*MATERIAL, NAME=INCOMPLETE
"#;

        let deck = parse_deck(input);
        let library = MaterialLibrary::build_from_deck(&deck).expect("Failed to build library");

        let stats = library.statistics();
        assert_eq!(stats.num_materials, 2);
        assert_eq!(stats.num_valid_structural, 1);
    }

    #[test]
    fn rejects_material_without_name() {
        let input = r#"
*MATERIAL
*ELASTIC
210000, 0.3
"#;

        let deck = parse_deck(input);
        let result = MaterialLibrary::build_from_deck(&deck);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("NAME"));
    }
}
