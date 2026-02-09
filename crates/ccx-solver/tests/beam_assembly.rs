/// End-to-end integration tests for beam element assembly
///
/// Tests complete workflow: mesh → assembly → boundary conditions → solve
/// Validates against analytical solutions for classical beam problems

use ccx_solver::{
    BoundaryConditions, ConcentratedLoad, DisplacementBC, GlobalSystem, Material,
    MaterialLibrary, MaterialModel, Mesh,
};

/// Helper to create a steel material
fn steel_material() -> Material {
    Material {
        name: "Steel".to_string(),
        model: MaterialModel::LinearElastic,
        elastic_modulus: Some(200e9), // 200 GPa
        poissons_ratio: Some(0.3),
        density: Some(7850.0), // kg/m³
        thermal_expansion: None,
        conductivity: None,
        specific_heat: None,
    }
}

#[test]
fn test_single_beam_cantilever() {
    // Cantilever beam: Fixed at node 1, free at node 2
    // Load: 1000 N downward (negative z) at node 2
    //
    // Analytical solution:
    // δ = PL³/(3EI)
    //
    // Parameters:
    // L = 1.0 m
    // r = 0.05 m (circular section)
    // E = 200 GPa
    // I = πr⁴/4 = 4.9087e-6 m⁴
    // P = 1000 N
    //
    // Expected: δ = 1000 * 1³ / (3 * 200e9 * 4.9087e-6) = 3.395e-4 m

    let length = 1.0;
    let radius = 0.05;
    let load_magnitude = 1000.0;

    // Create mesh with one beam element
    use ccx_solver::{Element, ElementType, Node};
    let mut mesh = Mesh::new();

    // Add nodes
    mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
    mesh.add_node(Node::new(2, length, 0.0, 0.0));

    // Add beam element
    let _ = mesh.add_element(Element::new(1, ElementType::B31, vec![1, 2]));

    // Calculate DOFs
    mesh.calculate_dofs();

    println!("\n=== Cantilever Beam Assembly Test ===");
    println!("Nodes: {}", mesh.nodes.len());
    println!("Elements: {}", mesh.elements.len());
    println!("Total DOFs: {}", mesh.num_dofs);

    // Create material library
    let mut materials = MaterialLibrary::new();
    let steel = steel_material();
    materials.add_material(steel.clone());
    materials.assign_material(1, "Steel".to_string());

    // Create boundary conditions
    let mut bcs = BoundaryConditions::new();

    // Fix all DOFs at node 1 (cantilever support)
    bcs.add_displacement_bc(DisplacementBC::new(1, 1, 6, 0.0));

    // Apply downward force at node 2 (z-direction, DOF 3)
    bcs.add_concentrated_load(ConcentratedLoad::new(2, 3, -load_magnitude));

    // Assemble system
    let default_area = std::f64::consts::PI * radius * radius;
    let system = GlobalSystem::assemble(&mesh, &materials, &bcs, default_area)
        .expect("Failed to assemble system");

    println!("Assembly successful!");
    println!("Global stiffness matrix: {} × {}", system.stiffness.nrows(), system.stiffness.ncols());
    println!("Force vector size: {}", system.force.len());

    // Verify system is valid
    system.validate().expect("System validation failed");
    println!("System validation passed!");

    // Solve for displacements
    let displacements = system.solve().expect("Failed to solve system");

    println!("\n=== Solution ===");
    println!("Node 1 (fixed):");
    println!("  ux: {:.6e} m", displacements[0]);
    println!("  uy: {:.6e} m", displacements[1]);
    println!("  uz: {:.6e} m", displacements[2]);
    println!("  rx: {:.6e} rad", displacements[3]);
    println!("  ry: {:.6e} rad", displacements[4]);
    println!("  rz: {:.6e} rad", displacements[5]);

    println!("\nNode 2 (free end):");
    println!("  ux: {:.6e} m", displacements[6]);
    println!("  uy: {:.6e} m", displacements[7]);
    println!("  uz: {:.6e} m", displacements[8]);
    println!("  rx: {:.6e} rad", displacements[9]);
    println!("  ry: {:.6e} rad", displacements[10]);
    println!("  rz: {:.6e} rad", displacements[11]);

    // Analytical solution
    let e = steel.elastic_modulus.unwrap();
    let i = std::f64::consts::PI * radius.powi(4) / 4.0;
    let analytical_deflection = (load_magnitude * length.powi(3)) / (3.0 * e * i);

    println!("\n=== Validation ===");
    println!("Analytical deflection: {:.6e} m", analytical_deflection);
    println!("Computed deflection:   {:.6e} m", displacements[8].abs());

    let error = ((displacements[8].abs() - analytical_deflection) / analytical_deflection).abs();
    println!("Relative error: {:.2e}%", error * 100.0);

    // Node 1 should be fixed (all displacements ≈ 0)
    assert!(displacements[0].abs() < 1e-6, "Node 1 ux not fixed");
    assert!(displacements[1].abs() < 1e-6, "Node 1 uy not fixed");
    assert!(displacements[2].abs() < 1e-6, "Node 1 uz not fixed");
    assert!(displacements[3].abs() < 1e-6, "Node 1 rx not fixed");
    assert!(displacements[4].abs() < 1e-6, "Node 1 ry not fixed");
    assert!(displacements[5].abs() < 1e-6, "Node 1 rz not fixed");

    // Node 2 z-displacement should match analytical solution
    assert!(
        error < 0.01,
        "Cantilever deflection error too large: {:.2e}%",
        error * 100.0
    );

    println!("✓ Test passed!");
}

