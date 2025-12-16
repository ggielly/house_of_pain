use bevy::prelude::*;
use pain_bevy_visualizer::{ParticlePlugin, SimulationResource};
use pain_core::SimulationState;

const SIM_WIDTH: f32 = 1000.0;
const SIM_HEIGHT: f32 = 720.0;
const SIM_DEPTH: f32 = 1000.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
        ))
        .add_plugins(ParticlePlugin)
        .add_systems(Update, update_simulation_state)
        .run();
}

fn create_initial_simulation() -> SimulationState {
    let mut sim_state = SimulationState::new(SIM_WIDTH as f32, SIM_HEIGHT as f32, SIM_DEPTH as f32);
    sim_state.initialize_classic_recipe();
    sim_state
}

// Système pour mettre à jour l'état de la simulation à chaque frame
fn update_simulation_state(
    mut sim_resource: ResMut<SimulationResource>,
    time: Res<Time>,
) {
    // Mettre à jour la simulation avec le delta time
    sim_resource.state.tick(time.delta_seconds());
}