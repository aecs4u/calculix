//! Analysis pipeline definitions and execution framework.
//!
//! This module provides the structure for running different types of finite element
//! analyses (linear static, modal, dynamic, etc.).

use ccx_inp::Deck;
use ccx_model::ModelSummary;

/// Analysis type enumeration matching CalculiX capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisType {
    /// Linear static structural analysis (*STATIC)
    LinearStatic,
    /// Nonlinear static analysis (*STATIC with nonlinear material/contact)
    NonlinearStatic,
    /// Modal/frequency analysis (*FREQUENCY)
    Modal,
    /// Steady-state dynamics (*STEADY STATE DYNAMICS)
    SteadyStateDynamics,
    /// Dynamic time integration (*DYNAMIC)
    Dynamic,
    /// Heat transfer analysis (*HEAT TRANSFER)
    HeatTransfer,
    /// Coupled thermomechanical analysis
    CoupledThermoMechanical,
    /// Buckling analysis (*BUCKLE)
    Buckling,
    /// Complex frequency analysis (*COMPLEX FREQUENCY)
    ComplexFrequency,
    /// Green's function analysis (*GREEN)
    Green,
    /// Sensitivity analysis (*SENSITIVITY)
    Sensitivity,
    /// Modal dynamics with superposition (*MODAL DYNAMIC)
    ModalDynamic,
    /// Visco analysis (*VISCO)
    Visco,
    /// Electromagnetic analysis (*ELECTROMAGNETICS)
    Electromagnetic,
    /// Uncoupled temperature-displacement (*UNCOUPLED TEMPERATURE-DISPLACEMENT)
    UncoupledThermoMechanical,
    /// CFD analysis (*CFD)
    CFD,
}

/// Analysis results and statistics
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisResults {
    /// Whether the analysis completed successfully
    pub success: bool,
    /// Number of degrees of freedom
    pub num_dofs: usize,
    /// Number of equations solved
    pub num_equations: usize,
    /// Analysis type that was run
    pub analysis_type: AnalysisType,
    /// Human-readable status message
    pub message: String,
}

/// Analysis configuration and control
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Type of analysis to perform
    pub analysis_type: AnalysisType,
    /// Maximum number of iterations (for nonlinear)
    pub max_iterations: usize,
    /// Convergence tolerance (for nonlinear)
    pub tolerance: f64,
    /// Whether to write detailed output
    pub verbose: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            analysis_type: AnalysisType::LinearStatic,
            max_iterations: 200,
            tolerance: 1e-8,
            verbose: false,
        }
    }
}

/// Main analysis pipeline orchestrator
pub struct AnalysisPipeline {
    config: AnalysisConfig,
}

impl AnalysisPipeline {
    /// Create a new analysis pipeline with the given configuration
    pub fn new(config: AnalysisConfig) -> Self {
        Self { config }
    }

    /// Create a pipeline for linear static analysis
    pub fn linear_static() -> Self {
        Self::new(AnalysisConfig {
            analysis_type: AnalysisType::LinearStatic,
            ..Default::default()
        })
    }

    /// Create a pipeline for modal/frequency analysis
    pub fn modal() -> Self {
        Self::new(AnalysisConfig {
            analysis_type: AnalysisType::Modal,
            ..Default::default()
        })
    }

    /// Create a pipeline for heat transfer analysis
    pub fn heat_transfer() -> Self {
        Self::new(AnalysisConfig {
            analysis_type: AnalysisType::HeatTransfer,
            ..Default::default()
        })
    }

    /// Create a pipeline for dynamic analysis
    pub fn dynamic() -> Self {
        Self::new(AnalysisConfig {
            analysis_type: AnalysisType::Dynamic,
            ..Default::default()
        })
    }

    /// Create a pipeline for buckling analysis
    pub fn buckling() -> Self {
        Self::new(AnalysisConfig {
            analysis_type: AnalysisType::Buckling,
            ..Default::default()
        })
    }

