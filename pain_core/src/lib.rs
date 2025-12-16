use nalgebra::Vector3;
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum MoleculeType {
    Gliadin,
    Glutenin { has_free_thiol: bool },
    Water,
    Yeast,
    CO2,
    Ethanol,
    Sugar,
    Salt,
    Ash,
}

#[derive(Debug, Clone)]
pub struct Molecule {
    pub id: u64,
    pub pos: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub mol_type: MoleculeType,
}

#[derive(Debug)]
pub struct Bond {
    pub molecule_a_id: u64,
    pub molecule_b_id: u64,
    pub target_distance: f32,
}

#[derive(Debug)]
pub struct SpatialGrid3D {
    cell_size: f32,
    grid: HashMap<(i32, i32, i32), Vec<u64>>,
    molecules: HashMap<u64, Molecule>,
    next_id: u64,
}

pub struct SimulationState {
    pub grid: SpatialGrid3D,
    pub bonds: Vec<Bond>,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub temperature: f32,      // Influences reaction rates
    pub time_elapsed: f32,     // Time elapsed in seconds
    pub recipe_hydration: f32, // Hydration percentage (0.65 to 0.90)
    pub recipe_salt: f32,      // Salt percentage (0.0 to 0.03)
    pub recipe_yeast: f32,     // Yeast/levain percentage (0.10 to 0.30)
    pub autolyse_time: f32,    // Duration of autolyse phase in seconds
    pub salt_added: bool,      // Track if salt has been added
    pub yeast_added: bool,     // Track if yeast has been added
}

impl Molecule {
    pub fn new(mol_type: MoleculeType, pos: Vector3<f32>, velocity: Vector3<f32>) -> Self {
        Molecule {
            id: 0, // Will be assigned by SpatialGrid3D
            pos,
            velocity,
            mol_type,
        }
    }

    pub fn radius(&self) -> f32 {
        match self.mol_type {
            MoleculeType::Gliadin => 3.0,
            MoleculeType::Glutenin { .. } => 4.0,
            MoleculeType::Water => 1.5,
            MoleculeType::Yeast => 5.0,
            MoleculeType::CO2 => 8.0,
            MoleculeType::Ethanol => 2.0,
            MoleculeType::Sugar => 2.5,
            MoleculeType::Salt => 1.8,
            MoleculeType::Ash => 2.0,
        }
    }

    pub fn mass(&self) -> f32 {
        match self.mol_type {
            MoleculeType::Gliadin => 10.0,
            MoleculeType::Glutenin { .. } => 12.0,
            MoleculeType::Water => 1.0,
            MoleculeType::Yeast => 15.0,
            MoleculeType::CO2 => 2.0,
            MoleculeType::Ethanol => 3.0,
            MoleculeType::Sugar => 4.0,
            MoleculeType::Salt => 2.0,
            MoleculeType::Ash => 2.0,
        }
    }
}

impl SpatialGrid3D {
    pub fn new(_width: f32, _height: f32, _depth: f32, cell_size: f32) -> Self {
        SpatialGrid3D {
            cell_size,
            grid: HashMap::new(),
            molecules: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn insert(&mut self, mut molecule: Molecule) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        molecule.id = id;

        self.molecules.insert(id, molecule.clone());

        let cell_coords = self.get_cell_coords(molecule.pos);
        self.grid
            .entry(cell_coords)
            .or_insert_with(Vec::new)
            .push(id);

        id
    }

    pub fn get_cell_coords(&self, pos: Vector3<f32>) -> (i32, i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
            (pos.z / self.cell_size).floor() as i32,
        )
    }

