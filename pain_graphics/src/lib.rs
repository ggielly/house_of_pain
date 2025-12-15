use sdl2::event::Event;
use sdl2::gfx::primitives::*;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use nalgebra::Vector2;
use pain_core::{SimulationState, MoleculeType};
use rand::Rng;

pub struct GraphicsEngine {
    sdl_context: sdl2::Sdl,
    canvas: Canvas<Window>,
    event_pump: sdl2::EventPump,
    running: bool,
    recipe_hydration: f32,
    recipe_salt: f32,
    recipe_yeast: f32,
    temperature: f32,
    autolyse_duration: f32,
    salt_added: bool,
    yeast_added: bool,
    apply_force_pressed: bool,
    mouse_position: (i32, i32),
}

impl GraphicsEngine {
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("Bread Simulator - House of Pain", width, height)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

        // Set draw color to a dark background
        canvas.set_draw_color(Color::RGB(30, 30, 40));
        canvas.clear();
        canvas.present();

        let event_pump = sdl_context.event_pump()?;

        Ok(GraphicsEngine {
            sdl_context,
            canvas,
            event_pump,
            running: true,
            recipe_hydration: 0.72,  // 72%
            recipe_salt: 0.02,       // 2%
            recipe_yeast: 0.20,      // 20%
            temperature: 25.0,       // 25°C
            autolyse_duration: 1800.0, // 30 minutes in seconds
            salt_added: false,
            yeast_added: false,
            apply_force_pressed: false,
            mouse_position: (0, 0),
        })
    }

    pub fn handle_events(&mut self) -> Result<(), String> {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    self.running = false;
                }
                Event::MouseButtonDown { x, y, .. } => {
                    self.mouse_position = (x, y);
                    self.apply_force_pressed = true;
                }
                Event::MouseMotion { x, y, .. } => {
                    self.mouse_position = (x, y);
                }
                Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Space), .. } => {
                    self.apply_force_pressed = true;
                }
                Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::S), .. } => {
                    self.salt_added = true;
                }
                Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Y), .. } => {
                    self.yeast_added = true;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn render(&mut self, sim_state: &SimulationState) -> Result<(), String> {
        self.canvas.set_draw_color(Color::RGB(30, 30, 40));
        self.canvas.clear();

        // Draw simulation elements
        self.render_simulation(sim_state)?;

        // Draw UI info on screen
        self.render_info(sim_state)?;

        self.canvas.present();

        Ok(())
    }

    fn render_simulation(&mut self, sim_state: &SimulationState) -> Result<(), String> {
        // Draw bonds first (as background)
        for (start_pos, end_pos) in sim_state.get_bond_for_display() {
            let start_x = start_pos.x as i16;
            let start_y = start_pos.y as i16;
            let end_x = end_pos.x as i16;
            let end_y = end_pos.y as i16;

            // Draw bonds as thin white lines
            self.canvas
                .line(start_x, start_y, end_x, end_y, Color::RGB(200, 200, 200))?;
        }

        // Draw molecules
        for mol in sim_state.grid.get_all_molecules() {
            let x = mol.pos.x as i16;
            let y = mol.pos.y as i16;
            let radius = mol.radius() as i16;

            // Choose color based on molecule type
            let color = match &mol.mol_type {
                MoleculeType::Gliadin => Color::RGB(200, 100, 100),      // Reddish
                MoleculeType::Glutenin { has_free_thiol } => {
                    if *has_free_thiol {
                        Color::RGB(100, 200, 100)  // Greenish (with free thiol)
                    } else {
                        Color::RGB(50, 150, 50)    // Darker green (bonded)
                    }
                },
                MoleculeType::Water => Color::RGB(100, 100, 200),       // Blueish
                MoleculeType::Yeast => Color::RGB(200, 200, 100),       // Yellowish
                MoleculeType::CO2 => Color::RGB(220, 220, 220),         // Light gray (bubble)
                MoleculeType::Ethanol => Color::RGB(150, 100, 200),     // Purple
                MoleculeType::Sugar => Color::RGB(255, 255, 255),       // White
                MoleculeType::Salt => Color::RGB(240, 240, 240),        // Near-white
                MoleculeType::Ash => Color::RGB(150, 150, 150),         // Gray
            };

            // Draw filled circle for the molecule
            self.canvas.filled_circle(x, y, radius, color)?;

            // Draw a border to make it more visible
            self.canvas.circle(x, y, radius, Color::RGB(50, 50, 50))?;
        }

        Ok(())
    }

    fn render_info(&mut self, sim_state: &SimulationState) -> Result<(), String> {
        use sdl2::ttf::{Font, Sdl2TtfContext};

        // Initialize font (in a real application, you'd cache this)
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

        // Load a system font directly from Windows
        let font = ttf_context.load_font("C:\\Windows\\Fonts\\arial.ttf", 16)
            .or_else(|_| ttf_context.load_font("C:\\Windows\\Fonts\\calibri.ttf", 16))
            .or_else(|_| ttf_context.load_font("C:\\Windows\\Fonts\\times.ttf", 16))
            .or_else(|_| {
                // If no fonts found, return an error
                Err("Could not load any system font")
            }).map_err(|e: &str| e.to_string())?;

        // Draw text for simulation information
        let text = format!("Time: {:.1}s | Molecules: {} | Bonds: {}",
            sim_state.time_elapsed,
            sim_state.grid.get_all_molecules().len(),
            sim_state.bonds.len());

        let surface = font.render(&text)
            .blended(Color::RGB(255, 255, 255))
            .map_err(|e| e.to_string())?;

        let texture_creator = self.canvas.texture_creator();
        let texture = texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        let target = sdl2::rect::Rect::new(10, 10, surface.width(), surface.height());
        self.canvas.copy(&texture, None, Some(target))?;

        // Draw instructions
        let instructions = "Controls: Click/Mouse - Mix | Space - Mix | S - Add Salt | Y - Add Yeast";
        let inst_surface = font.render(instructions)
            .blended(Color::RGB(200, 200, 200))
            .map_err(|e| e.to_string())?;

        let inst_texture = texture_creator.create_texture_from_surface(&inst_surface)
            .map_err(|e| e.to_string())?;

        let inst_target = sdl2::rect::Rect::new(10, 40, inst_surface.width(), inst_surface.height());
        self.canvas.copy(&inst_texture, None, Some(inst_target))?;

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn update_simulation_parameters(&mut self, sim_state: &mut SimulationState) {
        // Update recipe parameters from internal state
        sim_state.recipe_hydration = self.recipe_hydration;
        sim_state.recipe_salt = self.recipe_salt;
        sim_state.recipe_yeast = self.recipe_yeast;
        sim_state.temperature = self.temperature;
        sim_state.autolyse_time = self.autolyse_duration;

        // Handle ingredient additions
        if self.salt_added && !sim_state.salt_added {
            sim_state.add_salt();
            self.salt_added = false; // Reset the flag after processing
        }

        if self.yeast_added && !sim_state.yeast_added {
            sim_state.add_yeast();
            self.yeast_added = false; // Reset the flag after processing
        }

        // Apply force to region where mouse is clicked
        if self.apply_force_pressed {
            let mut rng = rand::thread_rng();
            let (mouse_x, mouse_y) = self.mouse_position;
            let center = Vector2::new(mouse_x as f32, mouse_y as f32);
            let radius = 50.0;
            let force = Vector2::new(
                (rng.gen::<f32>() - 0.5) * 20.0,
                (rng.gen::<f32>() - 0.5) * 20.0,
            ); // Random force direction

            sim_state.apply_force_to_region(center, radius, force);
            self.apply_force_pressed = false; // Reset the flag after processing
        }
    }
}