/// Beam elements for CalculiX Rust solver
///
/// This module implements beam finite elements following Euler-Bernoulli beam theory:
/// - B31: 2-node 3D beam (linear)
/// - B32: 3-node 3D beam (quadratic) - TODO
///
/// Each node has 6 degrees of freedom:
/// - 3 translations (ux, uy, uz)
/// - 3 rotations (θx, θy, θz)
///
/// References:
/// - CalculiX documentation v2.23
/// - "Finite Element Procedures" by K.J. Bathe
/// - Cook et al., "Concepts and Applications of Finite Element Analysis"

use nalgebra::{DMatrix, SMatrix, Vector3};
use crate::elements::Element;
use crate::materials::Material;
use crate::mesh::Node;

/// Section shape definition for stress computation
#[derive(Debug, Clone, PartialEq)]
pub enum SectionShape {
    /// Rectangular section with width (y-direction) and height (z-direction)
    Rectangular { width: f64, height: f64 },
    /// Circular section with radius
    Circular { radius: f64 },
    /// Custom section (properties only, no stress computation)
    Custom,
}

/// Beam section properties for various cross-section shapes
#[derive(Debug, Clone, PartialEq)]
pub struct BeamSection {
    /// Section shape (for stress computation)
    pub shape: SectionShape,
    /// Cross-sectional area
    pub area: f64,
    /// Second moment of area about local y-axis (Iyy)
    pub iyy: f64,
    /// Second moment of area about local z-axis (Izz)
    pub izz: f64,
    /// Torsional constant (J)
    pub torsion_constant: f64,
    /// Shear area in y-direction (for Timoshenko beams)
    pub shear_area_y: Option<f64>,
    /// Shear area in z-direction (for Timoshenko beams)
    pub shear_area_z: Option<f64>,
}

impl BeamSection {
    /// Create a circular beam section
    ///
    /// # Arguments
    /// * `radius` - Radius of the circular cross-section
    ///
    /// # Returns
    /// BeamSection with properties for a circular cross-section
    ///
    /// # Example
    /// ```
    /// use ccx_solver::elements::beam::BeamSection;
    ///
    /// let section = BeamSection::circular(0.05); // 5cm radius
    /// assert!((section.area - std::f64::consts::PI * 0.05_f64.powi(2)).abs() < 1e-10);
    /// ```
    pub fn circular(radius: f64) -> Self {
        let area = std::f64::consts::PI * radius.powi(2);
        let i = std::f64::consts::PI * radius.powi(4) / 4.0;
        let j = std::f64::consts::PI * radius.powi(4) / 2.0;

        Self {
            shape: SectionShape::Circular { radius },
            area,
            iyy: i,
            izz: i,
            torsion_constant: j,
            shear_area_y: Some(area * 0.9), // Approximate for circular section
            shear_area_z: Some(area * 0.9),
        }
    }

    /// Create a rectangular beam section
    ///
    /// # Arguments
    /// * `width` - Width of the rectangle (in local y-direction)
    /// * `height` - Height of the rectangle (in local z-direction)
    ///
    /// # Returns
    /// BeamSection with properties for a rectangular cross-section
    pub fn rectangular(width: f64, height: f64) -> Self {
        let area = width * height;
        let iyy = width * height.powi(3) / 12.0;
        let izz = height * width.powi(3) / 12.0;

        // Torsional constant for rectangle (approximate formula)
        let a = width.max(height);
        let b = width.min(height);
        let j = (a * b.powi(3)) * (1.0 / 3.0 - 0.21 * (b / a) * (1.0 - b.powi(4) / (12.0 * a.powi(4))));

        Self {
            shape: SectionShape::Rectangular { width, height },
            area,
            iyy,
            izz,
            torsion_constant: j,
            shear_area_y: Some(5.0 / 6.0 * area),
            shear_area_z: Some(5.0 / 6.0 * area),
        }
    }

    /// Create a custom beam section with explicit properties
    pub fn custom(area: f64, iyy: f64, izz: f64, j: f64) -> Self {
        Self {
            shape: SectionShape::Custom,
            area,
            iyy,
            izz,
            torsion_constant: j,
            shear_area_y: None,
            shear_area_z: None,
        }
    }
}

