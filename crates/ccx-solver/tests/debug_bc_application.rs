use ccx_solver::*;

#[test]
fn debug_bc_application_detailed() {
    // Create simple 2-node truss
    let mut mesh = Mesh::new();
    mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
    mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));

    let elem = Element::new(1, ElementType::T3D2, vec![1, 2]);
    let _ = mesh.add_element(elem);
    mesh.calculate_dofs();

    // Create material
    let mut materials = MaterialLibrary::new();
    let mut steel = Material::new("STEEL".to_string());
    steel.elastic_modulus = Some(210000.0);
    steel.poissons_ratio = Some(0.3);
    materials.add_material(steel);
    materials.assign_material(1, "STEEL".to_string());

    // Boundary conditions - exactly like the failing test
    let mut bcs = BoundaryConditions::new();
    // Fix node 1 in all directions
    bcs.add_displacement_bc(DisplacementBC::new(1, 1, 3, 0.0));
    // Fix node 2 in y and z directions (truss along x-axis doesn't constrain these)
    bcs.add_displacement_bc(DisplacementBC::new(2, 2, 3, 0.0));
    // Apply force to node 2 in x-direction
    bcs.add_concentrated_load(ConcentratedLoad::new(2, 1, 1000.0));

    let area = 0.01; // 100 cm²

    println!("Assembling sparse system with BCs...\n");

    let sparse_system = SparseGlobalSystem::assemble(&mesh, &materials, &bcs, area)
        .expect("Sparse assembly should succeed");

    println!("Sparse system:");
    println!("  num_dofs: {}", sparse_system.num_dofs);
    println!("  nnz: {}", sparse_system.stiffness.nnz());
    println!("  constrained_dofs: {:?}", sparse_system.constrained_dofs);
    println!();

    // Check all diagonal entries
    println!("Diagonal entries:");
    for i in 0..sparse_system.num_dofs {
        if let Some(row) = sparse_system.stiffness.get_row(i) {
            if let Some(pos) = row.col_indices().iter().position(|&col| col == i) {
                let diag_val = row.values()[pos];
                println!("  K[{}, {}] = {:.3e}", i, i, diag_val);
            } else {
                println!("  K[{}, {}] = MISSING", i, i);
            }
        }
    }
    println!();

    // Print force vector
    println!("Force vector:");
    for i in 0..sparse_system.num_dofs {
        println!("  F[{}] = {:.3e}", i, sparse_system.force[i]);
    }
    println!();

    // Print full matrix structure
    println!("Full matrix structure:");
    for i in 0..sparse_system.num_dofs {
        print!("Row {}: ", i);
        if let Some(row) = sparse_system.stiffness.get_row(i) {
            for (&col, &val) in row.col_indices().iter().zip(row.values().iter()) {
                print!("  ({}, {:.2e})", col, val);
            }
        }
        println!();
    }
    println!();

    // Try to solve
    match sparse_system.solve() {
        Ok(u) => {
            println!("Solution succeeded!");
            println!("\nDisplacements:");
            for i in 0..sparse_system.num_dofs {
                println!("  u[{}] = {:.6e}", i, u[i]);
            }

            // Check constraints
            println!("\nConstraint verification:");
            for &dof in &sparse_system.constrained_dofs {
                println!("  DOF {} (constrained): u = {:.6e}", dof, u[dof]);
            }

            // Analytical solution: u = FL/(AE) = 1000 * 1.0 / (0.01 * 210000) = 0.000476 m
            let expected = 1000.0 / (0.01 * 210000.0);
            let rel_error = ((u[3] - expected) / expected).abs();
            println!("\nAnalytical comparison:");
            println!("  Expected u[3] = {:.6e}", expected);
            println!("  Actual u[3]   = {:.6e}", u[3]);
            println!("  Relative error = {:.2}%", rel_error * 100.0);
        }
        Err(e) => {
            println!("Solve failed: {}", e);
        }
    }
}
