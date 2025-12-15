use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};

use pain_core::{MoleculeType, SimulationState};

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;
const SIDE_PANEL_WIDTH: u32 = 280;
const SIM_WIDTH: u32 = SCREEN_WIDTH - SIDE_PANEL_WIDTH;

pub struct Renderer<'a> {
    canvas: Canvas<Window>,
    font: sdl2::ttf::Font<'a, 'a>,
}

impl<'a> Renderer<'a> {
    pub fn new(
        _sdl_context: &sdl2::Sdl,
        video_subsystem: &sdl2::VideoSubsystem,
        ttf_context: &'a Sdl2TtfContext,
    ) -> Result<Self, String> {
        let window = video_subsystem
            .window(
                "House of Pain - Sourdough Simulator",
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
            )
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        // Load a font - ensure the path is correct for your system
        // Using a more generic path might be better, but for now this is explicit.
        let font_path = "C:/Windows/Fonts/consola.ttf";
        let font = ttf_context.load_font(font_path, 16)?;

        Ok(Renderer { canvas, font })
    }

    pub fn draw(&mut self, sim_state: &SimulationState) -> Result<(), String> {
        self.canvas.set_draw_color(Color::RGB(10, 10, 15));
        self.canvas.clear();

        // Draw simulation area
        self.draw_molecules(sim_state)?;
        self.draw_bonds(sim_state)?;

        // Draw side panel - this is where the borrow checker issues were
        self.draw_side_panel(sim_state)?;

        self.canvas.present();
        Ok(())
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

    fn draw_molecules(&mut self, sim_state: &SimulationState) -> Result<(), String> {
        for molecule in sim_state.grid.get_all_molecules() {
            let color = self.get_molecule_color(&molecule.mol_type);

            let radius = molecule.radius();
            // Use filled_circle from gfx for smooth circles
            self.canvas.filled_circle(
                molecule.pos.x as i16,
                molecule.pos.y as i16,
                radius as i16,
                color,
            )?;
        }
        Ok(())
    }

    fn draw_bonds(&mut self, sim_state: &SimulationState) -> Result<(), String> {
        self.canvas.set_draw_color(Color::RGB(200, 60, 100)); // Reddish color for bonds
        let bonds = sim_state.get_bond_for_display();
        for (start, end) in bonds {
            self.canvas.draw_line(
                Point::new(start.x as i32, start.y as i32),
                Point::new(end.x as i32, end.y as i32),
            )?;
        }
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
            "Controls:",
            "  S - Add Salt",
            "  Y - Add Yeast",
            "  C - Fold (Apply Force)",
            "  R - Reset Simulation",
        ];
        for text in &controls_text {
            render_line(text, panel_x + 20, y)?;
            y += line_height;
        }

        Ok(())
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
