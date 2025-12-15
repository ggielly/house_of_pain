use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{GLProfile, Window, WindowContext};
use std::ffi::{CStr, CString};
use std::ptr;
use std::str;

use pain_core::{MoleculeType, SimulationState};
use nalgebra::{Vector3, Matrix4, Perspective3, Point3};

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;
const SIDE_PANEL_WIDTH: u32 = 280;
const SIM_WIDTH: u32 = SCREEN_WIDTH - SIDE_PANEL_WIDTH;

// Define a 3D camera for navigation
pub struct Camera {
    pub position: Vector3<f32>,
    pub target: Vector3<f32>,
    pub up: Vector3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub movement_speed: f32,
    pub mouse_sensitivity: f32,
    pub fov: f32,
}

impl Camera {
    pub fn new(position: Vector3<f32>, target: Vector3<f32>, up: Vector3<f32>) -> Self {
        let mut camera = Camera {
            position,
            target,
            up,
            yaw: -90.0,
            pitch: 0.0,
            movement_speed: 2.5,
            mouse_sensitivity: 0.1,
            fov: 45.0,
        };
        camera.update_camera_vectors();
        camera
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(
            &Point3::from(self.position),
            &Point3::from(self.target),
            &self.up,
        )
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        let aspect = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
        let perspective = Perspective3::new(aspect, self.fov.to_radians(), 0.1, 1000.0);
        perspective.as_matrix().clone()
    }

    pub fn process_keyboard(&mut self, direction: CameraMovement, delta_time: f32) {
        let velocity = self.movement_speed * delta_time;
        match direction {
            CameraMovement::Forward => self.position += self.front() * velocity,
            CameraMovement::Backward => self.position -= self.front() * velocity,
            CameraMovement::Left => self.position -= self.right() * velocity,
            CameraMovement::Right => self.position += self.right() * velocity,
            CameraMovement::Up => self.position += self.up * velocity,
            CameraMovement::Down => self.position -= self.up * velocity,
        }
        // Update target to maintain look direction
        self.target = self.position + self.front();
    }

    fn front(&self) -> Vector3<f32> {
        Vector3::new(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        ).normalize()
    }

    fn right(&self) -> Vector3<f32> {
        self.front().cross(&self.up).normalize()
    }

    fn up(&self) -> Vector3<f32> {
        self.up
    }

    pub fn process_mouse_movement(&mut self, xoffset: f32, yoffset: f32, constrain_pitch: bool) {
        let xoffset = xoffset * self.mouse_sensitivity;
        let yoffset = yoffset * self.mouse_sensitivity;

        self.yaw += xoffset;
        self.pitch += yoffset;

        if constrain_pitch {
            if self.pitch > 89.0 {
                self.pitch = 89.0;
            }
            if self.pitch < -89.0 {
                self.pitch = -89.0;
            }
        }

        self.update_camera_vectors();
    }

    fn update_camera_vectors(&mut self) {
        // Calculate the new front vector
        let front = Vector3::new(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        );
        self.target = self.position + front.normalize();
    }
}

#[derive(Copy, Clone)]
pub enum CameraMovement {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}

pub struct Renderer<'a> {
    window: Window,
    context: sdl2::video::GLContext,
    canvas: Canvas<Window>,
    font: sdl2::ttf::Font<'a, 'a>,
    camera: Camera,
    last_x: f32,
    last_y: f32,
    first_mouse: bool,
}

impl<'a> Renderer<'a> {
    pub fn new(
        sdl_context: &sdl2::Sdl,
        video_subsystem: &sdl2::VideoSubsystem,
        ttf_context: &'a Sdl2TtfContext,
    ) -> Result<Self, String> {
        // Set OpenGL attributes
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        gl_attr.set_double_buffer(true);
        gl_attr.set_depth_size(24);

        let window = video_subsystem
            .window(
                "House of Pain - 3D Sourdough Simulator",
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let context = window.gl_create_context().map_err(|e| e.to_string())?;
        window.gl_make_current(&context).map_err(|e| e.to_string())?;

        // Load OpenGL function pointers
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

        // Enable depth testing for 3D
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
        }

        // Load a font
        let font_path = "C:/Windows/Fonts/consola.ttf";
        let font = ttf_context.load_font(font_path, 16)?;

        Ok(Renderer {
            window,
            context,
            canvas: window.into_canvas().build().map_err(|e| e.to_string())?,
            font,
            camera: Camera::new(
                Vector3::new(500.0, 360.0, 500.0),  // Position
                Vector3::new(500.0, 360.0, 0.0),   // Look at center
                Vector3::new(0.0, 1.0, 0.0),       // Up vector
            ),
            last_x: SCREEN_WIDTH as f32 / 2.0,
            last_y: SCREEN_HEIGHT as f32 / 2.0,
            first_mouse: true,
        })
    }

