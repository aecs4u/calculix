//! Frequency (modal) analysis for extracting natural frequencies and mode shapes.
//!
//! This module implements eigenvalue analysis to compute:
//! - Natural frequencies (ω = √λ)
//! - Mode shapes (eigenvectors)
//! - Participation factors
//!
//! Solves the generalized eigenvalue problem:
//! ```text
//! K * φ = ω² * M * φ
//! ```
//! where:
//! - K: Stiffness matrix
//! - M: Mass matrix
//! - φ: Eigenvector (mode shape)
//! - ω: Angular frequency (rad/s)
//! - λ = ω²: Eigenvalue

use nalgebra::DMatrix;
use std::collections::HashMap;

use crate::assembly::GlobalSystem;
use crate::backend::{default_backend, EigenResult, EigenSystemData, SparseTripletsF64};
use crate::boundary_conditions::BoundaryConditions;
use crate::materials::Material;
use crate::mesh::Mesh;

/// Configuration for frequency analysis
#[derive(Debug, Clone)]
pub struct FrequencyConfig {
    /// Number of modes to extract
    pub num_modes: usize,
    /// Which eigenvalues to compute
    pub which: WhichEigenvalues,
    /// Convergence tolerance
    pub tolerance: f64,
    /// Maximum iterations
    pub max_iterations: usize,
    /// Use shift-invert for better convergence
    pub use_shift: bool,
    /// Shift value (for shift-invert)
    pub shift_value: Option<f64>,
}

impl Default for FrequencyConfig {
    fn default() -> Self {
        Self {
            num_modes: 10,
            which: WhichEigenvalues::SmallestMagnitude,
            tolerance: 1e-6,
            max_iterations: 1000,
            use_shift: false,
            shift_value: None,
        }
    }
}

/// Which eigenvalues to compute
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichEigenvalues {
    /// Smallest magnitude (lowest frequencies)
    SmallestMagnitude,
    /// Largest magnitude (highest frequencies)
    LargestMagnitude,
    /// Target eigenvalue (around a specific frequency)
    Target,
}

/// Results from frequency analysis
#[derive(Debug, Clone)]
pub struct FrequencyResult {
    /// Natural frequencies in Hz
    pub frequencies: Vec<f64>,
    /// Angular frequencies in rad/s (ω = 2πf)
    pub angular_frequencies: Vec<f64>,
    /// Eigenvalues (λ = ω²)
    pub eigenvalues: Vec<f64>,
    /// Mode shapes (num_dofs × num_modes)
    pub mode_shapes: DMatrix<f64>,
    /// Number of modes extracted
    pub num_modes: usize,
    /// Participation factors (optional)
    pub participation_factors: Option<Vec<f64>>,
}

impl FrequencyResult {
    /// Get mode shape for a specific mode
    ///
    /// # Arguments
    /// * `mode_index` - Mode number (0-indexed)
    ///
    /// # Returns
    /// Column vector of displacements for this mode
    pub fn get_mode_shape(&self, mode_index: usize) -> Option<Vec<f64>> {
        if mode_index >= self.num_modes {
            return None;
        }
        Some(self.mode_shapes.column(mode_index).iter().copied().collect())
    }

    /// Get frequency in Hz for a specific mode
    pub fn get_frequency(&self, mode_index: usize) -> Option<f64> {
        self.frequencies.get(mode_index).copied()
    }

    /// Get period in seconds for a specific mode
    pub fn get_period(&self, mode_index: usize) -> Option<f64> {
        self.frequencies.get(mode_index).map(|f| 1.0 / f)
    }
}

