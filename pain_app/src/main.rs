use pain_core::SimulationState;
use std::time::Instant;

fn main() -> Result<(), String> {
    // Initialize the simulation with dimensions
    let mut sim_state = SimulationState::new(1000.0, 700.0);

    // Initialize with classic recipe
    sim_state.initialize_classic_recipe();

    println!("Bread Simulator - House of Pain initialized!");
    println!("Starting simulation with:");
    println!("  - Hydration: {:.2}%", sim_state.recipe_hydration * 100.0);
    println!("  - Salt: {:.2}%", sim_state.recipe_salt * 100.0);
    println!("  - Yeast: {:.2}%", sim_state.recipe_yeast * 100.0);
    println!("  - Temperature: {:.1}°C", sim_state.temperature);
    println!("Initial State:");
    println!("  - Molecules: {}", sim_state.grid.get_all_molecules().len());
    println!("  - Bonds: {}", sim_state.bonds.len());
    println!("\nPress Enter to start the simulation...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Main simulation loop for text-only output
    let mut last_time = Instant::now();
    let mut step_counter = 0;

    while step_counter < 1000 { // Run for 1000 steps
        // Calculate delta time
        let now = Instant::now();
        let dt = (now - last_time).as_secs_f32();
        last_time = now;

        // Cap delta time to prevent large jumps
        let dt = dt.min(0.1);

        // Apply some mixing force periodically
        if step_counter % 50 == 0 && step_counter > 100 {
            let center = nalgebra::Vector2::new(500.0, 350.0);
            let force = nalgebra::Vector2::new(10.0, 0.0);
            sim_state.apply_force_to_region(center, 100.0, force);
        }

        // Add salt at step 200
        if step_counter == 200 && !sim_state.salt_added {
            sim_state.add_salt();
            println!("Salt added at step {}!", step_counter);
        }

        // Add yeast at step 400
        if step_counter == 400 && !sim_state.yeast_added {
            sim_state.add_yeast();
            println!("Yeast added at step {}!", step_counter);
        }

        // Update simulation state
        sim_state.tick(dt);

        // Print status every 100 steps
        if step_counter % 100 == 0 {
            println!("Step {}: Time: {:.1}s | Molecules: {} | Bonds: {} | CO2: {}",
                step_counter,
                sim_state.time_elapsed,
                sim_state.grid.get_all_molecules().len(),
                sim_state.bonds.len(),
                sim_state.get_molecules_by_type(&pain_core::MoleculeType::CO2).len()
            );
        }

        step_counter += 1;
    }

    println!("Simulation completed!");
    println!("Final State:");
    println!("  - Time elapsed: {:.2}s", sim_state.time_elapsed);
    println!("  - Molecules: {}", sim_state.grid.get_all_molecules().len());
    println!("  - Bonds: {}", sim_state.bonds.len());
    println!("  - CO2 molecules: {}", sim_state.get_molecules_by_type(&pain_core::MoleculeType::CO2).len());
    println!("  - Glutenins with free thiol: {}",
        sim_state.get_molecules_by_type(&pain_core::MoleculeType::Glutenin { has_free_thiol: true }).len());

    Ok(())
}