// Post-processing module for CalculiX solver output
// Reads element variable output from .dat files and computes stress/strain metrics
// Based on CCXStressReader.py by Henning Richter

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Stress tensor components at an integration point
#[derive(Debug, Clone, PartialEq)]
pub struct StressState {
    pub sxx: f64,
    pub syy: f64,
    pub szz: f64,
    pub sxy: f64,
    pub sxz: f64,
    pub syz: f64,
}

/// Strain tensor components at an integration point
#[derive(Debug, Clone, PartialEq)]
pub struct StrainState {
    pub exx: f64,
    pub eyy: f64,
    pub ezz: f64,
    pub exy: f64,
    pub exz: f64,
    pub eyz: f64,
}

/// Integration point data from element variable output
#[derive(Debug, Clone)]
pub struct IntegrationPointData {
    pub element_id: i32,
    pub point_id: i32,
    pub stress: Option<StressState>,
    pub strain: Option<StrainState>,
    pub peeq: Option<f64>, // Equivalent plastic strain
}

/// Results for a single integration point including computed values
#[derive(Debug, Clone)]
pub struct IntegrationPointResult {
    pub element_id: i32,
    pub point_id: i32,
    pub mises: f64,          // von Mises equivalent stress
    pub eeq: f64,            // Total effective strain
    pub peeq: f64,           // Equivalent plastic strain
}

/// Statistical summary of results
#[derive(Debug, Clone)]
pub struct ResultStatistics {
    pub mises_min: f64,
    pub mises_max: f64,
    pub mises_mean: f64,
    pub eeq_min: f64,
    pub eeq_max: f64,
    pub eeq_mean: f64,
    pub peeq_min: f64,
    pub peeq_max: f64,
    pub peeq_mean: f64,
}

/// Compute von Mises equivalent stress from stress tensor components
///
/// Formula: σ_v = sqrt(0.5 * ((σ_xx - σ_yy)² + (σ_yy - σ_zz)² + (σ_zz - σ_xx)²)
///                    + 3 * (τ_xy² + τ_xz² + τ_yz²))
///
/// # Arguments
/// * `stress` - Stress tensor components
///
/// # Returns
/// Von Mises equivalent stress
///
/// # Example
/// ```
/// use ccx_solver::postprocess::{StressState, compute_mises_stress};
///
/// let stress = StressState {
///     sxx: 100.0, syy: 50.0, szz: 30.0,
///     sxy: 10.0, sxz: 5.0, syz: 3.0,
/// };
/// let mises = compute_mises_stress(&stress);
/// assert!(mises > 0.0);
/// ```
pub fn compute_mises_stress(stress: &StressState) -> f64 {
    let s = stress;
    let term1 = 0.5 * (
        (s.sxx - s.syy).powi(2) +
        (s.syy - s.szz).powi(2) +
        (s.szz - s.sxx).powi(2)
    );
    let term2 = 3.0 * (
        s.sxy.powi(2) +
        s.sxz.powi(2) +
        s.syz.powi(2)
    );
    (term1 + term2).sqrt()
}

/// Compute total effective strain from strain tensor components
///
/// Formula: ε_eff = (2/3) * sqrt(0.5 * ((ε_xx - ε_yy)² + (ε_yy - ε_zz)² + (ε_zz - ε_xx)²)
///                              + 3 * (γ_xy² + γ_xz² + γ_yz²))
///
/// # Arguments
/// * `strain` - Strain tensor components
///
/// # Returns
/// Total effective strain
///
/// # Example
/// ```
/// use ccx_solver::postprocess::{StrainState, compute_effective_strain};
///
/// let strain = StrainState {
///     exx: 0.001, eyy: 0.0005, ezz: 0.0003,
///     exy: 0.0001, exz: 0.00005, eyz: 0.00003,
/// };
/// let eeq = compute_effective_strain(&strain);
/// assert!(eeq > 0.0);
/// ```
pub fn compute_effective_strain(strain: &StrainState) -> f64 {
    let e = strain;
    let term1 = 0.5 * (
        (e.exx - e.eyy).powi(2) +
        (e.eyy - e.ezz).powi(2) +
        (e.ezz - e.exx).powi(2)
    );
    let term2 = 3.0 * (
        e.exy.powi(2) +
        e.exz.powi(2) +
        e.eyz.powi(2)
    );
    (2.0 / 3.0) * (term1 + term2).sqrt()
}

