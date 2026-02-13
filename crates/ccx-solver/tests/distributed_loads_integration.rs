//! Integration tests for distributed loads (pressure, traction, body forces)
//!
//! These tests validate the complete workflow:
//! 1. Define distributed load (DistributedLoad struct)
//! 2. Convert to nodal forces (DistributedLoadConverter)
//! 3. Assemble into global system
//! 4. Solve for displacements
//! 5. Compare with analytical solutions

use ccx_solver::{
    BoundaryConditions, DistributedLoadConverter, GlobalSystem, Material, MaterialLibrary,
    MaterialModel, Mesh, Node,
};

#[cfg(test)]
mod tests {
    use super::*;
    use ccx_solver::boundary_conditions::{DistributedLoad, DistributedLoadType};
    use ccx_solver::mesh::{Element, ElementType};

    /// Create a steel material for tests
    fn steel_material() -> Material {
        Material {
            name: "STEEL".to_string(),
            model: MaterialModel::LinearElastic,
            elastic_modulus: Some(200e9), // 200 GPa
            poissons_ratio: Some(0.3),
            density: Some(7850.0),
            thermal_expansion: None,
            conductivity: None,
            specific_heat: None,
        }
    }

    /// Create a single S4 plate mesh (1m × 1m in XY plane)
    fn make_single_plate_mesh() -> Mesh {
        let mut mesh = Mesh::new();

        // 4 corner nodes
        mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
        mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));
        mesh.add_node(Node::new(3, 1.0, 1.0, 0.0));
        mesh.add_node(Node::new(4, 0.0, 1.0, 0.0));

        // Single S4 element
        let _ = mesh.add_element(Element::new(1, ElementType::S4, vec![1, 2, 3, 4]));

        mesh.calculate_dofs();
        mesh
    }

    /// Create a 2×2 plate mesh (1m × 1m total, 4 elements)
    fn make_2x2_plate_mesh() -> Mesh {
        let mut mesh = Mesh::new();

        // 9 nodes in a 3×3 grid
        for i in 0..3 {
            for j in 0..3 {
                let node_id = (i * 3 + j + 1) as i32;
                let x = j as f64 * 0.5;
                let y = i as f64 * 0.5;
                mesh.add_node(Node::new(node_id, x, y, 0.0));
            }
        }

        // 4 S4 elements
        // Element connectivity (counter-clockwise):
        // Elem 1: nodes 1-2-5-4
        // Elem 2: nodes 2-3-6-5
        // Elem 3: nodes 4-5-8-7
        // Elem 4: nodes 5-6-9-8
        let _ = mesh.add_element(Element::new(1, ElementType::S4, vec![1, 2, 5, 4]));
        let _ = mesh.add_element(Element::new(2, ElementType::S4, vec![2, 3, 6, 5]));
        let _ = mesh.add_element(Element::new(3, ElementType::S4, vec![4, 5, 8, 7]));
        let _ = mesh.add_element(Element::new(4, ElementType::S4, vec![5, 6, 9, 8]));

        mesh.calculate_dofs();
        mesh
    }

    #[test]
    fn single_plate_uniform_pressure() {
        // Test: Single S4 plate under uniform pressure
        // Verify that:
        // 1. Pressure converts to correct nodal forces
        // 2. Force conservation (total = pressure × area)
        // 3. Assembly integrates forces correctly

        let mesh = make_single_plate_mesh();

        let mut materials = MaterialLibrary::new();
        materials.add_material(steel_material());
        materials.assign_material(1, "STEEL".to_string());

        let mut bcs = BoundaryConditions::new();

        // Apply pressure load on element 1
        let pressure = 1000.0; // 1000 Pa
        bcs.add_distributed_load(DistributedLoad {
            element: "1".to_string(),
            load_type: DistributedLoadType::Pressure,
            magnitude: pressure,
            parameters: vec![],
        });

        // Keep an unconstrained copy to verify assembled load magnitudes directly.
        let load_only_bcs = bcs.clone();

        // Fix all edges (simply supported approximation)
        // In a full simply supported case, we'd allow in-plane movement
        // For this test, we just prevent rigid body motion
        for node_id in [1, 2, 3, 4] {
            bcs.add_displacement_bc(ccx_solver::boundary_conditions::DisplacementBC::new(
                node_id, 1, 6, 0.0,
            ));
        }

        let thickness = 0.01; // 1 cm
        let load_system = GlobalSystem::assemble(&mesh, &materials, &load_only_bcs, thickness).unwrap();
        let system = GlobalSystem::assemble(&mesh, &materials, &bcs, thickness).unwrap();

        // Verify force conservation
        // Total force = pressure × area = 1000 Pa × 1 m² = 1000 N
        let total_z_force: f64 = (0..mesh.nodes.len())
            .map(|i| load_system.force[i * 6 + 2]) // z-component (DOF 3)
            .sum();

        let expected_total = -pressure * 1.0; // Negative because compression
        assert!(
            (total_z_force - expected_total).abs() < 1.0,
            "Total z-force {} should be ~{} N",
            total_z_force,
            expected_total
        );

        // Solve system
        let result = system.solve();
        assert!(
            result.is_ok(),
            "Should solve successfully: {:?}",
            result.err()
        );

        let displacements = result.unwrap();

        // All displacements should be zero (fully constrained)
        for i in 0..displacements.len() {
            assert!(
                displacements[i].abs() < 1e-6,
                "Displacement at DOF {} should be ~0, got {}",
                i,
                displacements[i]
            );
        }
    }

    #[test]
    fn multiple_element_pressure_load() {
        // Test: 2×2 mesh with pressure on all elements
        // Verify that forces accumulate correctly at shared nodes

        let mesh = make_2x2_plate_mesh();

        let mut materials = MaterialLibrary::new();
        materials.add_material(steel_material());
        for elem_id in 1..=4 {
            materials.assign_material(elem_id, "STEEL".to_string());
        }

        let mut bcs = BoundaryConditions::new();

        // Apply pressure to all 4 elements
        let pressure = 1000.0; // 1000 Pa
        for elem_id in 1..=4 {
            bcs.add_distributed_load(DistributedLoad {
                element: elem_id.to_string(),
                load_type: DistributedLoadType::Pressure,
                magnitude: pressure,
                parameters: vec![],
            });
        }

        let thickness = 0.01; // 1 cm
        let system = GlobalSystem::assemble(&mesh, &materials, &bcs, thickness).unwrap();

        // Verify force conservation
        // Total force = pressure × total_area = 1000 Pa × 1 m² = 1000 N
        let total_z_force: f64 = (0..mesh.nodes.len())
            .map(|i| system.force[i * 6 + 2]) // z-component
            .sum();

        let expected_total = -pressure * 1.0; // Negative (compression)
        assert!(
            (total_z_force - expected_total).abs() < 1.0,
            "Total z-force {} should be ~{} N",
            total_z_force,
            expected_total
        );

        // Verify that interior node (node 5) has contributions from all 4 elements
        // It should have roughly 4× the force of a corner node
        let node5_dof = 4 * 6 + 2; // Node 5 z-DOF
        let node5_force = system.force[node5_dof].abs();

        let node1_dof = 0 * 6 + 2; // Node 1 z-DOF
        let node1_force = system.force[node1_dof].abs();

        // Node 5 is shared by 4 elements, node 1 by 1 element
        // Ratio should be ~4 (may vary slightly due to integration)
        let ratio = node5_force / node1_force;
        assert!(
            ratio > 3.0 && ratio < 5.0,
            "Force ratio {} should be ~4",
            ratio
        );
    }

    #[test]
    fn concentrated_plus_distributed_loads() {
        // Test: Combined concentrated and distributed loads
        // Verify that both load types accumulate correctly

        let mesh = make_single_plate_mesh();

        let mut materials = MaterialLibrary::new();
        materials.add_material(steel_material());
        materials.assign_material(1, "STEEL".to_string());

        let mut bcs = BoundaryConditions::new();

        // Add pressure load
        let pressure = 1000.0; // 1000 Pa
        bcs.add_distributed_load(DistributedLoad {
            element: "1".to_string(),
            load_type: DistributedLoadType::Pressure,
            magnitude: pressure,
            parameters: vec![],
        });

        // Add concentrated load at node 1 in z-direction
        let concentrated_force = 500.0; // 500 N
        bcs.add_concentrated_load(ccx_solver::boundary_conditions::ConcentratedLoad::new(
            1,
            3,
            concentrated_force,
        ));

        let thickness = 0.01;
        let system = GlobalSystem::assemble(&mesh, &materials, &bcs, thickness).unwrap();

        // Node 1 should have both pressure contribution and concentrated load
        let node1_z_dof = 0 * 6 + 2;
        let node1_force = system.force[node1_z_dof];

        // Pressure contributes ~-250 N (1/4 of total), concentrated adds +500 N
        // Net should be ~250 N
        assert!(
            node1_force > 200.0 && node1_force < 300.0,
            "Node 1 z-force {} should be ~250 N (combined loads)",
            node1_force
        );

        // Other nodes should only have pressure contribution (~-250 N each)
        for node_id in [2, 3, 4] {
            let z_dof = ((node_id - 1) as usize) * 6 + 2;
            let force = system.force[z_dof];
            assert!(
                force > -300.0 && force < -200.0,
                "Node {} z-force {} should be ~-250 N (pressure only)",
                node_id,
                force
            );
        }
    }

    #[test]
    fn pressure_load_force_conservation() {
        // Rigorous test of force conservation for pressure loads
        // Independent of boundary conditions

        let mesh = make_single_plate_mesh();

        let mut materials = MaterialLibrary::new();
        materials.add_material(steel_material());
        materials.assign_material(1, "STEEL".to_string());

        // Use converter directly (bypass assembly)
        let converter = DistributedLoadConverter::new(&mesh, &materials);

        let pressure = 5000.0; // 5000 Pa
        let load = DistributedLoad {
            element: "1".to_string(),
            load_type: DistributedLoadType::Pressure,
            magnitude: pressure,
            parameters: vec![],
        };

        let nodal_forces = converter.convert_to_nodal_forces(&load).unwrap();

        // Sum all z-components
        let total_z_force: f64 = nodal_forces.values().map(|f| f[2]).sum();

        // Should equal pressure × area
        let plate_area = 1.0 * 1.0; // 1 m²
        let expected_total = -pressure * plate_area; // Negative (compression)

        let error = (total_z_force - expected_total).abs();
        let relative_error = error / expected_total.abs();

        assert!(
            relative_error < 1e-6,
            "Force conservation error: {:.2e}% (expected 0%)",
            relative_error * 100.0
        );
    }
}