/// B31 - 2-node 3D Euler-Bernoulli beam element
///
/// This element uses Euler-Bernoulli beam theory with the following assumptions:
/// - Plane sections remain plane and perpendicular to the neutral axis
/// - Shear deformation is neglected
/// - Linear elastic material behavior
///
/// Degrees of freedom per node: 6 (ux, uy, uz, θx, θy, θz)
/// Total DOFs: 12
#[derive(Debug, Clone)]
pub struct Beam31 {
    pub id: i32,
    pub nodes: Vec<i32>,
    pub section: BeamSection,
}

impl Beam31 {
    /// Create a new B31 beam element
    pub fn new(id: i32, node1: i32, node2: i32, section: BeamSection) -> Self {
        Self {
            id,
            nodes: vec![node1, node2],
            section,
        }
    }

    /// Calculate the length of the beam element
    fn length(&self, nodes: &[Node]) -> Result<f64, String> {
        if nodes.len() != 2 {
            return Err(format!("B31 element requires exactly 2 nodes, got {}", nodes.len()));
        }

        let dx = nodes[1].x - nodes[0].x;
        let dy = nodes[1].y - nodes[0].y;
        let dz = nodes[1].z - nodes[0].z;

        Ok((dx * dx + dy * dy + dz * dz).sqrt())
    }

    /// Compute the transformation matrix from local to global coordinates
    ///
    /// The local coordinate system is defined with:
    /// - x-axis along the beam axis (from node 1 to node 2)
    /// - y and z axes perpendicular to x-axis
    fn transformation_matrix(&self, nodes: &[Node]) -> Result<DMatrix<f64>, String> {
        if nodes.len() != 2 {
            return Err(format!("B31 element requires exactly 2 nodes, got {}", nodes.len()));
        }

        // Beam axis vector (local x-axis)
        let dx = nodes[1].x - nodes[0].x;
        let dy = nodes[1].y - nodes[0].y;
        let dz = nodes[1].z - nodes[0].z;
        let length = (dx * dx + dy * dy + dz * dz).sqrt();

        if length < 1e-12 {
            return Err("Beam element has zero length".to_string());
        }

        // Unit vector along beam axis
        let ex = Vector3::new(dx / length, dy / length, dz / length);

        // Define local y and z axes
        // Choose a reference vector not parallel to the beam axis
        let reference = if ex.x.abs() < 0.9 {
            Vector3::new(1.0, 0.0, 0.0)
        } else {
            Vector3::new(0.0, 1.0, 0.0)
        };

        // Local z-axis perpendicular to beam and reference vector
        let ez = ex.cross(&reference).normalize();

        // Local y-axis completes the right-handed system
        let ey = ez.cross(&ex);

        // Build 3x3 rotation matrix
        let mut r = DMatrix::zeros(3, 3);
        for i in 0..3 {
            r[(0, i)] = ex[i];
            r[(1, i)] = ey[i];
            r[(2, i)] = ez[i];
        }

        // Expand to 12x12 transformation matrix for 6 DOFs per node
        let mut t = DMatrix::zeros(12, 12);
        for i in 0..4 {
            let row_offset = i * 3;
            for ii in 0..3 {
                for jj in 0..3 {
                    t[(row_offset + ii, row_offset + jj)] = r[(ii, jj)];
                }
            }
        }

        Ok(t)
    }