    /// Detect the appropriate analysis type from the input deck
    ///
    /// Examines keywords in the deck to automatically determine which analysis to run.
    pub fn detect_from_deck(deck: &Deck) -> Self {
        let summary = ModelSummary::from_deck(deck);

        // Check keyword counts for specific analysis types
        let has_buckle = summary
            .keyword_counts
            .keys()
            .any(|k| k.to_uppercase().contains("BUCKLE"));
        let has_complex_freq = summary.keyword_counts.keys().any(|k| {
            k.to_uppercase().contains("COMPLEX") && k.to_uppercase().contains("FREQUENCY")
        });
        let has_green = summary
            .keyword_counts
            .keys()
            .any(|k| k.to_uppercase().contains("GREEN"));
        let has_sensitivity = summary
            .keyword_counts
            .keys()
            .any(|k| k.to_uppercase().contains("SENSITIVITY"));
        let has_modal_dynamic = summary
            .keyword_counts
            .keys()
            .any(|k| k.to_uppercase().contains("MODAL") && k.to_uppercase().contains("DYNAMIC"));
        let has_steady_state = summary
            .keyword_counts
            .keys()
            .any(|k| k.to_uppercase().contains("STEADY") && k.to_uppercase().contains("STATE"));
        let has_visco = summary
            .keyword_counts
            .keys()
            .any(|k| k.to_uppercase().contains("VISCO"));
        let has_electromagnetic = summary
            .keyword_counts
            .keys()
            .any(|k| k.to_uppercase().contains("ELECTROMAGNETIC"));
        let has_cfd = summary
            .keyword_counts
            .keys()
            .any(|k| k.to_uppercase().contains("CFD"));
        let has_uncoupled_thermo = summary.keyword_counts.keys().any(|k| {
            k.to_uppercase().contains("UNCOUPLED") && k.to_uppercase().contains("TEMPERATURE")
        });

        let analysis_type = if has_buckle {
            AnalysisType::Buckling
        } else if has_complex_freq {
            AnalysisType::ComplexFrequency
        } else if has_green {
            AnalysisType::Green
        } else if has_sensitivity {
            AnalysisType::Sensitivity
        } else if has_modal_dynamic {
            AnalysisType::ModalDynamic
        } else if has_steady_state {
            AnalysisType::SteadyStateDynamics
        } else if has_visco {
            AnalysisType::Visco
        } else if has_electromagnetic {
            AnalysisType::Electromagnetic
        } else if has_cfd {
            AnalysisType::CFD
        } else if has_uncoupled_thermo {
            AnalysisType::UncoupledThermoMechanical
        } else if summary.has_frequency {
            AnalysisType::Modal
        } else if summary.has_dynamic {
            AnalysisType::Dynamic
        } else if summary.has_heat_transfer && summary.has_static {
            AnalysisType::CoupledThermoMechanical
        } else if summary.has_heat_transfer {
            AnalysisType::HeatTransfer
        } else if summary.has_static {
            // TODO: Detect nonlinear from material/contact cards
            AnalysisType::LinearStatic
        } else {
            // Default to linear static
            AnalysisType::LinearStatic
        };

        Self::new(AnalysisConfig {
            analysis_type,
            ..Default::default()
        })
    }

    /// Run the analysis pipeline
    ///
    /// This is currently a skeleton that will be filled in as we port more solver code.
    pub fn run(&self, deck: &Deck) -> Result<AnalysisResults, String> {
        let summary = ModelSummary::from_deck(deck);

        // Validate we have necessary data
        if summary.node_rows == 0 {
            return Err("No nodes defined in model".to_string());
        }

        if summary.element_rows == 0 {
            return Err("No elements defined in model".to_string());
        }

        // Step 1: Build node/element data structures
        let mut mesh = crate::mesh_builder::MeshBuilder::build_from_deck(deck)?;
        mesh.calculate_dofs();
        let mesh_stats = mesh.statistics();

        // Step 2: Build boundary conditions and loads
        let bcs = crate::bc_builder::BCBuilder::build_from_deck(deck)?;
        let bc_stats = bcs.statistics();

        // Calculate constrained and free DOFs
        let constrained_dofs = bcs.get_constrained_dofs();
        let free_dofs = mesh.num_dofs - constrained_dofs.len();

        // For structural analysis with truss elements, attempt to solve
        let solve_message = if self.config.analysis_type == AnalysisType::LinearStatic {
            // Step 3: Build materials
            match crate::materials::MaterialLibrary::build_from_deck(deck) {
                Ok(mut materials) => {
                    // Assign default material to all elements if not explicitly assigned
                    if let Some(first_mat_name) = materials.material_names().first().cloned() {
                        for elem_id in mesh.elements.keys() {
                            if materials.get_element_material(*elem_id).is_none() {
                                materials.assign_material(*elem_id, first_mat_name.clone());
                            }
                        }
                    }

                    // Step 4: Assemble and solve (only for truss elements currently)
                    let has_truss_elements = mesh
                        .elements
                        .values()
                        .any(|e| matches!(e.element_type, crate::mesh::ElementType::T3D2));

                    if has_truss_elements {
                        match crate::assembly::GlobalSystem::assemble(
                            &mesh, &materials, &bcs, 0.001,
                        ) {
                            Ok(system) => match system.solve() {
                                Ok(_displacements) => " [SOLVED]".to_string(),
                                Err(e) => format!(" [SOLVE FAILED: {}]", e),
                            },
                            Err(e) => format!(" [ASSEMBLY FAILED: {}]", e),
                        }
                    } else {
                        " [solver supports T3D2 truss elements only]".to_string()
                    }
                }
                Err(_) => " [no materials defined]".to_string(),
            }
        } else {
            String::new()
        };

        Ok(AnalysisResults {
            success: true,
            num_dofs: mesh.num_dofs,
            num_equations: free_dofs, // Only free DOFs are solved
            analysis_type: self.config.analysis_type,
            message: format!(
                "Model initialized: {} nodes, {} elements, {} DOFs ({} free, {} constrained), {} loads{}",
                mesh_stats.num_nodes,
                mesh_stats.num_elements,
                mesh.num_dofs,
                free_dofs,
                constrained_dofs.len(),
                bc_stats.num_concentrated_loads,
                solve_message
            ),
        })
    }