/// Parse a .dat file and extract element variable output
///
/// # Arguments
/// * `filepath` - Path to the .dat file
///
/// # Returns
/// Vector of integration point data
///
/// # Errors
/// Returns error if file cannot be read or parsing fails
pub fn read_dat_file<P: AsRef<Path>>(filepath: P) -> Result<Vec<IntegrationPointData>, String> {
    let file = File::open(filepath.as_ref())
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);

    let mut lines: Vec<Vec<String>> = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let parts: Vec<String> = trimmed.split_whitespace().map(|s| s.to_string()).collect();
            if !parts.is_empty() {
                lines.push(parts);
            }
        }
    }

    // Find section headers
    let mut stress_idx: Option<usize> = None;
    let mut strain_idx: Option<usize> = None;
    let mut peeq_idx: Option<usize> = None;

    for (i, parts) in lines.iter().enumerate() {
        let joined = parts.join(" ").to_lowercase();
        if joined.contains("stresses") && joined.contains("elem") && joined.contains("integ") {
            stress_idx = Some(i);
        } else if joined.contains("strains") && joined.contains("elem") && joined.contains("integ") {
            strain_idx = Some(i);
        } else if joined.contains("equivalent plastic strain") {
            peeq_idx = Some(i);
        }
    }

    if stress_idx.is_none() {
        return Err("Stress output S not found in .dat file".to_string());
    }

    // Parse stress data
    let stress_start = stress_idx.unwrap() + 1;
    let stress_end = if let Some(idx) = strain_idx {
        idx
    } else if let Some(idx) = peeq_idx {
        idx
    } else {
        lines.len()
    };

    let mut stress_data: Vec<(i32, i32, StressState)> = Vec::new();
    for i in stress_start..stress_end {
        let parts = &lines[i];
        if parts.len() >= 8 && parts[0].chars().all(|c| c.is_numeric() || c == '-') {
            let elem_id = parts[0].parse::<i32>().map_err(|e| format!("Parse error: {}", e))?;
            let pt_id = parts[1].parse::<i32>().map_err(|e| format!("Parse error: {}", e))?;
            let stress = StressState {
                sxx: parts[2].parse::<f64>().map_err(|e| format!("Parse error: {}", e))?,
                syy: parts[3].parse::<f64>().map_err(|e| format!("Parse error: {}", e))?,
                szz: parts[4].parse::<f64>().map_err(|e| format!("Parse error: {}", e))?,
                sxy: parts[5].parse::<f64>().map_err(|e| format!("Parse error: {}", e))?,
                sxz: parts[6].parse::<f64>().map_err(|e| format!("Parse error: {}", e))?,
                syz: parts[7].parse::<f64>().map_err(|e| format!("Parse error: {}", e))?,
            };
            stress_data.push((elem_id, pt_id, stress));
        }
    }

    // Parse strain data if available
    let mut strain_data: Vec<(i32, i32, StrainState)> = Vec::new();
    if let Some(strain_start_idx) = strain_idx {
        let strain_start = strain_start_idx + 1;
        let strain_end = if let Some(idx) = peeq_idx {
            idx
        } else {
            lines.len()
        };

        for i in strain_start..strain_end {
            let parts = &lines[i];
            if parts.len() >= 8 && parts[0].chars().all(|c| c.is_numeric() || c == '-') {
                let elem_id = parts[0].parse::<i32>().map_err(|e| format!("Parse error: {}", e))?;
                let pt_id = parts[1].parse::<i32>().map_err(|e| format!("Parse error: {}", e))?;
                let strain = StrainState {
                    exx: parts[2].parse::<f64>().map_err(|e| format!("Parse error: {}", e))?,
                    eyy: parts[3].parse::<f64>().map_err(|e| format!("Parse error: {}", e))?,
                    ezz: parts[4].parse::<f64>().map_err(|e| format!("Parse error: {}", e))?,
                    exy: parts[5].parse::<f64>().map_err(|e| format!("Parse error: {}", e))?,
                    exz: parts[6].parse::<f64>().map_err(|e| format!("Parse error: {}", e))?,
                    eyz: parts[7].parse::<f64>().map_err(|e| format!("Parse error: {}", e))?,
                };
                strain_data.push((elem_id, pt_id, strain));
            }
        }
    }

    // Parse PEEQ data if available
    let mut peeq_data: Vec<(i32, i32, f64)> = Vec::new();
    if let Some(peeq_start_idx) = peeq_idx {
        let peeq_start = peeq_start_idx + 1;
        let peeq_end = lines.len();

        for i in peeq_start..peeq_end {
            let parts = &lines[i];
            if parts.len() >= 3 && parts[0].chars().all(|c| c.is_numeric() || c == '-') {
                let elem_id = parts[0].parse::<i32>().map_err(|e| format!("Parse error: {}", e))?;
                let pt_id = parts[1].parse::<i32>().map_err(|e| format!("Parse error: {}", e))?;
                let peeq = parts[2].parse::<f64>().map_err(|e| format!("Parse error: {}", e))?;
                peeq_data.push((elem_id, pt_id, peeq));
            }
        }
    }

    // Combine all data
    let mut result: Vec<IntegrationPointData> = Vec::new();
    for (elem_id, pt_id, stress) in stress_data {
        let strain = strain_data.iter()
            .find(|(e, p, _)| *e == elem_id && *p == pt_id)
            .map(|(_, _, s)| s.clone());
        let peeq = peeq_data.iter()
            .find(|(e, p, _)| *e == elem_id && *p == pt_id)
            .map(|(_, _, p)| *p);

        result.push(IntegrationPointData {
            element_id: elem_id,
            point_id: pt_id,
            stress: Some(stress),
            strain,
            peeq,
        });
    }

    Ok(result)
}