    pub fn get_neighbors(&self, pos: Vector3<f32>) -> Vec<&Molecule> {
        let center_cell = self.get_cell_coords(pos);
        let mut neighbors = Vec::new();

        // Check the center cell and all 26 surrounding cells (3x3x3 cube)
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let cell_coords = (center_cell.0 + dx, center_cell.1 + dy, center_cell.2 + dz);

                    if let Some(ids) = self.grid.get(&cell_coords) {
                        for &id in ids {
                            if let Some(mol) = self.molecules.get(&id) {
                                neighbors.push(mol);
                            }
                        }
                    }
                }
            }
        }

        neighbors
    }

    pub fn remove(&mut self, id: u64) {
        if let Some(molecule) = self.molecules.remove(&id) {
            let cell_coords = self.get_cell_coords(molecule.pos);

            if let Some(ids) = self.grid.get_mut(&cell_coords) {
                ids.retain(|&mol_id| mol_id != id);
            }
        }
    }

    pub fn update_molecule_pos(&mut self, id: u64, new_pos: Vector3<f32>) {
        if let Some(mol) = self.molecules.get_mut(&id) {
            let old_pos = mol.pos;
            // Update position
            mol.pos = new_pos;

            // Remove from old cell
            let old_cell_coords = self.get_cell_coords(old_pos);
            if let Some(ids) = self.grid.get_mut(&old_cell_coords) {
                ids.retain(|&mol_id| mol_id != id);
            }

            // Insert into new cell
            let new_cell_coords = self.get_cell_coords(new_pos);
            self.grid
                .entry(new_cell_coords)
                .or_insert_with(Vec::new)
                .push(id);
        }
    }

    pub fn get_molecule(&self, id: u64) -> Option<&Molecule> {
        self.molecules.get(&id)
    }

    pub fn get_molecule_mut(&mut self, id: u64) -> Option<&mut Molecule> {
        self.molecules.get_mut(&id)
    }

    pub fn get_all_molecules(&self) -> Vec<&Molecule> {
        self.molecules.values().collect()
    }

    pub fn get_all_molecules_mut(&mut self) -> Vec<&mut Molecule> {
        self.molecules.values_mut().collect()
    }
}

impl SimulationState {
    pub fn new(width: f32, height: f32, depth: f32) -> Self {
        SimulationState {
            grid: SpatialGrid3D::new(width, height, depth, 15.0),
            bonds: Vec::new(),
            width,
            height,
            depth,
            temperature: 25.0, // Default temperature in Celsius
            time_elapsed: 0.0,
            recipe_hydration: 0.72, // 72% hydration
            recipe_salt: 0.02,      // 2% salt
            recipe_yeast: 0.20,     // 20% yeast/levain
            autolyse_time: 1800.0,  // 30 minutes of autolyse (in seconds)
            salt_added: true,       // Initially true for new simulation, but will be managed by UI
            yeast_added: false,     // Initially false until user adds yeast
        }
    }

    pub fn initialize_classic_recipe(&mut self) {
        self.recipe_hydration = 0.72;
        self.recipe_salt = 0.02;
        self.recipe_yeast = 0.20;
        self.autolyse_time = 1800.0; // 30 minutes
        self.temperature = 25.0;

        // Reset simulation state
        self.grid = SpatialGrid3D::new(self.width, self.height, self.depth, 15.0);
        self.bonds.clear();
        self.time_elapsed = 0.0;
        self.salt_added = false; // We'll add salt later
        self.yeast_added = false;

        // Add initial flour components: gliadin and glutenin proteins
        let flour_proteins = 200; // Limite stricte pour la démo
        for _ in 0..flour_proteins {
            let x = rand::thread_rng().gen_range(0.0..self.width);
            let y = rand::thread_rng().gen_range(0.0..self.height);
            let z = rand::thread_rng().gen_range(0.0..self.depth);
            let pos = Vector3::new(x, y, z);

            // Randomly distribute gliadins and glutens
            let vel_x = rand::thread_rng().gen_range(-0.1..0.1);
            let vel_y = rand::thread_rng().gen_range(-0.1..0.1);
            let vel_z = rand::thread_rng().gen_range(-0.1..0.1);
            let velocity = Vector3::new(vel_x, vel_y, vel_z);

            let protein_choice = rand::thread_rng().gen_range(0..100);
            if protein_choice < 40 {
                // 40% gliadin
                let molecule = Molecule::new(MoleculeType::Gliadin, pos, velocity);
                self.grid.insert(molecule);
            } else {
                // 60% glutenin
                let molecule = Molecule::new(
                    MoleculeType::Glutenin {
                        has_free_thiol: true,
                    },
                    pos,
                    velocity,
                );
                self.grid.insert(molecule);
            }
        }

        // Add water based on hydration percentage
        let water_amount = 200; // Limite stricte pour la démo
        for _ in 0..water_amount as usize {
            let x = rand::thread_rng().gen_range(0.0..self.width);
            let y = rand::thread_rng().gen_range(0.0..self.height);
            let z = rand::thread_rng().gen_range(0.0..self.depth);
            let pos = Vector3::new(x, y, z);

            let vel_x = rand::thread_rng().gen_range(-0.2..0.2);
            let vel_y = rand::thread_rng().gen_range(-0.2..0.2);
            let vel_z = rand::thread_rng().gen_range(-0.2..0.2);
            let velocity = Vector3::new(vel_x, vel_y, vel_z);

            let molecule = Molecule::new(MoleculeType::Water, pos, velocity);
            self.grid.insert(molecule);
        }
    }

