use ccx_solver::*;

#[test]
fn debug_sparse_assembly() {
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
    steel.elastic_modulus = Some(210000.0); // MPa
    steel.poissons_ratio = Some(0.3);
    materials.add_material(steel);
    materials.assign_material(1, "STEEL".to_string());

    // Boundary conditions
    let mut bcs = BoundaryConditions::new();
    // Fix node 1 in all directions
    bcs.add_displacement_bc(DisplacementBC::new(1, 1, 3, 0.0));
    // Apply force to node 2
    bcs.add_concentrated_load(ConcentratedLoad::new(2, 1, 1000.0));

    let area = 0.01; // 100 cm²

    // Assemble using sparse
    let sparse_system = SparseGlobalSystem::assemble(&mesh, &materials, &bcs, area)
        .expect("Sparse assembly should succeed");

    println!("Sparse system:");
    println!("  num_dofs: {}", sparse_system.num_dofs);
    println!("  nnz: {}", sparse_system.stiffness.nnz());
    println!("  constrained_dofs: {:?}", sparse_system.constrained_dofs);

    // Check if diagonal entries exist
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

    // Print force vector
    println!("\nForce vector:");
    for i in 0..sparse_system.num_dofs {
        println!("  F[{}] = {:.3e}", i, sparse_system.force[i]);
    }

    // Try to solve
    match sparse_system.solve() {
        Ok(u) => {
            println!("\nDisplacements:");
            for i in 0..sparse_system.num_dofs {
                println!("  u[{}] = {:.6e}", i, u[i]);
            }
        }
        Err(e) => {
            println!("\nSolve failed: {}", e);
        }
    }

    // Compare with dense assembly
    let dense_system = GlobalSystem::assemble(&mesh, &materials, &bcs, area)
        .expect("Dense assembly should succeed");

    println!("\nDense system:");
    println!("  num_dofs: {}", dense_system.num_dofs);

    for i in 0..dense_system.num_dofs {
        println!("  K[{}, {}] = {:.3e}", i, i, dense_system.stiffness[(i, i)]);
    }

    match dense_system.solve() {
        Ok(u) => {
            println!("\nDense displacements:");
            for i in 0..dense_system.num_dofs {
                println!("  u[{}] = {:.6e}", i, u[i]);
            }
        }
        Err(e) => {
            println!("\nDense solve failed: {}", e);
        }
    }
}
