//! Analysis pipeline definitions and execution framework.
//!
//! This module provides the structure for running different types of finite element
//! analyses (linear static, modal, dynamic, etc.).

use ccx_io::inp::Deck;
use ccx_model::ModelSummary;
use crate::elements::BeamSection;
use nalgebra::Vector3;

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
    /// Displacement solution vector (empty if solve failed)
    pub displacements: Vec<f64>,
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

        // Step 1.5: Expand B32R elements to C3D20R if needed
        let use_expansion = std::env::var("CCX_EXPAND_B32R").is_ok();
        let beam_node_mapping = if use_expansion && Self::has_b32r_elements(&mesh) {
            eprintln!("  🔧 Expanding B32R → C3D20R...");
            eprintln!("     Original: {} nodes, {} elements", mesh.nodes.len(), mesh.elements.len());

            let (expanded_mesh, mapping) = Self::expand_b32r_mesh(&mesh, deck)?;
            mesh = expanded_mesh;

            eprintln!("     Expanded: {} nodes, {} elements", mesh.nodes.len(), mesh.elements.len());
            eprintln!("     Memory optimization: Using sparse assembly");

            mapping
        } else {
            std::collections::HashMap::new()
        };

        mesh.calculate_dofs();
        let mesh_stats = mesh.statistics();

        // Step 2: Build boundary conditions and loads
        let mut bcs = crate::bc_builder::BCBuilder::build_from_deck(deck)?;

        // Step 2.5: Transfer BCs and loads if beam expansion was used
        if !beam_node_mapping.is_empty() {
            eprintln!("  🔄 Transferring BCs and loads to expanded nodes...");
            eprintln!("     Original: {} disp BCs, {} loads", bcs.displacement_bcs.len(), bcs.concentrated_loads.len());
            let transfer = crate::bc_transfer::BCTransfer::new(beam_node_mapping.clone());
            bcs = transfer.transfer_all(&bcs);
            eprintln!("     Transferred: {} disp BCs, {} loads", bcs.displacement_bcs.len(), bcs.concentrated_loads.len());
            eprintln!("     {}", transfer.statistics());

            // Constrain orphan nodes (nodes not referenced by any element)
            let referenced_nodes: std::collections::HashSet<i32> = mesh.elements.values()
                .flat_map(|e| e.nodes.iter().copied())
                .collect();
            let mut orphan_count = 0;
            for &node_id in mesh.nodes.keys() {
                if !referenced_nodes.contains(&node_id) {
                    bcs.add_displacement_bc(crate::boundary_conditions::DisplacementBC::new(node_id, 1, 3, 0.0));
                    orphan_count += 1;
                }
            }
            if orphan_count > 0 {
                eprintln!("     Constrained {} orphan nodes ({} DOFs)", orphan_count, orphan_count * 3);
            }
        }

        eprintln!("  📊 Computing BC statistics...");
        let bc_stats = bcs.statistics();
        eprintln!("  📊 BC statistics computed");

        eprintln!("  🔍 Calculating constrained DOFs...");
        // Calculate constrained and free DOFs
        let constrained_dofs = bcs.get_constrained_dofs();
        eprintln!("  🔍 Found {} constrained DOFs", constrained_dofs.len());
        let free_dofs = mesh.num_dofs - constrained_dofs.len();
        eprintln!("  🔍 Free DOFs: {}", free_dofs);

        // For structural analysis with truss elements, attempt to solve
        let mut displacements = Vec::new();
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

                    // Step 4: Assemble and solve (supports multiple element types)
                    let has_supported_elements = mesh
                        .elements
                        .values()
                        .any(|e| matches!(
                            e.element_type,
                            crate::mesh::ElementType::T3D2
                                | crate::mesh::ElementType::T3D3
                                | crate::mesh::ElementType::B31
                                | crate::mesh::ElementType::B32
                                | crate::mesh::ElementType::S4
                                | crate::mesh::ElementType::S8
                                | crate::mesh::ElementType::C3D8
                                | crate::mesh::ElementType::C3D10
                                | crate::mesh::ElementType::C3D20
                        ));

                    if has_supported_elements {
                        // Use sparse assembly for expanded meshes or large systems
                        let use_sparse = use_expansion || mesh.nodes.len() > 100;

                        if use_sparse {
                            eprintln!("  ⚡ Using SPARSE assembly for {} nodes, {} elements", mesh.nodes.len(), mesh.elements.len());
                            match crate::sparse_assembly::SparseGlobalSystem::assemble(
                                &mesh, &materials, &bcs, 0.001,
                            ) {
                                Ok(system) => match system.solve() {
                                    Ok(solution) => {
                                        displacements = solution.as_slice().to_vec();
                                        " [SOLVED]".to_string()
                                    },
                                    Err(e) => format!(" [SOLVE FAILED: {}]", e),
                                },
                                Err(e) => format!(" [ASSEMBLY FAILED: {}]", e),
                            }
                        } else {
                            eprintln!("  🔧 Using DENSE assembly for {} nodes, {} elements", mesh.nodes.len(), mesh.elements.len());
                            match crate::assembly::GlobalSystem::assemble(
                                &mesh, &materials, &bcs, 0.001,
                            ) {
                                Ok(system) => match system.solve() {
                                    Ok(solution) => {
                                        displacements = solution.as_slice().to_vec();
                                        " [SOLVED]".to_string()
                                    },
                                    Err(e) => format!(" [SOLVE FAILED: {}]", e),
                                },
                                Err(e) => format!(" [ASSEMBLY FAILED: {}]", e),
                            }
                        }
                    } else {
                        " [no supported elements found - solver supports: T3D2, T3D3, B31, B32, S4, S8, C3D8, C3D10, C3D20]".to_string()
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
            displacements,
        })
    }

    /// Get the current configuration
    pub fn config(&self) -> &AnalysisConfig {
        &self.config
    }

    /// Check if mesh contains B32R beam elements
    fn has_b32r_elements(mesh: &crate::Mesh) -> bool {
        use crate::mesh::ElementType;
        mesh.elements.values().any(|elem| {
            elem.element_type == ElementType::B32
        })
    }

    /// Expand B32R beam elements to C3D20R solid elements
    fn expand_b32r_mesh(
        mesh: &crate::Mesh,
        deck: &Deck,
    ) -> Result<(crate::Mesh, std::collections::HashMap<i32, [i32; 8]>), String> {
        use crate::elements::{expand_b32r, BeamExpansionConfig, BeamSection, SectionShape};
        use crate::mesh::ElementType;
        use nalgebra::Vector3;
        use std::collections::HashMap;

        // Parse beam section and normal direction from deck
        let (section, normal) = Self::parse_beam_section_from_deck(deck)?;

        eprintln!("    Section: area={:.6}", section.area);
        eprintln!("    Normal: [{:.3}, {:.3}, {:.3}]", normal.x, normal.y, normal.z);

        // Initialize expansion config with actual max IDs from mesh
        let max_node_id = mesh.nodes.keys().max().copied().unwrap_or(0);
        let max_elem_id = mesh.elements.keys().max().copied().unwrap_or(0);
        let mut config = BeamExpansionConfig {
            next_node_id: max_node_id + 1,
            next_element_id: max_elem_id + 1,
        };
        eprintln!("    Starting new nodes at ID {}, elements at ID {}", config.next_node_id, config.next_element_id);

        // Create new mesh for expanded elements
        let mut expanded_mesh = crate::Mesh::new();

        // Collect all beam node mappings
        let mut beam_node_mapping: HashMap<i32, [i32; 8]> = HashMap::new();

        // Copy all original nodes
        for (id, node) in &mesh.nodes {
            expanded_mesh.add_node(node.clone());
        }

        // Process all elements
        let mut num_expanded = 0;
        for (elem_id, element) in &mesh.elements {
            // Check if this is a B32 element (includes B32R from INP files)
            if element.element_type == ElementType::B32 {

                if element.nodes.len() != 3 {
                    return Err(format!("B32R element {} has {} nodes, expected 3",
                                       elem_id, element.nodes.len()));
                }

                // Get the 3 beam nodes
                let beam_nodes: Vec<_> = element.nodes.iter()
                    .map(|&node_id| {
                        mesh.nodes.get(&node_id)
                            .ok_or(format!("Node {} not found", node_id))
                            .map(|n| n.clone())
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                // Expand to C3D20R
                let result = expand_b32r(
                    element,
                    &[beam_nodes[0].clone(), beam_nodes[1].clone(), beam_nodes[2].clone()],
                    &section,
                    normal,
                    &mut config,
                )?;

                // Collect beam node mappings from this expansion
                for (beam_node_id, section_nodes) in &result.beam_node_mapping {
                    beam_node_mapping.insert(*beam_node_id, *section_nodes);
                }

                // Add expanded nodes and elements
                for (_, node) in &result.nodes {
                    expanded_mesh.add_node(node.clone());
                }
                for (_, elem) in &result.elements {
                    expanded_mesh.add_element(elem.clone());
                }

                num_expanded += 1;
            } else {
                // Copy non-beam element as-is
                expanded_mesh.add_element(element.clone());
            }
        }

        eprintln!("    Expanded {} B32R elements", num_expanded);
        eprintln!("    Nodes: {} → {}", mesh.nodes.len(), expanded_mesh.nodes.len());
        eprintln!("    Elements: {} → {}", mesh.elements.len(), expanded_mesh.elements.len());
        eprintln!("    Beam node mapping: {} beam nodes → {} section nodes",
                  beam_node_mapping.len(), beam_node_mapping.len() * 8);

        expanded_mesh.validate()?;

        Ok((expanded_mesh, beam_node_mapping))
    }

    /// Parse beam section and normal direction from INP deck
    fn parse_beam_section_from_deck(deck: &Deck) -> Result<(BeamSection, Vector3<f64>), String> {
        use crate::elements::{BeamSection, SectionShape};
        use nalgebra::Vector3;

        // Find BEAM SECTION card
        for card in &deck.cards {
            if card.keyword.to_uppercase() == "BEAM SECTION" {
                // Parse section type from parameters
                let mut section_type = "RECT"; // default
                for param in &card.parameters {
                    if param.key.to_uppercase() == "SECTION" {
                        if let Some(ref val) = param.value {
                            section_type = val.as_str();
                        }
                    }
                }

                // Parse dimensions from first data line
                if card.data_lines.is_empty() {
                    return Err("BEAM SECTION card missing dimension data".to_string());
                }

                let dims_line = &card.data_lines[0];
                let dims: Vec<f64> = dims_line
                    .split(',')
                    .filter_map(|s| s.trim().parse::<f64>().ok())
                    .collect();

                if dims.len() < 2 {
                    return Err(format!("BEAM SECTION requires at least 2 dimensions, got {}", dims.len()));
                }

                // Parse normal direction from second data line
                let normal = if card.data_lines.len() >= 2 {
                    let normal_line = &card.data_lines[1];
                    let normal_vals: Vec<f64> = normal_line
                        .split(',')
                        .filter_map(|s| s.trim().parse::<f64>().ok())
                        .collect();

                    if normal_vals.len() >= 3 {
                        Vector3::new(normal_vals[0], normal_vals[1], normal_vals[2])
                    } else {
                        eprintln!("    Warning: Beam normal not fully specified, using default [1, 0, 0]");
                        Vector3::new(1.0, 0.0, 0.0)
                    }
                } else {
                    eprintln!("    Warning: Beam normal not specified, using default [1, 0, 0]");
                    Vector3::new(1.0, 0.0, 0.0)
                };

                // Create beam section based on type
                let section = match section_type.to_uppercase().as_str() {
                    "RECT" => {
                        let width = dims[0];
                        let height = dims[1];
                        BeamSection {
                            shape: SectionShape::Rectangular { width, height },
                            area: width * height,
                            iyy: width * height.powi(3) / 12.0,
                            izz: height * width.powi(3) / 12.0,
                            torsion_constant: 0.0,
                            shear_area_y: None,
                            shear_area_z: None,
                        }
                    }
                    "CIRC" => {
                        let radius = dims[0];
                        let area = std::f64::consts::PI * radius.powi(2);
                        BeamSection {
                            shape: SectionShape::Circular { radius },
                            area,
                            iyy: std::f64::consts::PI * radius.powi(4) / 4.0,
                            izz: std::f64::consts::PI * radius.powi(4) / 4.0,
                            torsion_constant: 0.0,
                            shear_area_y: None,
                            shear_area_z: None,
                        }
                    }
                    _ => return Err(format!("Unsupported beam section type: {}", section_type)),
                };

                return Ok((section, normal));
            }
        }

        Err("No BEAM SECTION card found in deck".to_string())
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