    pub fn add_salt(&mut self) {
        if !self.salt_added {
            let salt_amount = (self.width * self.height * self.depth * 0.00005 * self.recipe_salt) as usize;

            for _ in 0..salt_amount {
                let x = rand::thread_rng().gen_range(0.0..self.width);
                let y = rand::thread_rng().gen_range(0.0..self.height);
                let z = rand::thread_rng().gen_range(0.0..self.depth);
                let pos = Vector3::new(x, y, z);

                let vel_x = rand::thread_rng().gen_range(-0.2..0.2);
                let vel_y = rand::thread_rng().gen_range(-0.2..0.2);
                let vel_z = rand::thread_rng().gen_range(-0.2..0.2);
                let velocity = Vector3::new(vel_x, vel_y, vel_z);

                let molecule = Molecule::new(MoleculeType::Salt, pos, velocity);
                self.grid.insert(molecule);
            }

            self.salt_added = true;
        }
    }

    pub fn add_yeast(&mut self) {
        if !self.yeast_added {
            let yeast_amount = (self.width * self.height * self.depth * 0.00002 * self.recipe_yeast) as usize;

            for _ in 0..yeast_amount {
                let x = rand::thread_rng().gen_range(0.0..self.width);
                let y = rand::thread_rng().gen_range(0.0..self.height);
                let z = rand::thread_rng().gen_range(0.0..self.depth);
                let pos = Vector3::new(x, y, z);

                let vel_x = rand::thread_rng().gen_range(-0.1..0.1);
                let vel_y = rand::thread_rng().gen_range(-0.1..0.1);
                let vel_z = rand::thread_rng().gen_range(-0.1..0.1);
                let velocity = Vector3::new(vel_x, vel_y, vel_z);

                let molecule = Molecule::new(MoleculeType::Yeast, pos, velocity);
                self.grid.insert(molecule);

                // Also add sugar for the yeast to consume
                let sugar_x =
                    rand::thread_rng().gen_range((x - 20.0).max(0.0)..(x + 20.0).min(self.width));
                let sugar_y =
                    rand::thread_rng().gen_range((y - 20.0).max(0.0)..(y + 20.0).min(self.height));
                let sugar_z =
                    rand::thread_rng().gen_range((z - 20.0).max(0.0)..(z + 20.0).min(self.depth));
                let sugar_pos = Vector3::new(sugar_x, sugar_y, sugar_z);

                let sugar_vel_x = rand::thread_rng().gen_range(-0.1..0.1);
                let sugar_vel_y = rand::thread_rng().gen_range(-0.1..0.1);
                let sugar_vel_z = rand::thread_rng().gen_range(-0.1..0.1);
                let sugar_velocity = Vector3::new(sugar_vel_x, sugar_vel_y, sugar_vel_z);

                let sugar_molecule = Molecule::new(MoleculeType::Sugar, sugar_pos, sugar_velocity);
                self.grid.insert(sugar_molecule);
            }

            self.yeast_added = true;
        }
    }