    /// Compute the local stiffness matrix (12x12) in the local coordinate system
    ///
    /// The local stiffness matrix combines:
    /// - Axial stiffness
    /// - Bending stiffness (in two planes)
    /// - Torsional stiffness
    fn local_stiffness(&self, length: f64, material: &Material) -> Result<SMatrix<f64, 12, 12>, String> {
        let e = material.elastic_modulus
            .ok_or("Material missing elastic modulus")?;
        let g = material.shear_modulus()
            .ok_or("Material missing shear modulus (requires E and ν)")?;
        let a = self.section.area;
        let iyy = self.section.iyy;
        let izz = self.section.izz;
        let j = self.section.torsion_constant;
        let l = length;

        // Initialize 12x12 stiffness matrix
        let mut k = SMatrix::<f64, 12, 12>::zeros();

        // Axial stiffness (DOFs 0, 6)
        let k_axial = e * a / l;
        k[(0, 0)] = k_axial;
        k[(0, 6)] = -k_axial;
        k[(6, 0)] = -k_axial;
        k[(6, 6)] = k_axial;

        // Bending in XY plane (DOFs 1, 5, 7, 11)
        // Uses Iyy (second moment about y-axis)
        let k_bend_y = 12.0 * e * izz / l.powi(3);
        let k_rot_y = 6.0 * e * izz / l.powi(2);
        let k_rot_rot_y = 4.0 * e * izz / l;
        let k_rot_rot_y2 = 2.0 * e * izz / l;

        k[(1, 1)] = k_bend_y;
        k[(1, 5)] = k_rot_y;
        k[(1, 7)] = -k_bend_y;
        k[(1, 11)] = k_rot_y;

        k[(5, 1)] = k_rot_y;
        k[(5, 5)] = k_rot_rot_y;
        k[(5, 7)] = -k_rot_y;
        k[(5, 11)] = k_rot_rot_y2;

        k[(7, 1)] = -k_bend_y;
        k[(7, 5)] = -k_rot_y;
        k[(7, 7)] = k_bend_y;
        k[(7, 11)] = -k_rot_y;

        k[(11, 1)] = k_rot_y;
        k[(11, 5)] = k_rot_rot_y2;
        k[(11, 7)] = -k_rot_y;
        k[(11, 11)] = k_rot_rot_y;

        // Bending in XZ plane (DOFs 2, 4, 8, 10)
        // Uses Izz (second moment about z-axis)
        let k_bend_z = 12.0 * e * iyy / l.powi(3);
        let k_rot_z = 6.0 * e * iyy / l.powi(2);
        let k_rot_rot_z = 4.0 * e * iyy / l;
        let k_rot_rot_z2 = 2.0 * e * iyy / l;

        k[(2, 2)] = k_bend_z;
        k[(2, 4)] = -k_rot_z;
        k[(2, 8)] = -k_bend_z;
        k[(2, 10)] = -k_rot_z;

        k[(4, 2)] = -k_rot_z;
        k[(4, 4)] = k_rot_rot_z;
        k[(4, 8)] = k_rot_z;
        k[(4, 10)] = k_rot_rot_z2;

        k[(8, 2)] = -k_bend_z;
        k[(8, 4)] = k_rot_z;
        k[(8, 8)] = k_bend_z;
        k[(8, 10)] = k_rot_z;

        k[(10, 2)] = -k_rot_z;
        k[(10, 4)] = k_rot_rot_z2;
        k[(10, 8)] = k_rot_z;
        k[(10, 10)] = k_rot_rot_z;

        // Torsional stiffness (DOFs 3, 9)
        let k_torsion = g * j / l;
        k[(3, 3)] = k_torsion;
        k[(3, 9)] = -k_torsion;
        k[(9, 3)] = -k_torsion;
        k[(9, 9)] = k_torsion;

        Ok(k)
    }