/// Process integration point data and compute results
///
/// # Arguments
/// * `data` - Vector of integration point data from .dat file
///
/// # Returns
/// Vector of integration point results with computed Mises stress, effective strain, and PEEQ
pub fn process_integration_points(data: &[IntegrationPointData]) -> Vec<IntegrationPointResult> {
    data.iter()
        .map(|pt| {
            let mises = if let Some(ref stress) = pt.stress {
                compute_mises_stress(stress)
            } else {
                0.0
            };

            let eeq = if let Some(ref strain) = pt.strain {
                compute_effective_strain(strain)
            } else {
                0.0
            };

            let peeq = pt.peeq.unwrap_or(0.0);

            IntegrationPointResult {
                element_id: pt.element_id,
                point_id: pt.point_id,
                mises,
                eeq,
                peeq,
            }
        })
        .collect()
}

/// Compute statistics from integration point results
///
/// # Arguments
/// * `results` - Vector of integration point results
///
/// # Returns
/// Statistical summary (min/max/mean for Mises, EEQ, PEEQ)
pub fn compute_statistics(results: &[IntegrationPointResult]) -> ResultStatistics {
    if results.is_empty() {
        return ResultStatistics {
            mises_min: 0.0, mises_max: 0.0, mises_mean: 0.0,
            eeq_min: 0.0, eeq_max: 0.0, eeq_mean: 0.0,
            peeq_min: 0.0, peeq_max: 0.0, peeq_mean: 0.0,
        };
    }

    let mises_vals: Vec<f64> = results.iter().map(|r| r.mises).collect();
    let eeq_vals: Vec<f64> = results.iter().map(|r| r.eeq).collect();
    let peeq_vals: Vec<f64> = results.iter().map(|r| r.peeq).collect();

    let mises_min = mises_vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mises_max = mises_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mises_mean = mises_vals.iter().sum::<f64>() / mises_vals.len() as f64;

    let eeq_min = eeq_vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let eeq_max = eeq_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let eeq_mean = eeq_vals.iter().sum::<f64>() / eeq_vals.len() as f64;

    let peeq_min = peeq_vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let peeq_max = peeq_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let peeq_mean = peeq_vals.iter().sum::<f64>() / peeq_vals.len() as f64;

    ResultStatistics {
        mises_min, mises_max, mises_mean,
        eeq_min, eeq_max, eeq_mean,
        peeq_min, peeq_max, peeq_mean,
    }
}

