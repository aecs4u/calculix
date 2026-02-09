///! Postprocessing utilities for stress and strain analysis
///!
///! Provides computations for derived quantities like:
///! - von Mises stress and strain
///! - Principal stresses and strains
///! - Effective stress and strain
///!
///! ## Usage
///!
///! ```rust
///! use ccx_io::postprocess::{compute_mises_stress, TensorComponents};
///!
///! let stress = TensorComponents {
///!     xx: 100.0,
///!     yy: 50.0,
///!     zz: 25.0,
///!     xy: 10.0,
///!     yz: 5.0,
///!     xz: 2.0,
///! };
///!
///! let mises = compute_mises_stress(&stress);
///! println!("von Mises stress: {}", mises);
///! ```

/// Stress or strain tensor components (Voigt notation)
#[derive(Debug, Clone, Copy, Default)]
pub struct TensorComponents {
    /// Normal component XX
    pub xx: f64,
    /// Normal component YY
    pub yy: f64,
    /// Normal component ZZ
    pub zz: f64,
    /// Shear component XY
    pub xy: f64,
    /// Shear component YZ
    pub yz: f64,
    /// Shear component XZ
    pub xz: f64,
}

/// Principal values (eigenvalues of tensor)
#[derive(Debug, Clone, Copy)]
pub struct PrincipalValues {
    /// Maximum principal value
    pub max: f64,
    /// Middle principal value
    pub mid: f64,
    /// Minimum principal value
    pub min: f64,
}

/// Compute von Mises stress from stress tensor components
///
/// Formula: σ_v = sqrt(0.5 * [(σ_xx - σ_yy)² + (σ_yy - σ_zz)² + (σ_zz - σ_xx)²] + 3 * [τ_xy² + τ_yz² + τ_xz²])
///
/// # Arguments
///
/// * `stress` - Stress tensor components in Voigt notation
///
/// # Returns
///
/// von Mises equivalent stress
///
/// # Example
///
/// ```
/// use ccx_io::postprocess::{compute_mises_stress, TensorComponents};
///
/// let stress = TensorComponents {
///     xx: 100.0, yy: 50.0, zz: 25.0,
///     xy: 10.0, yz: 5.0, xz: 2.0,
/// };
///
/// let mises = compute_mises_stress(&stress);
/// assert!(mises > 0.0);
/// ```
pub fn compute_mises_stress(stress: &TensorComponents) -> f64 {
    let term1 = 0.5
        * ((stress.xx - stress.yy).powi(2)
            + (stress.yy - stress.zz).powi(2)
            + (stress.zz - stress.xx).powi(2));

    let term2 = 3.0 * (stress.xy.powi(2) + stress.yz.powi(2) + stress.xz.powi(2));

    (term1 + term2).sqrt()
}

/// Compute von Mises strain from strain tensor components
///
/// Formula: ε_v = (2/3) * sqrt(0.5 * [(ε_xx - ε_yy)² + (ε_yy - ε_zz)² + (ε_zz - ε_xx)²] + 3 * [γ_xy² + γ_yz² + γ_xz²])
///
/// Note: Engineering shear strains (γ) are used, not tensor shear strains (ε)
pub fn compute_mises_strain(strain: &TensorComponents) -> f64 {
    let term1 = 0.5
        * ((strain.xx - strain.yy).powi(2)
            + (strain.yy - strain.zz).powi(2)
            + (strain.zz - strain.xx).powi(2));

    let term2 = 3.0 * (strain.xy.powi(2) + strain.yz.powi(2) + strain.xz.powi(2));

    (2.0 / 3.0) * (term1 + term2).sqrt()
}

/// Compute principal stresses (eigenvalues of stress tensor)
///
/// For a 3D symmetric tensor, computes the three principal values.
/// Uses characteristic equation for 3×3 symmetric matrix.
///
/// # Arguments
///
/// * `stress` - Stress tensor components
///
/// # Returns
///
/// Principal stresses sorted as (max, mid, min)
pub fn compute_principal_stresses(stress: &TensorComponents) -> PrincipalValues {
    compute_principal_values(stress)
}

/// Compute principal strains (eigenvalues of strain tensor)
pub fn compute_principal_strains(strain: &TensorComponents) -> PrincipalValues {
    compute_principal_values(strain)
}

