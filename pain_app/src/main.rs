use pain_core::SimulationState;
use pain_graphics::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;

const SIM_WIDTH: f32 = 1000.0;
const SIM_HEIGHT: f32 = 720.0;

fn main() -> Result<(), String> {
    // --- SDL2 Initialization ---
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    // --- Renderer Initialization ---
    let mut renderer = Renderer::new(&sdl_context, &video_subsystem, &ttf_context)?;

    // --- Simulation State Initialization ---
    let mut sim_state = SimulationState::new(SIM_WIDTH, SIM_HEIGHT);
    sim_state.initialize_classic_recipe();

    println!("Bread Simulator - House of Pain initialized!");
    println!("Starting graphical simulation...");

    // --- Main Loop ---
    let mut event_pump = sdl_context.event_pump()?;
    let mut last_time = Instant::now();

    'running: loop {
        // --- Event Handling ---
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    if !sim_state.salt_added {
                        sim_state.add_salt();
                        println!("Salt added!");
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Y),
                    ..
                } => {
                    if !sim_state.yeast_added {
                        sim_state.add_yeast();
                        println!("Yeast added!");
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::C), // 'C' for 'Coil Fold'
                    ..
                } => {
                    // Apply a swirling force to simulate folding
                    let center = nalgebra::Vector2::new(SIM_WIDTH / 2.0, SIM_HEIGHT / 2.0);
                    let force = nalgebra::Vector2::new(0.0, 30.0); // Downward pull
                    sim_state.apply_force_to_region(center, 200.0, force);
                    println!("Fold applied!");
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    println!("Resetting simulation...");
                    sim_state = SimulationState::new(SIM_WIDTH, SIM_HEIGHT);
                    sim_state.initialize_classic_recipe();
                }
                _ => {}
            }
        }

        // --- Time Management ---
        let now = Instant::now();
        let dt = (now - last_time).as_secs_f32();
        last_time = now;
        let dt = dt.min(0.05); // Cap delta time to prevent physics explosion

        // --- Simulation Update ---
        sim_state.tick(dt);

        // --- Drawing ---
        renderer.draw(&sim_state)?;

        // A short delay to not fry the CPU
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    println!("Simulation finished. Goodbye!");
    Ok(())
}