/// Perform frequency (modal) analysis
///
/// # Arguments
/// * `mesh` - Finite element mesh
/// * `materials` - Material properties
/// * `boundary_conditions` - Boundary conditions (constraints only, no loads)
/// * `config` - Frequency analysis configuration
///
/// # Returns
/// Frequency analysis results with natural frequencies and mode shapes
///
/// # Example
/// ```ignore
/// let config = FrequencyConfig {
///     num_modes: 5,
///     which: WhichEigenvalues::SmallestMagnitude,
///     ..Default::default()
/// };
///
/// let result = frequency_analysis(&mesh, &materials, &bcs, &config)?;
///
/// for (i, freq) in result.frequencies.iter().enumerate() {
///     println!("Mode {}: {:.2} Hz", i + 1, freq);
/// }
/// ```
pub fn frequency_analysis(
    mesh: &Mesh,
    materials: &HashMap<String, Material>,
    boundary_conditions: &BoundaryConditions,
    config: &FrequencyConfig,
) -> Result<FrequencyResult, String> {
    // 1. Create placeholder system
    // TODO: Implement proper sparse assembly integration
    let num_dofs = mesh.num_dofs;

    // For now, return error indicating incomplete implementation
    return Err("Frequency analysis requires complete mass matrix assembly - implementation in progress".to_string());

    // TODO: Complete implementation when mass matrix assembly is integrated
    // The code below is commented out to prevent compilation errors
    /*
    // 4. Solve generalized eigenvalue problem: K * φ = λ * M * φ
    let backend = default_backend();
    let eigen_system = EigenSystemData {
        stiffness: k_reduced,
        mass: m_reduced,
        num_dofs: system.num_dofs,
        free_dofs: free_dofs.clone(),
    };

    let (eigen_result, _solve_info) = backend
        .solve_eigen(&eigen_system, config.num_modes)
        .map_err(|e| format!("Eigenvalue solve failed: {}", e))?;

    // 5. Convert eigenvalues to frequencies
    let eigenvalues = eigen_result.eigenvalues.clone();
    let angular_frequencies: Vec<f64> = eigenvalues
        .iter()
        .map(|&lambda| {
            if lambda < 0.0 {
                0.0 // Negative eigenvalues → rigid body modes
            } else {
                lambda.sqrt()
            }
        })
        .collect();

    let frequencies: Vec<f64> = angular_frequencies
        .iter()
        .map(|&omega| omega / (2.0 * std::f64::consts::PI))
        .collect();

    // 6. Expand mode shapes to full DOF space
    let mode_shapes = expand_eigenvectors(&eigen_result.eigenvectors, &free_dofs, system.num_dofs)?;

    Ok(FrequencyResult {
        frequencies,
        angular_frequencies,
        eigenvalues,
        mode_shapes,
        num_modes: config.num_modes.min(eigenvalues.len()),
        participation_factors: None,
    })
    */
}

/// Convert dense matrix to COO (Coordinate) sparse format
fn to_coo_triplets(matrix: &DMatrix<f64>) -> Result<SparseTripletsF64, String> {
    let nrows = matrix.nrows();
    let ncols = matrix.ncols();

    let mut row_indices = Vec::new();
    let mut col_indices = Vec::new();
    let mut values = Vec::new();

    for i in 0..nrows {
        for j in 0..ncols {
            let val = matrix[(i, j)];
            if val.abs() > 1e-15 {
                row_indices.push(i);
                col_indices.push(j);
                values.push(val);
            }
        }
    }

    Ok(SparseTripletsF64 {
        nrows,
        ncols,
        row_indices,
        col_indices,
        values,
    })
}

/// Assemble global mass matrix in COO format
///
/// TODO: This is a placeholder. Full implementation requires:
/// - Element-level mass matrix computation
/// - Assembly similar to stiffness matrix
fn assemble_mass_matrix_coo(
    _mesh: &Mesh,
    _materials: &HashMap<String, Material>,
) -> Result<SparseTripletsF64, String> {
    // Placeholder: Return identity mass matrix
    // In production, this should call element.mass_matrix() for each element
    Err("Mass matrix assembly not yet implemented".to_string())
}