    pub fn apply_force_to_region(
        &mut self,
        center: Vector3<f32>,
        radius: f32,
        force: Vector3<f32>,
    ) {
        let mut mol_ids_to_update = Vec::new();
        let neighbors = self.grid.get_neighbors(center);

        for mol in neighbors {
            if (mol.pos - center).magnitude() < radius {
                mol_ids_to_update.push(mol.id);
            }
        }

        for id in mol_ids_to_update {
            if let Some(mol_mut) = self.grid.get_molecule_mut(id) {
                mol_mut.velocity += force / mol_mut.mass();

                // Limit max velocity to prevent particles from flying away too fast
                let max_vel = 5.0;
                let vel_mag = mol_mut.velocity.magnitude();
                if vel_mag > max_vel {
                    mol_mut.velocity = mol_mut.velocity.normalize() * max_vel;
                }
            }
        }
    }

    pub fn tick(&mut self, dt: f32) {
        // Update time elapsed
        self.time_elapsed += dt;

        // Update molecule positions and apply physics
        let mut molecules_to_update = Vec::new();
        for mol in self.grid.get_all_molecules_mut() {
            // Apply velocity
            mol.pos += mol.velocity * dt;

            // Boundary conditions (bounce off walls)
            if mol.pos.x < mol.radius() {
                mol.pos.x = mol.radius();
                mol.velocity.x = -mol.velocity.x * 0.8; // Dampening
            }
            if mol.pos.x > self.width - mol.radius() {
                mol.pos.x = self.width - mol.radius();
                mol.velocity.x = -mol.velocity.x * 0.8;
            }
            if mol.pos.y < mol.radius() {
                mol.pos.y = mol.radius();
                mol.velocity.y = -mol.velocity.y * 0.8;
            }
            if mol.pos.y > self.height - mol.radius() {
                mol.pos.y = self.height - mol.radius();
                mol.velocity.y = -mol.velocity.y * 0.8;
            }
            if mol.pos.z < mol.radius() {
                mol.pos.z = mol.radius();
                mol.velocity.z = -mol.velocity.z * 0.8; // Dampening
            }
            if mol.pos.z > self.depth - mol.radius() {
                mol.pos.z = self.depth - mol.radius();
                mol.velocity.z = -mol.velocity.z * 0.8;
            }

            // Apply some friction to slow down movement gradually
            mol.velocity *= 0.999;

            // Store for updating spatial grid
            molecules_to_update.push((mol.id, mol.pos));
        }

        // Update spatial grid with new positions
        for (id, pos) in molecules_to_update {
            self.grid.update_molecule_pos(id, pos);
        }

        // Handle chemical reactions and yeast activity
        self.handle_chemistry(dt);

        // Apply bond constraints
        self.apply_bond_constraints();
    }

    fn handle_chemistry(&mut self, dt: f32) {
        // Formation of disulfide bridges between glutenins
        self.form_disulfide_bridges();

        // Yeast activity (consuming sugar and producing CO2 and ethanol)
        if self.yeast_added {
            self.handle_yeast_activity(dt);
        }
    }