    pub fn handle_events(&mut self, event_pump: &mut sdl2::EventPump) {
        for event in event_pump.poll_iter() {
            match event {
                Event::MouseMotion { x, y, .. } => {
                    if self.first_mouse {
                        self.last_x = x as f32;
                        self.last_y = y as f32;
                        self.first_mouse = false;
                    }

                    let xoffset = x as f32 - self.last_x;
                    let yoffset = self.last_y - y as f32; // Reversed since y-coordinates range from bottom to top
                    self.last_x = x as f32;
                    self.last_y = y as f32;

                    self.camera.process_mouse_movement(xoffset, yoffset, true);
                }
                _ => {}
            }
        }
    }

    pub fn update_camera(&mut self, sim_state: &SimulationState, dt: f32) {
        // Get keyboard state to update camera movement
        let keyboard_state = self.window.subsystem().sdl().keyboard_state();

        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::W) {
            self.camera.process_keyboard(CameraMovement::Forward, dt);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::S) {
            self.camera.process_keyboard(CameraMovement::Backward, dt);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::A) {
            self.camera.process_keyboard(CameraMovement::Left, dt);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::D) {
            self.camera.process_keyboard(CameraMovement::Right, dt);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Space) {
            self.camera.process_keyboard(CameraMovement::Up, dt);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::LShift) {
            self.camera.process_keyboard(CameraMovement::Down, dt);
        }
    }

    pub fn draw(&mut self, sim_state: &SimulationState) -> Result<(), String> {
        unsafe {
            // Clear the color and depth buffer
            gl::ClearColor(0.05, 0.05, 0.07, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Draw 3D simulation elements
        self.draw_molecules_3d(sim_state)?;
        self.draw_bonds_3d(sim_state)?;

        // Draw 2D UI overlay
        self.draw_ui_2d(sim_state)?;

        // Swap the buffers to present the frame
        self.window.gl_swap_window();

        Ok(())
    }

    fn draw_molecules_3d(&self, sim_state: &SimulationState) -> Result<(), String> {
        unsafe {
            // Set up OpenGL state for rendering points
            gl::Enable(gl::PROGRAM_POINT_SIZE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            // Use a simple point rendering approach for now
            // In a full implementation, we would use instanced rendering or proper sphere models
            gl::PointSize(5.0); // Base size for points

            // Draw each molecule as a colored point
            for molecule in sim_state.grid.get_all_molecules() {
                let color = self.get_molecule_color(&molecule.mol_type);

                // Set color based on molecule type
                gl::Color4f(
                    color.r as f32 / 255.0,
                    color.g as f32 / 255.0,
                    color.b as f32 / 255.0,
                    color.a as f32 / 255.0,
                );

                // Begin rendering points
                gl::Begin(gl::POINTS);
                gl::Vertex3f(molecule.pos.x, molecule.pos.y, molecule.pos.z);
                gl::End();
            }

            gl::Disable(gl::BLEND);
            gl::Disable(gl::PROGRAM_POINT_SIZE);
        }

        Ok(())
    }

    fn draw_bonds_3d(&self, sim_state: &SimulationState) -> Result<(), String> {
        unsafe {
            // Set line properties for bonds
            gl::LineWidth(1.0);
            gl::Color4f(0.8, 0.2, 0.4, 0.6); // Reddish color for bonds

            // Draw each bond as a line between two molecules
            for bond in &sim_state.bonds {
                if let (Some(mol_a), Some(mol_b)) = (
                    sim_state.grid.get_molecule(bond.molecule_a_id),
                    sim_state.grid.get_molecule(bond.molecule_b_id),
                ) {
                    gl::Begin(gl::LINES);
                    gl::Vertex3f(mol_a.pos.x, mol_a.pos.y, mol_a.pos.z);
                    gl::Vertex3f(mol_b.pos.x, mol_b.pos.y, mol_b.pos.z);
                    gl::End();
                }
            }
        }

        Ok(())
    }

    fn draw_ui_2d(&mut self, sim_state: &SimulationState) -> Result<(), String> {
        // Temporarily switch back to 2D rendering for UI elements
        let texture_creator = self.canvas.texture_creator();

        // Clear and set background for UI
        self.canvas.set_draw_color(Color::RGB(10, 10, 15));
        self.canvas.clear();

        // Draw side panel
        self.draw_side_panel(sim_state)?;

        self.canvas.present();

        Ok(())
    }

    fn draw_side_panel(&mut self, sim_state: &SimulationState) -> Result<(), String> {
        let panel_x = SIM_WIDTH as i32;
        let panel_rect = Rect::new(panel_x, 0, SIDE_PANEL_WIDTH, SCREEN_HEIGHT);
        self.canvas.set_draw_color(Color::RGB(20, 20, 30));
        self.canvas.fill_rect(panel_rect)?;

        // Draw dividing line
        self.canvas.set_draw_color(Color::RGB(50, 50, 60));
        self.canvas.draw_line(
            Point::new(panel_x, 0),
            Point::new(panel_x, SCREEN_HEIGHT as i32),
        )?;

        // --- Display Text ---
        let texture_creator = self.canvas.texture_creator();
        let mut y = 20;
        let line_height = 25;

        // Helper closure to render a line of text
        let mut render_line = |text: &str, x_offset: i32, current_y: i32| -> Result<(), String> {
            if text.is_empty() {
                return Ok(()); // Skip empty lines
            }
            let surface = self
                .font
                .render(text)
                .blended(Color::RGB(220, 220, 220))
                .map_err(|e| e.to_string())?;
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;
            let query = texture.query();
            self.canvas.copy(
                &texture,
                None,
                Rect::new(x_offset, current_y, query.width, query.height),
            )?;
            Ok(())
        };

        render_line("House of Pain", panel_x + 20, y)?;
        render_line("3D Mode", panel_x + 20, y + 20)?;
        y += line_height * 2;

        // Stats
        let stats = collect_stats(sim_state);
        for (key, value) in stats {
            let text = if key.is_empty() {
                "".to_string()
            } else {
                format!("{}: {}", key, value)
            };
            render_line(&text, panel_x + 20, y)?;
            y += line_height;
        }

        // Controls
        y += line_height;
        let controls_text = [
            "3D Controls:",
            "  WASD - Move",
            "  Space - Up",
            "  Shift - Down",
            "  Mouse - Look",
            "  S - Add Salt",
            "  Y - Add Yeast",
            "  C - Fold",
            "  R - Reset",
        ];
        for text in &controls_text {
            render_line(text, panel_x + 20, y)?;
            y += line_height;
        }

        Ok(())
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    pub fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}

    fn get_molecule_color(&self, mol_type: &MoleculeType) -> Color {
        match mol_type {
            MoleculeType::Gliadin => Color::RGB(255, 180, 100), // Light orange
            MoleculeType::Glutenin { has_free_thiol } => {
                if *has_free_thiol {
                    Color::RGB(150, 100, 255) // Purple for reactive
                } else {
                    Color::RGB(100, 60, 200) // Darker purple for bonded
                }
            }
            MoleculeType::Water => Color::RGBA(50, 100, 200, 150), // Translucent blue
            MoleculeType::Yeast => Color::RGB(255, 255, 100),      // Bright yellow
            MoleculeType::CO2 => Color::RGBA(200, 220, 200, 180),  // Light, almost white bubbles
            MoleculeType::Ethanol => Color::RGB(200, 100, 200),    // Pinkish-purple
            MoleculeType::Sugar => Color::RGB(255, 255, 255),      // White
            MoleculeType::Salt => Color::RGB(100, 200, 180),       // Teal/aqua
            MoleculeType::Ash => Color::RGB(120, 120, 120),        // Grey
        }
    }
}

fn collect_stats(sim_state: &SimulationState) -> Vec<(String, String)> {
    let total_molecules = sim_state.grid.get_all_molecules().len();
    let yeast_count = sim_state.get_molecules_by_type(&MoleculeType::Yeast).len();
    let sugar_count = sim_state.get_molecules_by_type(&MoleculeType::Sugar).len();
    let co2_count = sim_state.get_molecules_by_type(&MoleculeType::CO2).len();
    let ethanol_count = sim_state
        .get_molecules_by_type(&MoleculeType::Ethanol)
        .len();

    vec![
        (
            "Time".to_string(),
            format!("{:.1}s", sim_state.time_elapsed),
        ),
        (
            "Temperature".to_string(),
            format!("{:.1}°C", sim_state.temperature),
        ),
        ("Molecules".to_string(), format!("{}", total_molecules)),
        (
            "Gluten Bonds".to_string(),
            format!("{}", sim_state.bonds.len()),
        ),
        ("".to_string(), "".to_string()), // Spacer
        ("Yeast".to_string(), format!("{}", yeast_count)),
        ("Sugar".to_string(), format!("{}", sugar_count)),
        ("CO2".to_string(), format!("{}", co2_count)),
        ("Ethanol".to_string(), format!("{}", ethanol_count)),
    ]
}
