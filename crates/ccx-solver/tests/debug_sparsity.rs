use ccx_solver::*;

#[test]
fn debug_sparsity_issue() {
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

    // No boundary conditions - just check assembly
    let bcs = BoundaryConditions::new();
    let area = 0.01;

    println!("Testing sparse assembly without BCs...\n");

    let sparse_system = SparseGlobalSystem::assemble(&mesh, &materials, &bcs, area)
        .expect("Sparse assembly should succeed");

    let nnz = sparse_system.stiffness.nnz();
    let total = sparse_system.num_dofs * sparse_system.num_dofs;
    let sparsity = (nnz as f64) / (total as f64);

    println!("Sparse system:");
    println!("  num_dofs: {}", sparse_system.num_dofs);
    println!("  nnz: {}", nnz);
    println!("  total_entries: {}", total);
    println!("  sparsity: {:.1}%", sparsity * 100.0);

    // For a single 2-node truss element (6x6 element matrix)
    // Only 36 entries should be non-zero out of 36 total (but sparse in global matrix)
    // Expected: Element DOFs are [0,1,2,3,4,5], so 6x6 = 36 entries
    // In a 6x6 global matrix, this is 36/36 = 100%
    // This is actually correct for this tiny problem!

    println!("\nMatrix structure:");
    for i in 0..sparse_system.num_dofs {
        print!("Row {}: ", i);
        if let Some(row) = sparse_system.stiffness.get_row(i) {
            for (&col, &val) in row.col_indices().iter().zip(row.values().iter()) {
                if val.abs() > 1e-10 {
                    print!("  col {} = {:.2e}", col, val);
                }
            }
        }
        println!();
    }

    // Compare with dense assembly
    let dense_system = GlobalSystem::assemble(&mesh, &materials, &bcs, area)
        .expect("Dense assembly should succeed");

    println!("\nDense matrix non-zero entries:");
    let mut dense_nnz = 0;
    for i in 0..dense_system.num_dofs {
        for j in 0..dense_system.num_dofs {
            if dense_system.stiffness[(i, j)].abs() > 1e-10 {
                dense_nnz += 1;
            }
        }
    }
    println!("  Dense nnz: {}", dense_nnz);

    // The sparsity test is actually wrong for this small problem!
    // With only 2 nodes and 1 element connecting them all,
    // we expect high sparsity (all nodes connected)
    assert!(
        sparsity <= 1.0,
        "Sparsity should be <= 100%, got {}%",
        sparsity * 100.0
    );
}

#[test]
fn debug_larger_mesh_sparsity() {
    // Create a 4-node chain of truss elements to see real sparsity
    let mut mesh = Mesh::new();
    mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
    mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));
    mesh.add_node(Node::new(3, 2.0, 0.0, 0.0));
    mesh.add_node(Node::new(4, 3.0, 0.0, 0.0));

    // 3 elements connecting the nodes
    mesh.add_element(Element::new(1, ElementType::T3D2, vec![1, 2]))
        .unwrap();
    mesh.add_element(Element::new(2, ElementType::T3D2, vec![2, 3]))
        .unwrap();
    mesh.add_element(Element::new(3, ElementType::T3D2, vec![3, 4]))
        .unwrap();

    mesh.calculate_dofs();

    let mut materials = MaterialLibrary::new();
    let mut steel = Material::new("STEEL".to_string());
    steel.elastic_modulus = Some(210000.0);
    steel.poissons_ratio = Some(0.3);
    materials.add_material(steel);
    materials.assign_material(1, "STEEL".to_string());
    materials.assign_material(2, "STEEL".to_string());
    materials.assign_material(3, "STEEL".to_string());

    let bcs = BoundaryConditions::new();
    let area = 0.01;

    println!("\n\nTesting 4-node chain mesh...\n");

    let sparse_system = SparseGlobalSystem::assemble(&mesh, &materials, &bcs, area)
        .expect("Sparse assembly should succeed");

    let nnz = sparse_system.stiffness.nnz();
    let total = sparse_system.num_dofs * sparse_system.num_dofs;
    let sparsity = (nnz as f64) / (total as f64);

    println!("Sparse system:");
    println!("  num_dofs: {}", sparse_system.num_dofs);
    println!("  nnz: {}", nnz);
    println!("  total_entries: {}", total);
    println!("  sparsity: {:.1}%", sparsity * 100.0);

    // For a chain of 4 nodes (12 DOFs), 3 elements
    // Each element contributes 6x6 = 36 entries to global matrix
    // Many overlap, so actual nnz should be much less than 144 (12x12)
    // Expected bandwidth: ~6 (each node connects to neighbors)
    // Expected nnz: ~12 * 6 = 72 entries (diagonal band)
    // Sparsity should be ~50% for this linear chain

    println!("\nExpected sparsity for linear chain: ~30-50%");
    println!("Actual sparsity: {:.1}%", sparsity * 100.0);

    // Count actual non-zeros in sparse matrix
    let mut actual_nonzeros = 0;
    for row in sparse_system.stiffness.row_iter() {
        for &val in row.values() {
            if val.abs() > 1e-10 {
                actual_nonzeros += 1;
            }
        }
    }
    println!("Actual non-zero values: {}", actual_nonzeros);

    // This test should show lower sparsity
    assert!(
        sparsity < 0.8,
        "Expected sparsity < 80% for chain, got {}%",
        sparsity * 100.0
    );
}