/// Generic principal value computation for symmetric 3×3 tensor
///
/// Solves the characteristic equation: det(T - λI) = 0
/// where λ are the eigenvalues (principal values)
fn compute_principal_values(tensor: &TensorComponents) -> PrincipalValues {
    // Special case: diagonal or nearly diagonal tensor
    let shear_norm = tensor.xy.abs() + tensor.yz.abs() + tensor.xz.abs();
    if shear_norm < 1e-10 {
        // Diagonal tensor - eigenvalues are just the diagonal entries
        let mut principals = [tensor.xx, tensor.yy, tensor.zz];
        principals.sort_by(|a, b| b.partial_cmp(a).unwrap());
        return PrincipalValues {
            max: principals[0],
            mid: principals[1],
            min: principals[2],
        };
    }

    // Tensor in matrix form (symmetric):
    // | xx  xy  xz |
    // | xy  yy  yz |
    // | xz  yz  zz |

    // Invariants of the stress tensor
    let i1 = tensor.xx + tensor.yy + tensor.zz; // First invariant (trace)

    let i2 = tensor.xx * tensor.yy + tensor.yy * tensor.zz + tensor.zz * tensor.xx
        - tensor.xy.powi(2)
        - tensor.yz.powi(2)
        - tensor.xz.powi(2); // Second invariant

    let i3 = tensor.xx * tensor.yy * tensor.zz
        + 2.0 * tensor.xy * tensor.yz * tensor.xz
        - tensor.xx * tensor.yz.powi(2)
        - tensor.yy * tensor.xz.powi(2)
        - tensor.zz * tensor.xy.powi(2); // Third invariant (determinant)

    // Solve cubic equation: λ³ - I₁λ² + I₂λ - I₃ = 0
    // Using trigonometric method for three real roots

    let p = i2 - i1.powi(2) / 3.0;
    let q = 2.0 * i1.powi(3) / 27.0 - i1 * i2 / 3.0 + i3;

    // For symmetric tensor, always use trigonometric method
    let eps = 1e-14;
    if p.abs() < eps {
        // Degenerate case: all eigenvalues equal
        let lambda = i1 / 3.0;
        return PrincipalValues {
            max: lambda,
            mid: lambda,
            min: lambda,
        };
    }

    let theta = ((-q / 2.0) / ((-p / 3.0).powf(1.5))).acos();
    let k = 2.0 * (-p / 3.0).sqrt();

    let lambda1 = k * (theta / 3.0).cos() + i1 / 3.0;
    let lambda2 = k * ((theta + 2.0 * std::f64::consts::PI) / 3.0).cos() + i1 / 3.0;
    let lambda3 = k * ((theta + 4.0 * std::f64::consts::PI) / 3.0).cos() + i1 / 3.0;

    // Sort in descending order
    let mut principals = [lambda1, lambda2, lambda3];
    principals.sort_by(|a, b| b.partial_cmp(a).unwrap());

    PrincipalValues {
        max: principals[0],
        mid: principals[1],
        min: principals[2],
    }
}

/// Compute hydrostatic (mean) stress
///
/// Formula: σ_h = (σ_xx + σ_yy + σ_zz) / 3
pub fn compute_hydrostatic_stress(stress: &TensorComponents) -> f64 {
    (stress.xx + stress.yy + stress.zz) / 3.0
}

/// Compute deviatoric stress tensor
///
/// Subtracts the hydrostatic component from each normal stress
pub fn compute_deviatoric_stress(stress: &TensorComponents) -> TensorComponents {
    let hydro = compute_hydrostatic_stress(stress);

    TensorComponents {
        xx: stress.xx - hydro,
        yy: stress.yy - hydro,
        zz: stress.zz - hydro,
        xy: stress.xy,
        yz: stress.yz,
        xz: stress.xz,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mises_stress_uniaxial() {
        // Uniaxial tension: σ_xx = 100, others = 0
        // von Mises should equal σ_xx for uniaxial state
        let stress = TensorComponents {
            xx: 100.0,
            yy: 0.0,
            zz: 0.0,
            xy: 0.0,
            yz: 0.0,
            xz: 0.0,
        };

        let mises = compute_mises_stress(&stress);
        assert!((mises - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_mises_stress_pure_shear() {
        // Pure shear: τ_xy = 100, others = 0
        // von Mises = sqrt(3) * τ_xy
        let stress = TensorComponents {
            xx: 0.0,
            yy: 0.0,
            zz: 0.0,
            xy: 100.0,
            yz: 0.0,
            xz: 0.0,
        };

        let mises = compute_mises_stress(&stress);
        let expected = (3.0_f64).sqrt() * 100.0;
        assert!((mises - expected).abs() < 1e-6);
    }

    #[test]
    fn test_principal_stress_uniaxial() {
        // Uniaxial tension
        let stress = TensorComponents {
            xx: 100.0,
            yy: 0.0,
            zz: 0.0,
            xy: 0.0,
            yz: 0.0,
            xz: 0.0,
        };

        let principals = compute_principal_stresses(&stress);

        // For uniaxial stress, one principal should be 100, others should be 0
        // Check that at least one is close to 100 and the sum equals the trace
        let values = [principals.max, principals.mid, principals.min];
        let has_100 = values.iter().any(|&v| (v - 100.0).abs() < 1e-3);
        let sum = principals.max + principals.mid + principals.min;

        assert!(has_100, "One principal value should be ~100, got: {:?}", values);
        assert!((sum - 100.0).abs() < 1e-3, "Sum of principals should equal trace (100)");
    }

    #[test]
    fn test_hydrostatic_stress() {
        let stress = TensorComponents {
            xx: 100.0,
            yy: 50.0,
            zz: 25.0,
            xy: 0.0,
            yz: 0.0,
            xz: 0.0,
        };

        let hydro = compute_hydrostatic_stress(&stress);
        let expected = (100.0 + 50.0 + 25.0) / 3.0;
        assert!((hydro - expected).abs() < 1e-6);
    }

    #[test]
    fn test_deviatoric_stress() {
        let stress = TensorComponents {
            xx: 100.0,
            yy: 50.0,
            zz: 25.0,
            xy: 10.0,
            yz: 5.0,
            xz: 2.0,
        };

        let deviatoric = compute_deviatoric_stress(&stress);
        let hydro = compute_hydrostatic_stress(&stress);

        // Check that normal stresses sum to zero
        let sum = deviatoric.xx + deviatoric.yy + deviatoric.zz;
        assert!(sum.abs() < 1e-6);

        // Check shear components unchanged
        assert!((deviatoric.xy - stress.xy).abs() < 1e-6);
        assert!((deviatoric.yz - stress.yz).abs() < 1e-6);
        assert!((deviatoric.xz - stress.xz).abs() < 1e-6);
    }
}