    /// Compute the local mass matrix (12x12) in the local coordinate system
    ///
    /// The consistent mass matrix for Euler-Bernoulli beam combines:
    /// - Axial mass (translational)
    /// - Bending mass (translational + rotational coupling)
    /// - Torsional mass (rotational)
    ///
    /// # Theory
    /// The consistent mass matrix is derived from:
    /// M = ∫ ρ * N^T * N dx
    ///
    /// where N are the shape functions and ρ is the material density.
    ///
    /// # DOF Ordering (local coordinates)
    /// - DOF 0, 6: Axial displacement (ux)
    /// - DOF 1, 7: Transverse displacement y (uy)
    /// - DOF 2, 8: Transverse displacement z (uz)
    /// - DOF 3, 9: Torsion (θx)
    /// - DOF 4, 10: Bending rotation about y (θy)
    /// - DOF 5, 11: Bending rotation about z (θz)
    fn local_mass(&self, length: f64, material: &Material) -> Result<SMatrix<f64, 12, 12>, String> {
        let rho = material.density
            .ok_or("Material missing density (required for mass matrix)")?;
        let a = self.section.area;
        let iyy = self.section.iyy;
        let izz = self.section.izz;
        let l = length;

        // Initialize 12x12 mass matrix
        let mut m = SMatrix::<f64, 12, 12>::zeros();

        // Axial mass (DOFs 0, 6) - consistent mass formulation
        let m_axial = rho * a * l / 6.0;
        m[(0, 0)] = 2.0 * m_axial;
        m[(0, 6)] = m_axial;
        m[(6, 0)] = m_axial;
        m[(6, 6)] = 2.0 * m_axial;

        // Torsional mass (DOFs 3, 9) - using polar moment of inertia
        let ip = iyy + izz; // Polar moment of inertia
        let m_torsion = rho * ip * l / 6.0;
        m[(3, 3)] = 2.0 * m_torsion;
        m[(3, 9)] = m_torsion;
        m[(9, 3)] = m_torsion;
        m[(9, 9)] = 2.0 * m_torsion;

        // Bending in XY plane (DOFs 1, 5, 7, 11)
        // Consistent mass matrix with translational-rotational coupling
        let m_coeff = rho * a * l / 420.0;

        // Translational DOFs (1, 7)
        m[(1, 1)] = 156.0 * m_coeff;
        m[(1, 5)] = 22.0 * m_coeff * l;
        m[(1, 7)] = 54.0 * m_coeff;
        m[(1, 11)] = -13.0 * m_coeff * l;

        m[(7, 1)] = 54.0 * m_coeff;
        m[(7, 5)] = 13.0 * m_coeff * l;
        m[(7, 7)] = 156.0 * m_coeff;
        m[(7, 11)] = -22.0 * m_coeff * l;

        // Rotational DOFs (5, 11)
        m[(5, 1)] = 22.0 * m_coeff * l;
        m[(5, 5)] = 4.0 * m_coeff * l * l;
        m[(5, 7)] = 13.0 * m_coeff * l;
        m[(5, 11)] = -3.0 * m_coeff * l * l;

        m[(11, 1)] = -13.0 * m_coeff * l;
        m[(11, 5)] = -3.0 * m_coeff * l * l;
        m[(11, 7)] = -22.0 * m_coeff * l;
        m[(11, 11)] = 4.0 * m_coeff * l * l;

        // Bending in XZ plane (DOFs 2, 4, 8, 10)
        // Same pattern as XY plane, but with negative signs for θy
        m[(2, 2)] = 156.0 * m_coeff;
        m[(2, 4)] = -22.0 * m_coeff * l;
        m[(2, 8)] = 54.0 * m_coeff;
        m[(2, 10)] = 13.0 * m_coeff * l;

        m[(8, 2)] = 54.0 * m_coeff;
        m[(8, 4)] = -13.0 * m_coeff * l;
        m[(8, 8)] = 156.0 * m_coeff;
        m[(8, 10)] = 22.0 * m_coeff * l;

        m[(4, 2)] = -22.0 * m_coeff * l;
        m[(4, 4)] = 4.0 * m_coeff * l * l;
        m[(4, 8)] = -13.0 * m_coeff * l;
        m[(4, 10)] = -3.0 * m_coeff * l * l;

        m[(10, 2)] = 13.0 * m_coeff * l;
        m[(10, 4)] = -3.0 * m_coeff * l * l;
        m[(10, 8)] = 22.0 * m_coeff * l;
        m[(10, 10)] = 4.0 * m_coeff * l * l;

        Ok(m)
    }
}

impl Element for Beam31 {
    fn stiffness_matrix(&self, nodes: &[Node], material: &Material) -> Result<DMatrix<f64>, String> {
        let length = self.length(nodes)?;
        let k_local = self.local_stiffness(length, material)?;
        let t = self.transformation_matrix(nodes)?;

        // Transform to global coordinates: K_global = T^T * K_local * T
        Ok(&t.transpose() * k_local * &t)
    }

    fn num_nodes(&self) -> usize {
        2
    }

    fn dofs_per_node(&self) -> usize {
        6 // 3 translations + 3 rotations
    }