    fn form_disulfide_bridges(&mut self) {
        let mut new_bonds = Vec::new();
        let mut mol_ids_to_update = Vec::new();

        for mol in self.grid.get_all_molecules() {
            if let MoleculeType::Glutenin {
                has_free_thiol: true,
            } = mol.mol_type
            {
                let neighbors = self.grid.get_neighbors(mol.pos);

                for neighbor in neighbors {
                    if neighbor.id == mol.id {
                        continue; // Skip self
                    }

                    if let MoleculeType::Glutenin {
                        has_free_thiol: true,
                    } = neighbor.mol_type
                    {
                        let dist = (mol.pos - neighbor.pos).magnitude();

                        // Check if they're close enough to react
                        if dist < 8.0 {
                            // Reaction distance threshold
                            // Probability of reaction based on temperature and presence of salt
                            let mut reaction_prob = 0.20; // Augmented base probability (was 0.05)

                            // Increase probability with temperature
                            reaction_prob *= (self.temperature / 25.0).max(0.1); // Normalize to 25°C base

                            // Check if there's salt nearby to catalyze the reaction
                            let salt_neighbors = self.grid.get_neighbors(mol.pos);
                            for salt_neighbor in salt_neighbors {
                                if matches!(salt_neighbor.mol_type, MoleculeType::Salt) {
                                    reaction_prob *= 1.2; // Salt increases reaction rate
                                    break;
                                }
                            }

                            if rand::thread_rng().gen::<f32>() < reaction_prob * 0.1 {
                                // Scale down frequency
                                // Create a bond between the two molecules
                                new_bonds.push(Bond {
                                    molecule_a_id: mol.id,
                                    molecule_b_id: neighbor.id,
                                    target_distance: dist,
                                });

                                // Schedule molecules to update their thiol state
                                mol_ids_to_update.push(mol.id);
                                mol_ids_to_update.push(neighbor.id);
                            }
                        }
                    }
                }
            }
        }

        // Add new bonds to our bonds list
        for bond in new_bonds {
            // Check if this bond already exists to avoid duplicates
            if !self.bonds.iter().any(|b| {
                (b.molecule_a_id == bond.molecule_a_id && b.molecule_b_id == bond.molecule_b_id)
                    || (b.molecule_a_id == bond.molecule_b_id
                        && b.molecule_b_id == bond.molecule_a_id)
            }) {
                self.bonds.push(bond);
            }
        }

        // Update thiol states after bond creation
        for id in mol_ids_to_update {
            if let Some(mol_mut) = self.grid.get_molecule_mut(id) {
                if let MoleculeType::Glutenin {
                    ref mut has_free_thiol,
                } = mol_mut.mol_type
                {
                    *has_free_thiol = false;
                }
            }
        }
    }