/// Write integration point results to a text file
///
/// # Arguments
/// * `filepath` - Output file path (will replace .dat extension with _IntPtOutput.txt)
/// * `results` - Vector of integration point results
/// * `stats` - Statistical summary
///
/// # Errors
/// Returns error if file cannot be written
pub fn write_results<P: AsRef<Path>>(
    filepath: P,
    results: &[IntegrationPointResult],
    stats: &ResultStatistics,
) -> Result<(), String> {
    let path = filepath.as_ref();
    let output_path = if let Some(stem) = path.file_stem() {
        path.with_file_name(format!("{}_IntPtOutput.txt", stem.to_string_lossy()))
    } else {
        return Err("Invalid file path".to_string());
    };

    let mut file = File::create(&output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    // Write header
    writeln!(file, "     Elem.    Int.Pt.         MISES              EEQ             PEEQ")
        .map_err(|e| format!("Write error: {}", e))?;

    // Write data
    for r in results {
        writeln!(
            file,
            "{:12}{:9}   {:16.4e} {:16.4e} {:16.4e}",
            r.element_id, r.point_id, r.mises, r.eeq, r.peeq
        )
        .map_err(|e| format!("Write error: {}", e))?;
    }

    // Write statistics
    writeln!(file).map_err(|e| format!("Write error: {}", e))?;
    writeln!(
        file,
        "     Minimum                 {:16.4e} {:16.4e} {:16.4e}",
        stats.mises_min, stats.eeq_min, stats.peeq_min
    )
    .map_err(|e| format!("Write error: {}", e))?;

    writeln!(
        file,
        "     Maximum                 {:16.4e} {:16.4e} {:16.4e}",
        stats.mises_max, stats.eeq_max, stats.peeq_max
    )
    .map_err(|e| format!("Write error: {}", e))?;

    writeln!(
        file,
        "     Mean (arith.)           {:16.4e} {:16.4e} {:16.4e}",
        stats.mises_mean, stats.eeq_mean, stats.peeq_mean
    )
    .map_err(|e| format!("Write error: {}", e))?;

    writeln!(file).map_err(|e| format!("Write error: {}", e))?;

    println!("Results successfully written to file '{}'", output_path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_mises_stress_zero() {
        let stress = StressState {
            sxx: 0.0, syy: 0.0, szz: 0.0,
            sxy: 0.0, sxz: 0.0, syz: 0.0,
        };
        let mises = compute_mises_stress(&stress);
        assert_eq!(mises, 0.0);
    }

    #[test]
    fn test_compute_mises_stress_uniaxial() {
        // Uniaxial tension: σ_xx = 100 MPa, others = 0
        // Mises = |σ_xx| = 100 MPa
        let stress = StressState {
            sxx: 100.0, syy: 0.0, szz: 0.0,
            sxy: 0.0, sxz: 0.0, syz: 0.0,
        };
        let mises = compute_mises_stress(&stress);
        assert!((mises - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_compute_mises_stress_pure_shear() {
        // Pure shear: τ_xy = 100 MPa, normals = 0
        // Mises = sqrt(3) * τ_xy ≈ 173.2 MPa
        let stress = StressState {
            sxx: 0.0, syy: 0.0, szz: 0.0,
            sxy: 100.0, sxz: 0.0, syz: 0.0,
        };
        let mises = compute_mises_stress(&stress);
        let expected = (3.0_f64).sqrt() * 100.0;
        assert!((mises - expected).abs() < 1e-10);
    }

    #[test]
    fn test_compute_mises_stress_general() {
        let stress = StressState {
            sxx: 100.0, syy: 50.0, szz: 30.0,
            sxy: 10.0, sxz: 5.0, syz: 3.0,
        };
        let mises = compute_mises_stress(&stress);
        // Manual calculation:
        // term1 = 0.5 * ((100-50)^2 + (50-30)^2 + (30-100)^2)
        //       = 0.5 * (2500 + 400 + 4900) = 3900
        // term2 = 3 * (10^2 + 5^2 + 3^2) = 3 * 134 = 402
        // mises = sqrt(3900 + 402) = sqrt(4302) ≈ 65.59
        let expected = (3900.0 + 402.0_f64).sqrt();
        assert!((mises - expected).abs() < 1e-10);
    }

    #[test]
    fn test_compute_effective_strain_zero() {
        let strain = StrainState {
            exx: 0.0, eyy: 0.0, ezz: 0.0,
            exy: 0.0, exz: 0.0, eyz: 0.0,
        };
        let eeq = compute_effective_strain(&strain);
        assert_eq!(eeq, 0.0);
    }

    #[test]
    fn test_compute_effective_strain_uniaxial() {
        // Uniaxial strain: ε_xx = 0.001, others = 0
        let strain = StrainState {
            exx: 0.001, eyy: 0.0, ezz: 0.0,
            exy: 0.0, exz: 0.0, eyz: 0.0,
        };
        let eeq = compute_effective_strain(&strain);
        // eeq = (2/3) * sqrt(0.5 * (0.001^2 + 0 + 0.001^2))
        //     = (2/3) * sqrt(0.5 * 2e-6)
        //     = (2/3) * sqrt(1e-6)
        //     = (2/3) * 0.001 ≈ 0.000667
        let expected = (2.0 / 3.0) * (1e-6_f64).sqrt();
        assert!((eeq - expected).abs() < 1e-12);
    }

    #[test]
    fn test_compute_effective_strain_general() {
        let strain = StrainState {
            exx: 0.001, eyy: 0.0005, ezz: 0.0003,
            exy: 0.0001, exz: 0.00005, eyz: 0.00003,
        };
        let eeq = compute_effective_strain(&strain);
        assert!(eeq > 0.0);
        assert!(eeq < 0.01); // Sanity check
    }

    #[test]
    fn test_process_integration_points() {
        let data = vec![
            IntegrationPointData {
                element_id: 1,
                point_id: 1,
                stress: Some(StressState {
                    sxx: 100.0, syy: 0.0, szz: 0.0,
                    sxy: 0.0, sxz: 0.0, syz: 0.0,
                }),
                strain: Some(StrainState {
                    exx: 0.001, eyy: 0.0, ezz: 0.0,
                    exy: 0.0, exz: 0.0, eyz: 0.0,
                }),
                peeq: Some(0.0),
            },
        ];

        let results = process_integration_points(&data);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].element_id, 1);
        assert_eq!(results[0].point_id, 1);
        assert!((results[0].mises - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_compute_statistics() {
        let results = vec![
            IntegrationPointResult {
                element_id: 1, point_id: 1,
                mises: 100.0, eeq: 0.001, peeq: 0.0,
            },
            IntegrationPointResult {
                element_id: 1, point_id: 2,
                mises: 200.0, eeq: 0.002, peeq: 0.0,
            },
            IntegrationPointResult {
                element_id: 2, point_id: 1,
                mises: 150.0, eeq: 0.0015, peeq: 0.0,
            },
        ];

        let stats = compute_statistics(&results);
        assert_eq!(stats.mises_min, 100.0);
        assert_eq!(stats.mises_max, 200.0);
        assert_eq!(stats.mises_mean, 150.0);
        assert_eq!(stats.eeq_min, 0.001);
        assert_eq!(stats.eeq_max, 0.002);
        assert!((stats.eeq_mean - 0.0015).abs() < 1e-10);
    }

    #[test]
    fn test_compute_statistics_empty() {
        let results: Vec<IntegrationPointResult> = vec![];
        let stats = compute_statistics(&results);
        assert_eq!(stats.mises_min, 0.0);
        assert_eq!(stats.mises_max, 0.0);
        assert_eq!(stats.mises_mean, 0.0);
    }
}