    fn global_dof_indices(&self, connectivity: &[i32]) -> Vec<usize> {
        let mut indices = Vec::with_capacity(12);
        for &node_id in connectivity {
            let base = (node_id as usize) * 6;
            for offset in 0..6 {
                indices.push(base + offset);
            }
        }
        indices
    }

    fn mass_matrix(
        &self,
        nodes: &[Node],
        material: &Material,
    ) -> Result<DMatrix<f64>, String> {
        // Compute element length
        let length = self.length(nodes)?;

        // Get local mass matrix (12×12)
        let m_local = self.local_mass(length, material)?;

        // Get transformation matrix (12×12)
        let t = self.transformation_matrix(nodes)?;

        // Transform to global coordinates: M_global = T^T * M_local * T
        let m_global = &t.transpose() * m_local * &t;

        Ok(m_global)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::MaterialModel;

    #[test]
    fn test_circular_section() {
        let radius = 0.05;
        let section = BeamSection::circular(radius);

        let expected_area = std::f64::consts::PI * radius.powi(2);
        let expected_i = std::f64::consts::PI * radius.powi(4) / 4.0;
        let expected_j = std::f64::consts::PI * radius.powi(4) / 2.0;

        assert!((section.area - expected_area).abs() < 1e-10);
        assert!((section.iyy - expected_i).abs() < 1e-10);
        assert!((section.izz - expected_i).abs() < 1e-10);
        assert!((section.torsion_constant - expected_j).abs() < 1e-10);
    }

    #[test]
    fn test_rectangular_section() {
        let width = 0.1;
        let height = 0.2;
        let section = BeamSection::rectangular(width, height);

        let expected_area = width * height;
        let expected_iyy = width * height.powi(3) / 12.0;
        let expected_izz = height * width.powi(3) / 12.0;

        assert_eq!(section.area, expected_area);
        assert_eq!(section.iyy, expected_iyy);
        assert_eq!(section.izz, expected_izz);
        assert!(section.torsion_constant > 0.0);
    }

    #[test]
    fn test_beam31_creation() {
        let section = BeamSection::circular(0.05);
        let beam = Beam31::new(1, 0, 1, section);

        assert_eq!(beam.id, 1);
        assert_eq!(beam.nodes.len(), 2);
        assert_eq!(beam.num_nodes(), 2);
        assert_eq!(beam.dofs_per_node(), 6);
    }

    #[test]
    fn test_beam31_length() {
        let section = BeamSection::circular(0.05);
        let beam = Beam31::new(1, 0, 1, section);

        let nodes = vec![
            Node::new(0, 0.0, 0.0, 0.0),
            Node::new(1, 1.0, 0.0, 0.0),
        ];

        let length = beam.length(&nodes).unwrap();
        assert!((length - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_beam31_global_dof_indices() {
        let section = BeamSection::circular(0.05);
        let beam = Beam31::new(1, 5, 10, section);

        let indices = beam.global_dof_indices(&[5, 10]);
        assert_eq!(indices.len(), 12);

        // Node 5: DOFs 30-35
        assert_eq!(indices[0], 30);
        assert_eq!(indices[5], 35);

        // Node 10: DOFs 60-65
        assert_eq!(indices[6], 60);
        assert_eq!(indices[11], 65);
    }

    #[test]
    fn test_beam31_axial_stiffness() {
        // Simple axial test - beam along x-axis
        let section = BeamSection::custom(0.01, 1e-6, 1e-6, 1e-6); // 1 cm^2 area
        let beam = Beam31::new(1, 0, 1, section);

        let nodes = vec![
            Node::new(0, 0.0, 0.0, 0.0),
            Node::new(1, 1.0, 0.0, 0.0),
        ];

        let material = Material {
            name: "Steel".to_string(),
            model: MaterialModel::LinearElastic,
            elastic_modulus: Some(200e9), // 200 GPa
            poissons_ratio: Some(0.3),
            density: None,
            thermal_expansion: None,
            conductivity: None,
            specific_heat: None,
        };

        let k = beam.stiffness_matrix(&nodes, &material).unwrap();

        // Check stiffness matrix dimensions
        assert_eq!(k.nrows(), 12);
        assert_eq!(k.ncols(), 12);

        // Axial stiffness should be E*A/L
        let expected_axial = 200e9 * 0.01 / 1.0;

        // Check axial DOFs (0 and 6 are axial for beam along x-axis)
        assert!((k[(0, 0)] - expected_axial).abs() / expected_axial < 1e-6);
        assert!((k[(0, 6)] + expected_axial).abs() / expected_axial < 1e-6);
    }

    #[test]
    fn test_transformation_matrix_dimensions() {
        let section = BeamSection::circular(0.05);
        let beam = Beam31::new(1, 0, 1, section);

        let nodes = vec![
            Node::new(0, 0.0, 0.0, 0.0),
            Node::new(1, 1.0, 2.0, 3.0),
        ];

        let t = beam.transformation_matrix(&nodes).unwrap();
        assert_eq!(t.nrows(), 12);
        assert_eq!(t.ncols(), 12);
    }

    // ========== Mass Matrix Tests ==========

    fn make_material_with_density() -> Material {
        Material {
            name: "STEEL".to_string(),
            model: crate::materials::MaterialModel::LinearElastic,
            elastic_modulus: Some(200e9), // Pa
            poissons_ratio: Some(0.3),
            density: Some(7850.0), // kg/m³
            thermal_expansion: None,
            conductivity: None,
            specific_heat: None,
        }
    }

    #[test]
    fn mass_matrix_requires_density() {
        let section = BeamSection::circular(0.05);
        let beam = Beam31::new(1, 0, 1, section);
        let nodes = vec![
            Node::new(0, 0.0, 0.0, 0.0),
            Node::new(1, 1.0, 0.0, 0.0),
        ];

        let material = Material {
            name: "NO_DENSITY".to_string(),
            model: crate::materials::MaterialModel::LinearElastic,
            elastic_modulus: Some(200e9),
            poissons_ratio: Some(0.3),
            density: None, // Missing density
            thermal_expansion: None,
            conductivity: None,
            specific_heat: None,
        };

        let result = beam.mass_matrix(&nodes, &material);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("density"));
    }

    #[test]
    fn mass_matrix_is_symmetric() {
        let section = BeamSection::circular(0.05);
        let beam = Beam31::new(1, 0, 1, section);
        let nodes = vec![
            Node::new(0, 0.0, 0.0, 0.0),
            Node::new(1, 1.0, 0.0, 0.0),
        ];
        let material = make_material_with_density();

        let m = beam.mass_matrix(&nodes, &material).unwrap();

        // Check symmetry: M[i,j] == M[j,i]
        for i in 0..12 {
            for j in 0..12 {
                let error = (m[(i, j)] - m[(j, i)]).abs();
                assert!(
                    error < 1e-10,
                    "Mass matrix not symmetric at ({}, {}): {} vs {}",
                    i,
                    j,
                    m[(i, j)],
                    m[(j, i)]
                );
            }
        }
    }

    #[test]
    fn mass_matrix_has_positive_diagonals() {
        let section = BeamSection::circular(0.05);
        let beam = Beam31::new(1, 0, 1, section);
        let nodes = vec![
            Node::new(0, 0.0, 0.0, 0.0),
            Node::new(1, 1.0, 0.0, 0.0),
        ];
        let material = make_material_with_density();

        let m = beam.mass_matrix(&nodes, &material).unwrap();

        // Check all diagonal entries are positive
        for i in 0..12 {
            assert!(
                m[(i, i)] > 0.0,
                "Diagonal entry M[{}, {}] = {} should be positive",
                i,
                i,
                m[(i, i)]
            );
        }
    }

    #[test]
    fn mass_matrix_conserves_total_mass() {
        // Test: Total translational mass should be consistent with ρ*A*L
        let radius = 0.05; // m
        let length = 2.0; // m
        let section = BeamSection::circular(radius);
        let beam = Beam31::new(1, 0, 1, section);
        let nodes = vec![
            Node::new(0, 0.0, 0.0, 0.0),
            Node::new(1, length, 0.0, 0.0),
        ];
        let material = make_material_with_density();

        let m = beam.mass_matrix(&nodes, &material).unwrap();

        // For a beam along x-axis, extract axial mass (DOFs 0, 6)
        // The sum of entries in the axial DOF rows should equal total axial mass
        let axial_mass_matrix_sum: f64 = m[(0, 0)] + m[(0, 6)] + m[(6, 0)] + m[(6, 6)];

        // Expected: (ρ*A*L/6) * (2 + 1 + 1 + 2) = ρ*A*L
        let expected_axial_mass = material.density.unwrap() * beam.section.area * length;

        let error = (axial_mass_matrix_sum - expected_axial_mass).abs();
        let relative_error = error / expected_axial_mass;

        assert!(
            relative_error < 1e-10,
            "Axial mass conservation error: {:.2e}% (expected 0%)",
            relative_error * 100.0
        );
    }

    #[test]
    fn mass_matrix_axial_component() {
        // Test specific values for axial mass
        // Element: 1m long, area=1m², density=1000 kg/m³
        let section = BeamSection::custom(1.0, 0.0001, 0.0001, 0.0001);
        let beam = Beam31::new(1, 0, 1, section);
        let nodes = vec![
            Node::new(0, 0.0, 0.0, 0.0),
            Node::new(1, 1.0, 0.0, 0.0),
        ];

        let mut material = make_material_with_density();
        material.density = Some(1000.0); // kg/m³

        let m = beam.mass_matrix(&nodes, &material).unwrap();

        // Axial mass coefficient: ρ*A*L/6 = 1000*1*1/6 = 166.67
        let m_axial = 1000.0 * 1.0 * 1.0 / 6.0;

        // For horizontal beam, axial DOFs are 0 and 6
        assert!(
            (m[(0, 0)] - 2.0 * m_axial).abs() < 1e-6,
            "M[0,0] = {} should be ~{}",
            m[(0, 0)],
            2.0 * m_axial
        );
        assert!(
            (m[(0, 6)] - m_axial).abs() < 1e-6,
            "M[0,6] = {} should be ~{}",
            m[(0, 6)],
            m_axial
        );
        assert!(
            (m[(6, 6)] - 2.0 * m_axial).abs() < 1e-6,
            "M[6,6] = {} should be ~{}",
            m[(6, 6)],
            2.0 * m_axial
        );
    }

    #[test]
    fn mass_matrix_dimensions() {
        let section = BeamSection::circular(0.05);
        let beam = Beam31::new(1, 0, 1, section);
        let nodes = vec![
            Node::new(0, 0.0, 0.0, 0.0),
            Node::new(1, 1.0, 0.0, 0.0),
        ];
        let material = make_material_with_density();

        let m = beam.mass_matrix(&nodes, &material).unwrap();

        assert_eq!(m.nrows(), 12, "Mass matrix should be 12×12");
        assert_eq!(m.ncols(), 12, "Mass matrix should be 12×12");
    }

    #[test]
    fn mass_matrix_bending_components_nonzero() {
        // Test that bending DOFs have non-zero mass
        let section = BeamSection::circular(0.05);
        let beam = Beam31::new(1, 0, 1, section);
        let nodes = vec![
            Node::new(0, 0.0, 0.0, 0.0),
            Node::new(1, 1.0, 0.0, 0.0),
        ];
        let material = make_material_with_density();

        let m = beam.mass_matrix(&nodes, &material).unwrap();

        // For a beam along x-axis:
        // - Transverse DOFs (1, 2, 7, 8) should have mass
        // - Rotational DOFs (3, 4, 5, 9, 10, 11) should have mass

        // Check transverse y (DOF 1, 7)
        assert!(m[(1, 1)] > 0.0, "Transverse y mass should be positive");
        assert!(m[(7, 7)] > 0.0, "Transverse y mass should be positive");

        // Check transverse z (DOF 2, 8)
        assert!(m[(2, 2)] > 0.0, "Transverse z mass should be positive");
        assert!(m[(8, 8)] > 0.0, "Transverse z mass should be positive");

        // Check rotational DOFs
        for dof in [3, 4, 5, 9, 10, 11] {
            assert!(
                m[(dof, dof)] > 0.0,
                "Rotational DOF {} should have positive mass",
                dof
            );
        }
    }
}