    /// Get the current configuration
    pub fn config(&self) -> &AnalysisConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deck_with_keywords(extra_cards: &str) -> Deck {
        let deck_src = format!(
            r#"
*NODE
1,0,0,0
2,1,0,0
3,1,1,0
4,0,1,0
5,0,0,1
6,1,0,1
7,1,1,1
8,0,1,1
*ELEMENT,TYPE=C3D8
1,1,2,3,4,5,6,7,8
*MATERIAL,NAME=STEEL
*STEP
{extra_cards}
*END STEP
"#
        );
        Deck::parse_str(&deck_src).expect("deck should parse")
    }

    #[test]
    fn linear_static_pipeline_creation() {
        let pipeline = AnalysisPipeline::linear_static();
        assert_eq!(pipeline.config().analysis_type, AnalysisType::LinearStatic);
    }

    #[test]
    fn detects_static_analysis() {
        let deck = deck_with_keywords("*STATIC");
        let pipeline = AnalysisPipeline::detect_from_deck(&deck);
        assert_eq!(pipeline.config().analysis_type, AnalysisType::LinearStatic);
    }

    #[test]
    fn detects_frequency_analysis() {
        let deck = deck_with_keywords("*FREQUENCY");
        let pipeline = AnalysisPipeline::detect_from_deck(&deck);
        assert_eq!(pipeline.config().analysis_type, AnalysisType::Modal);
    }

    #[test]
    fn validates_required_data() {
        let deck_src = r#"
*MATERIAL,NAME=STEEL
"#;
        let deck = Deck::parse_str(deck_src).expect("deck should parse");
        let pipeline = AnalysisPipeline::linear_static();
        let result = pipeline.run(&deck);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No nodes defined"));
    }

    #[test]
    fn validates_missing_elements() {
        let deck_src = r#"
*NODE
1,0,0,0
*STEP
*STATIC
*END STEP
"#;
        let deck = Deck::parse_str(deck_src).expect("deck should parse");
        let pipeline = AnalysisPipeline::linear_static();
        let result = pipeline.run(&deck);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No elements defined"));
    }

    #[test]
    fn basic_pipeline_execution() {
        let deck = deck_with_keywords("*STATIC");
        let pipeline = AnalysisPipeline::linear_static();
        let result = pipeline.run(&deck).expect("run should succeed");

        assert!(result.success);
        assert_eq!(result.num_dofs, 8 * 3); // 8 nodes * 3 DOFs
        assert_eq!(result.analysis_type, AnalysisType::LinearStatic);
    }

    #[test]
    fn detects_buckling_analysis() {
        let deck = deck_with_keywords("*BUCKLE");
        let pipeline = AnalysisPipeline::detect_from_deck(&deck);
        assert_eq!(pipeline.config().analysis_type, AnalysisType::Buckling);
    }

    #[test]
    fn detects_complex_frequency_analysis() {
        let deck = deck_with_keywords("*COMPLEX FREQUENCY");
        let pipeline = AnalysisPipeline::detect_from_deck(&deck);
        assert_eq!(
            pipeline.config().analysis_type,
            AnalysisType::ComplexFrequency
        );
    }

    #[test]
    fn detects_steady_state_dynamics_analysis() {
        let deck = deck_with_keywords("*STEADY STATE DYNAMICS");
        let pipeline = AnalysisPipeline::detect_from_deck(&deck);
        assert_eq!(
            pipeline.config().analysis_type,
            AnalysisType::SteadyStateDynamics
        );
    }

    #[test]
    fn detects_modal_dynamic_analysis() {
        let deck = deck_with_keywords("*MODAL DYNAMIC");
        let pipeline = AnalysisPipeline::detect_from_deck(&deck);
        assert_eq!(pipeline.config().analysis_type, AnalysisType::ModalDynamic);
    }

    #[test]
    fn detects_uncoupled_thermo_mechanical_analysis() {
        let deck = deck_with_keywords("*UNCOUPLED TEMPERATURE-DISPLACEMENT");
        let pipeline = AnalysisPipeline::detect_from_deck(&deck);
        assert_eq!(
            pipeline.config().analysis_type,
            AnalysisType::UncoupledThermoMechanical
        );
    }

    #[test]
    fn detects_coupled_thermo_mechanical_analysis() {
        let deck = deck_with_keywords("*HEAT TRANSFER\n*STATIC");
        let pipeline = AnalysisPipeline::detect_from_deck(&deck);
        assert_eq!(
            pipeline.config().analysis_type,
            AnalysisType::CoupledThermoMechanical
        );
    }

    #[test]
    fn frequency_takes_precedence_over_dynamic() {
        let deck = deck_with_keywords("*FREQUENCY\n*DYNAMIC");
        let pipeline = AnalysisPipeline::detect_from_deck(&deck);
        assert_eq!(pipeline.config().analysis_type, AnalysisType::Modal);
    }
}