#[test]
fn test_two_beam_structure() {
    // Two-beam structure: L-shaped
    // Node 1 at origin (fixed)
    // Node 2 at (1, 0, 0)
    // Node 3 at (1, 1, 0)
    // Beam 1: 1→2 (horizontal)
    // Beam 2: 2→3 (vertical)
    // Load: 500 N in -z direction at node 3

    use ccx_solver::{Element, ElementType, Node};
    let mut mesh = Mesh::new();

    // Add nodes
    mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
    mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));
    mesh.add_node(Node::new(3, 1.0, 1.0, 0.0));

    // Add beam elements
    let _ = mesh.add_element(Element::new(1, ElementType::B31, vec![1, 2]));
    let _ = mesh.add_element(Element::new(2, ElementType::B31, vec![2, 3]));

    mesh.calculate_dofs();

    println!("\n=== Two-Beam Structure Test ===");
    println!("Nodes: {}", mesh.nodes.len());
    println!("Elements: {}", mesh.elements.len());
    println!("Total DOFs: {}", mesh.num_dofs);

    // Create material library
    let mut materials = MaterialLibrary::new();
    let steel = steel_material();
    materials.add_material(steel);
    materials.assign_material(1, "Steel".to_string());
    materials.assign_material(2, "Steel".to_string());

    // Boundary conditions
    let mut bcs = BoundaryConditions::new();

    // Fix all DOFs at node 1
    bcs.add_displacement_bc(DisplacementBC::new(1, 1, 6, 0.0));

    // Apply load at node 3
    bcs.add_concentrated_load(ConcentratedLoad::new(3, 3, -500.0));

    // Assemble and solve
    let radius = 0.05;
    let default_area = std::f64::consts::PI * radius * radius;
    let system = GlobalSystem::assemble(&mesh, &materials, &bcs, default_area)
        .expect("Failed to assemble system");

    system.validate().expect("System validation failed");
    let displacements = system.solve().expect("Failed to solve system");

    println!("\n=== Solution ===");
    println!("Node 1: uz = {:.6e} m", displacements[2]);
    println!("Node 2: uz = {:.6e} m", displacements[8]);
    println!("Node 3: uz = {:.6e} m", displacements[14]);

    // Basic sanity checks
    assert!(displacements[2].abs() < 1e-6, "Node 1 should be fixed");
    assert!(displacements[14].abs() > 1e-6, "Node 3 should deflect");
    assert!(displacements[14] < 0.0, "Node 3 should deflect downward");

    println!("✓ Test passed!");
}