    fn handle_yeast_activity(&mut self, dt: f32) {
        // Process yeast metabolism
        let mut consumed_sugars = Vec::new();
        let mut new_molecules = Vec::new();

        for mol in self.grid.get_all_molecules() {
            if let MoleculeType::Yeast = mol.mol_type {
                // Look for nearby sugar to consume
                let neighbors = self.grid.get_neighbors(mol.pos);

                for neighbor in neighbors {
                    if neighbor.id == mol.id {
                        continue; // Skip self
                    }

                    if matches!(neighbor.mol_type, MoleculeType::Sugar) {
                        // Calculate distance
                        let dist = (mol.pos - neighbor.pos).magnitude();
                        if dist < 5.0 {
                            // Within reaction distance
                            // Consume the sugar
                            consumed_sugars.push(neighbor.id);

                            // Increase yeast metabolism rate based on temperature
                            let metabolism_rate = (self.temperature / 20.0).max(0.1); // Normalized to 20°C base

                            // Random chance to produce CO2 based on metabolism rate
                            if rand::thread_rng().gen::<f32>() < 0.01 * metabolism_rate * dt {
                                // Produce CO2 bubble
                                let co2_pos = Vector3::new(
                                    mol.pos.x + rand::thread_rng().gen_range(-3.0..3.0),
                                    mol.pos.y + rand::thread_rng().gen_range(-3.0..3.0),
                                    mol.pos.z + rand::thread_rng().gen_range(-3.0..3.0),
                                );

                                let co2_vel = Vector3::new(
                                    rand::thread_rng().gen_range(-0.2..0.2),
                                    rand::thread_rng().gen_range(-0.2..0.2),
                                    rand::thread_rng().gen_range(-0.2..0.2),
                                );

                                let co2_molecule =
                                    Molecule::new(MoleculeType::CO2, co2_pos, co2_vel);
                                new_molecules.push(co2_molecule);

                                // Occasionally produce ethanol too
                                if rand::thread_rng().gen::<f32>() < 0.3 {
                                    let ethanol_pos = Vector3::new(
                                        mol.pos.x + rand::thread_rng().gen_range(-2.0..2.0),
                                        mol.pos.y + rand::thread_rng().gen_range(-2.0..2.0),
                                        mol.pos.z + rand::thread_rng().gen_range(-2.0..2.0),
                                    );

                                    let ethanol_vel = Vector3::new(
                                        rand::thread_rng().gen_range(-0.1..0.1),
                                        rand::thread_rng().gen_range(-0.1..0.1),
                                        rand::thread_rng().gen_range(-0.1..0.1),
                                    );

                                    let ethanol_molecule = Molecule::new(
                                        MoleculeType::Ethanol,
                                        ethanol_pos,
                                        ethanol_vel,
                                    );
                                    new_molecules.push(ethanol_molecule);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Add new molecules to the simulation
        for mol in new_molecules {
            self.grid.insert(mol);
        }

        // Remove consumed sugars after we're done checking neighbors
        for sugar_id in consumed_sugars {
            self.grid.remove(sugar_id);
        }

        // Handle CO2 bubble behavior - they tend to rise
        for mol in self.grid.get_all_molecules_mut() {
            if let MoleculeType::CO2 = mol.mol_type {
                // CO2 bubbles rise due to their lower density
                mol.velocity.y -= 0.05; // Apply upward force

                // Apply some random motion for realism
                mol.velocity.x += rand::thread_rng().gen_range(-0.02..0.02);
            }
        }
    }

    fn apply_bond_constraints(&mut self) {
        let mut forces: HashMap<u64, Vector3<f32>> = HashMap::new();

        for bond in &self.bonds {
            if let (Some(mol_a), Some(mol_b)) = (
                self.grid.get_molecule(bond.molecule_a_id),
                self.grid.get_molecule(bond.molecule_b_id),
            ) {
                let diff = mol_b.pos - mol_a.pos;
                let current_dist = diff.magnitude();

                if current_dist > 0.0 {
                    let correction = (bond.target_distance - current_dist) / current_dist * 0.5;
                    let correction_vec = diff * correction;

                    // Apply correction forces (but store them to apply later to avoid borrow checker issues)
                    forces
                        .entry(mol_a.id)
                        .and_modify(|f| *f += correction_vec)
                        .or_insert(correction_vec);

                    forces
                        .entry(mol_b.id)
                        .and_modify(|f| *f -= correction_vec)
                        .or_insert(-correction_vec);
                }
            }
        }

        // Apply accumulated forces to molecules
        for (mol_id, force) in forces {
            if let Some(mol) = self.grid.get_molecule_mut(mol_id) {
                mol.velocity += force / mol.mass();

                // Limit max velocity to prevent instability
                let max_vel = 3.0;
                let vel_mag = mol.velocity.magnitude();
                if vel_mag > max_vel {
                    mol.velocity = mol.velocity.normalize() * max_vel;
                }
            }
        }
    }

    pub fn get_bond_for_display(&self) -> Vec<(Vector3<f32>, Vector3<f32>)> {
        let mut bond_lines = Vec::new();

        for bond in &self.bonds {
            if let (Some(mol_a), Some(mol_b)) = (
                self.grid.get_molecule(bond.molecule_a_id),
                self.grid.get_molecule(bond.molecule_b_id),
            ) {
                bond_lines.push((mol_a.pos, mol_b.pos));
            }
        }

        bond_lines
    }

    pub fn get_molecules_by_type(&self, mol_type: &MoleculeType) -> Vec<&Molecule> {
        self.grid.get_all_molecules()
            .into_iter()
            .filter(|mol| matches!(&mol.mol_type, t if std::mem::discriminant(t) == std::mem::discriminant(mol_type)))
            .collect()
    }
}