/// Apply constraints to matrices by removing constrained DOFs
///
/// Returns (K_reduced, M_reduced, free_dofs)
fn apply_constraints(
    k_triplets: SparseTripletsF64,
    m_triplets: SparseTripletsF64,
    bcs: &BoundaryConditions,
    num_dofs: usize,
) -> Result<(SparseTripletsF64, SparseTripletsF64, Vec<usize>), String> {
    // Identify free (unconstrained) DOFs
    let mut is_constrained = vec![false; num_dofs];
    // Access displacement BCs directly from the struct
    let constrained_dofs = bcs.get_constrained_dofs();
    for (dof_id, _value) in constrained_dofs.iter() {
        let global_dof = (dof_id.node - 1) as usize * 3 + dof_id.dof - 1;
        if global_dof < num_dofs {
            is_constrained[global_dof] = true;
        }
    }

    let free_dofs: Vec<usize> = (0..num_dofs)
        .filter(|&i| !is_constrained[i])
        .collect();

    // Create mapping from full DOFs to reduced DOFs
    let mut dof_map = vec![None; num_dofs];
    for (new_idx, &old_idx) in free_dofs.iter().enumerate() {
        dof_map[old_idx] = Some(new_idx);
    }

    // Filter triplets to only include free DOFs
    let filter_triplets = |triplets: SparseTripletsF64| -> SparseTripletsF64 {
        let mut row_indices = Vec::new();
        let mut col_indices = Vec::new();
        let mut values = Vec::new();

        for ((&row, &col), &val) in triplets
            .row_indices
            .iter()
            .zip(triplets.col_indices.iter())
            .zip(triplets.values.iter())
        {
            if let (Some(new_row), Some(new_col)) = (dof_map[row], dof_map[col]) {
                row_indices.push(new_row);
                col_indices.push(new_col);
                values.push(val);
            }
        }

        SparseTripletsF64 {
            nrows: free_dofs.len(),
            ncols: free_dofs.len(),
            row_indices,
            col_indices,
            values,
        }
    };

    let k_reduced = filter_triplets(k_triplets);
    let m_reduced = filter_triplets(m_triplets);

    Ok((k_reduced, m_reduced, free_dofs))
}

/// Expand eigenvectors from reduced DOF space to full DOF space
fn expand_eigenvectors(
    eigenvectors: &DMatrix<f64>,
    free_dofs: &[usize],
    num_dofs: usize,
) -> Result<DMatrix<f64>, String> {
    let num_modes = eigenvectors.ncols();
    let mut full_eigenvectors = DMatrix::<f64>::zeros(num_dofs, num_modes);

    for mode in 0..num_modes {
        for (reduced_idx, &full_idx) in free_dofs.iter().enumerate() {
            full_eigenvectors[(full_idx, mode)] = eigenvectors[(reduced_idx, mode)];
        }
    }

    Ok(full_eigenvectors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_config_default() {
        let config = FrequencyConfig::default();
        assert_eq!(config.num_modes, 10);
        assert_eq!(config.which, WhichEigenvalues::SmallestMagnitude);
        assert_eq!(config.tolerance, 1e-6);
    }

    #[test]
    fn test_frequency_result_accessors() {
        let eigenvalues = vec![100.0, 400.0, 900.0];
        let angular_frequencies: Vec<f64> = eigenvalues.iter().map(|&e| (e as f64).sqrt()).collect();
        let frequencies: Vec<f64> = angular_frequencies
            .iter()
            .map(|&w| (w as f64) / (2.0 * std::f64::consts::PI))
            .collect();

        let result = FrequencyResult {
            frequencies,
            angular_frequencies,
            eigenvalues,
            mode_shapes: DMatrix::zeros(10, 3),
            num_modes: 3,
            participation_factors: None,
        };

        // Mode 0: λ=100, ω=10, f=10/(2π)
        assert!((result.get_frequency(0).unwrap() - 1.5915).abs() < 0.001);
        // Mode 1: λ=400, ω=20, f=20/(2π)
        assert!((result.get_frequency(1).unwrap() - 3.1831).abs() < 0.001);
    }

    #[test]
    fn test_to_coo_triplets() {
        let mut mat = DMatrix::zeros(3, 3);
        mat[(0, 0)] = 1.0;
        mat[(1, 1)] = 2.0;
        mat[(0, 2)] = 3.0;

        let coo = to_coo_triplets(&mat).unwrap();
        assert_eq!(coo.nnz(), 3);
        assert_eq!(coo.nrows, 3);
        assert_eq!(coo.ncols, 3);
    }
}
