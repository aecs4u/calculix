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

/// Beam section properties for various cross-section shapes
#[derive(Debug, Clone, PartialEq)]
pub struct BeamSection {
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
}