#[test]
fn test_mixed_truss_and_beam() {
    // Mixed element structure:
    // - Truss element: nodes 1→2 (horizontal bar)
    // - Beam element: nodes 3→4 (separate cantilever)
    //
    // This tests that the assembly system correctly handles mixed DOF counts:
    // - Truss uses DOFs 1-3 (translations only)
    // - Beam uses DOFs 1-6 (translations + rotations)
    // Both can coexist in the same mesh with proper DOF allocation

    use ccx_solver::{Element, ElementType, Node};
    let mut mesh = Mesh::new();

    // Add nodes for truss (1→2)
    mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
    mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));

    // Add nodes for beam (3→4)
    mesh.add_node(Node::new(3, 0.0, 1.0, 0.0));
    mesh.add_node(Node::new(4, 1.0, 1.0, 0.0));

    // Add mixed elements
    let _ = mesh.add_element(Element::new(1, ElementType::T3D2, vec![1, 2])); // Truss
    let _ = mesh.add_element(Element::new(2, ElementType::B31, vec![3, 4])); // Beam

    mesh.calculate_dofs();

    println!("\n=== Mixed Truss-Beam Test ===");
    println!("Nodes: {}", mesh.nodes.len());
    println!("Elements: {}", mesh.elements.len());
    println!("  Element 1: T3D2 (truss, 3 DOFs/node)");
    println!("  Element 2: B31 (beam, 6 DOFs/node)");
    println!("Total DOFs: {} (4 nodes × 6 DOFs/node)", mesh.num_dofs);

    // Create material library
    let mut materials = MaterialLibrary::new();
    let steel = steel_material();
    materials.add_material(steel);
    materials.assign_material(1, "Steel".to_string());
    materials.assign_material(2, "Steel".to_string());

    // Boundary conditions
    let mut bcs = BoundaryConditions::new();

    // Fix node 1 (truss support)
    bcs.add_displacement_bc(DisplacementBC::new(1, 1, 6, 0.0));

    // Fix node 2 in y and z (truss in x only)
    bcs.add_displacement_bc(DisplacementBC::new(2, 2, 6, 0.0));

    // Fix node 3 (beam support)
    bcs.add_displacement_bc(DisplacementBC::new(3, 1, 6, 0.0));

    // Apply loads
    bcs.add_concentrated_load(ConcentratedLoad::new(2, 1, 1000.0)); // Truss tension
    bcs.add_concentrated_load(ConcentratedLoad::new(4, 3, -500.0)); // Beam bending

    // Assemble and solve
    let default_area = 0.01; // 1 cm²
    let system = GlobalSystem::assemble(&mesh, &materials, &bcs, default_area)
        .expect("Failed to assemble system");

    println!("\nAssembly successful!");
    println!("Stiffness matrix: {} × {}", system.stiffness.nrows(), system.stiffness.ncols());

    system.validate().expect("System validation failed");
    let displacements = system.solve().expect("Failed to solve system");

    println!("\n=== Solution ===");
    println!("Truss element:");
    println!("  Node 1: ux = {:.6e} m (fixed)", displacements[0]);
    println!("  Node 2: ux = {:.6e} m (loaded)", displacements[6]);

    println!("\nBeam element:");
    println!("  Node 3: uz = {:.6e} m (fixed)", displacements[14]);
    println!("  Node 4: uz = {:.6e} m (loaded)", displacements[20]);

    // Verify truss behavior
    assert!(displacements[0].abs() < 1e-6, "Truss node 1 should be fixed");
    assert!(displacements[6] > 1e-8, "Truss node 2 should extend");

    // Verify beam behavior
    assert!(displacements[14].abs() < 1e-6, "Beam node 3 should be fixed");
    assert!(displacements[20] < -1e-8, "Beam node 4 should deflect downward");

    println!("✓ Mixed element assembly works correctly!");
}

#[test]
fn test_beam_with_moment_load() {
    // Beam with moment applied at free end
    // This tests rotational DOF loading (DOFs 4-6)

    use ccx_solver::{Element, ElementType, Node};
    let mut mesh = Mesh::new();

    mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
    mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));

    let _ = mesh.add_element(Element::new(1, ElementType::B31, vec![1, 2]));
    mesh.calculate_dofs();

    println!("\n=== Beam with Moment Load Test ===");

    // Material library
    let mut materials = MaterialLibrary::new();
    materials.add_material(steel_material());
    materials.assign_material(1, "Steel".to_string());

    // Boundary conditions
    let mut bcs = BoundaryConditions::new();

    // Fix node 1
    bcs.add_displacement_bc(DisplacementBC::new(1, 1, 6, 0.0));

    // Apply moment about y-axis at node 2 (DOF 5)
    // This should cause bending in the xz-plane
    bcs.add_concentrated_load(ConcentratedLoad::new(2, 5, 100.0)); // 100 Nm

    // Assemble and solve
    let radius = 0.05;
    let default_area = std::f64::consts::PI * radius * radius;
    let system = GlobalSystem::assemble(&mesh, &materials, &bcs, default_area)
        .expect("Failed to assemble system");

    system.validate().expect("System validation failed");
    let displacements = system.solve().expect("Failed to solve system");

    println!("\n=== Solution ===");
    println!("Node 2 rotation about y: {:.6e} rad", displacements[10]);
    println!("Node 2 z-displacement: {:.6e} m", displacements[8]);

    // The beam should rotate at the free end
    assert!(displacements[10].abs() > 1e-8, "Node 2 should rotate");

    println!("✓ Moment loading works correctly!");
}
