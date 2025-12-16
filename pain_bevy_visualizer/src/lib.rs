use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use avian3d::prelude::*;
use pain_core::{MoleculeType, SimulationState};
use bevy::ecs::world::FromWorld;

// Component pour représenter une particule de la simulation
#[derive(Component)]
pub struct MoleculeParticle {
    pub id: u64,
    pub mol_type: MoleculeType,
}

// Component pour représenter une liaison entre molécules
#[derive(Component)]
pub struct GlutenBond {
    pub molecule_a_id: u64,
    pub molecule_b_id: u64,
}

// Resource pour contenir l'état de la simulation
#[derive(Resource)]
pub struct SimulationResource {
    pub state: SimulationState,
}

impl FromWorld for SimulationResource {
    fn from_world(_world: &mut World) -> Self {
        let mut sim_state = SimulationState::new(1000.0, 720.0, 1000.0);
        sim_state.initialize_classic_recipe();
        SimulationResource { state: sim_state }
    }
}

// Plugin principal pour le système de particules et physique
pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(PhysicsPlugins::default())
            .init_resource::<SimulationResource>()
            .add_systems(Startup, setup)
            .add_systems(Update, (
                update_particles,
                update_bonds,
                handle_user_input
            ))
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_plugins(LogDiagnosticsPlugin::default());
    }
}

// Fonction d'initialisation de la scène
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Lumière ambiante plus forte
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.2,
    });

    // Lumière directionnelle plus puissante
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 50000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            -135.0_f32.to_radians(),
            -45.0_f32.to_radians(),
            0.0,
        )),
        ..default()
    });

    // Caméra
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(1500.0, 1200.0, 1500.0)
                .looking_at(Vec3::new(500.0, 360.0, 500.0), Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                far: 5000.0,
                ..default()
            }),
            ..default()
        },
    ));

    // Cadre de simulation
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1000.0, 720.0, 1000.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgba(0.7, 0.7, 0.7, 0.08),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        transform: Transform::from_xyz(500.0, 360.0, 500.0),
        ..default()
    });
}

// Système pour mettre à jour les particules à partir de l'état de la simulation
fn update_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut particle_query: Query<(Entity, &mut Transform, &mut MoleculeParticle)>,
    sim_resource: Res<SimulationResource>,
) {
    let sim_state = &sim_resource.state;

    let materials_map = create_materials_if_needed(&mut materials);

    // Map id -> entity pour update rapide
    let mut entity_map = std::collections::HashMap::new();
    for (entity, _, particle) in particle_query.iter_mut() {
        entity_map.insert(particle.id, entity);
    }

    // Synchronise ou crée les entités
    for molecule in sim_state.grid.get_all_molecules() {
        let pos = Vec3::new(
            molecule.pos.x as f32,
            molecule.pos.y as f32,
            molecule.pos.z as f32,
        );
        if let Some(entity) = entity_map.get(&molecule.id) {
            if let Ok((_, mut transform, _)) = particle_query.get_mut(*entity) {
                transform.translation = pos;
            }
        } else {
            let material_handle = match &molecule.mol_type {
                MoleculeType::Gliadin => materials_map.gliadin.clone(),
                MoleculeType::Glutenin { has_free_thiol: true } => materials_map.reactive_glutenin.clone(),
                MoleculeType::Glutenin { has_free_thiol: false } => materials_map.bonded_glutenin.clone(),
                MoleculeType::Water => materials_map.water.clone(),
                MoleculeType::Yeast => materials_map.yeast.clone(),
                MoleculeType::CO2 => materials_map.co2.clone(),
                MoleculeType::Ethanol => materials_map.ethanol.clone(),
                MoleculeType::Sugar => materials_map.sugar.clone(),
                MoleculeType::Salt => materials_map.salt.clone(),
                MoleculeType::Ash => materials_map.ash.clone(),
            };
            let radius = 3.0;
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Sphere::new(radius)),
                    material: material_handle,
                    transform: Transform::from_translation(pos).with_scale(Vec3::ONE),
                    ..default()
                },
                MoleculeParticle {
                    id: molecule.id,
                    mol_type: molecule.mol_type.clone(),
                },
            ));
        }
    }
    // Supprime les entités orphelines
    let valid_ids: std::collections::HashSet<u64> = sim_state.grid.get_all_molecules().iter().map(|m| m.id).collect();
    for (entity, _, particle) in particle_query.iter() {
        if !valid_ids.contains(&particle.id) {
            commands.entity(entity).despawn();
        }
    }
}

// Structure pour stocker les handles des matériaux
struct MaterialHandles {
    gliadin: Handle<StandardMaterial>,
    reactive_glutenin: Handle<StandardMaterial>,
    bonded_glutenin: Handle<StandardMaterial>,
    water: Handle<StandardMaterial>,
    yeast: Handle<StandardMaterial>,
    co2: Handle<StandardMaterial>,
    ethanol: Handle<StandardMaterial>,
    sugar: Handle<StandardMaterial>,
    salt: Handle<StandardMaterial>,
    ash: Handle<StandardMaterial>,
}

// Fonction utilitaire pour créer les matériaux si nécessaire
fn create_materials_if_needed(materials: &mut ResMut<Assets<StandardMaterial>>) -> MaterialHandles {
    let gliadin = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.27, 0.0), // orange-rouge
        perceptual_roughness: 0.5,
        reflectance: 0.2,
        ..default()
    });
    let reactive_glutenin = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.0), // jaune
        perceptual_roughness: 0.5,
        reflectance: 0.2,
        ..default()
    });
    let bonded_glutenin = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0), // vert
        perceptual_roughness: 0.5,
        reflectance: 0.2,
        ..default()
    });
    let water = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.4, 1.0), // bleu vif
        perceptual_roughness: 0.2,
        reflectance: 0.2,
        ..default()
    });
    let yeast = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0), // blanc
        perceptual_roughness: 0.5,
        reflectance: 0.2,
        ..default()
    });
    let co2 = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 1.0), // cyan
        perceptual_roughness: 0.5,
        reflectance: 0.2,
        ..default()
    });
    let ethanol = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.0, 0.8), // violet
        perceptual_roughness: 0.5,
        reflectance: 0.2,
        ..default()
    });
    let sugar = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.6), // rose
        perceptual_roughness: 0.5,
        reflectance: 0.2,
        ..default()
    });
    let salt = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5), // gris
        perceptual_roughness: 0.5,
        reflectance: 0.2,
        ..default()
    });
    let ash = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 0.0), // noir
        perceptual_roughness: 0.5,
        reflectance: 0.2,
        ..default()
    });
    MaterialHandles {
        gliadin,
        reactive_glutenin,
        bonded_glutenin,
        water,
        yeast,
        co2,
        ethanol,
        sugar,
        salt,
        ash,
    }
}

// Système pour mettre à jour les liaisons (bonds) entre molécules
fn update_bonds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sim_resource: Res<SimulationResource>,
    bond_query: Query<Entity, With<GlutenBond>>,
    particle_query: Query<(Entity, &MoleculeParticle), With<MoleculeParticle>>,
) {
    // Supprimer les anciennes liaisons
    for entity in bond_query.iter() {
        commands.entity(entity).despawn();
    }

    // Créer les nouvelles liaisons en tant que contraintes physiques
    let bond_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.3, 0.5), // Rouge clair
        ..default()
    });

    // Créer une map des entités par ID de molécule pour les liaisons
    let mut particle_map = std::collections::HashMap::new();
    for (entity, particle) in particle_query.iter() {
        particle_map.insert(particle.id, entity);
    }

    for bond in &sim_resource.state.bonds {
        if let (Some(mol_a), Some(mol_b)) = (
            sim_resource.state.grid.get_molecule(bond.molecule_a_id),
            sim_resource.state.grid.get_molecule(bond.molecule_b_id)
        ) {
            // Créer une contrainte physique entre les deux particules
            if let (Some(entity_a), Some(entity_b)) = (
                particle_map.get(&bond.molecule_a_id),
                particle_map.get(&bond.molecule_b_id)
            ) {
                // Créer une contrainte de distance entre les deux particules
                /* commands.spawn(
                    DistanceJoint::new(*entity_a, *entity_b)
                        .with_local_anchor_1(Vec3::ZERO)
                        .with_local_anchor_2(Vec3::ZERO)
                        .with_rest_length(bond.target_distance as f32)
                ); */
            }

            // Créer la visualisation de la liaison
            let pos_a = Vec3::new(
                mol_a.pos.x as f32,
                mol_a.pos.y as f32,
                mol_a.pos.z as f32,
            );

            let pos_b = Vec3::new(
                mol_b.pos.x as f32,
                mol_b.pos.y as f32,
                mol_b.pos.z as f32,
            );

            // Créer un cylindre pour représenter la liaison
            let bond_length = pos_a.distance(pos_b);
            let bond_center = (pos_a + pos_b) / 2.0;

            // Calculer la rotation pour aligner le cylindre entre les deux molécules
            let direction = (pos_b - pos_a).normalize();
            let up = Vec3::Y;
            let rotation = Quat::from_rotation_arc(up, direction);

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cylinder::new(5.0, bond_length)),
                    material: bond_material.clone(),
                    transform: Transform::from_translation(bond_center)
                        .with_rotation(rotation),
                    ..default()
                },
                GlutenBond {
                    molecule_a_id: bond.molecule_a_id,
                    molecule_b_id: bond.molecule_b_id,
                }
            ));
        }
    }
}

// Système pour gérer les entrées utilisateur
fn handle_user_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut sim_resource: ResMut<SimulationResource>,
    _time: Res<Time>,
) {
    // Ajouter du sel avec la touche 'S'
    if keyboard_input.just_pressed(KeyCode::KeyS) && !sim_resource.state.salt_added {
        sim_resource.state.add_salt();
        println!("Salt added!");
    }
    
    // Ajouter de la levure avec la touche 'Y'
    if keyboard_input.just_pressed(KeyCode::KeyY) && !sim_resource.state.yeast_added {
        sim_resource.state.add_yeast();
        println!("Yeast added!");
    }
    
    // Simuler un pli (fold) avec la touche 'C'
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        let center = nalgebra::Vector3::new(500.0, 360.0, 500.0);
        let force = nalgebra::Vector3::new(0.0, 30.0, 0.0);
        sim_resource.state.apply_force_to_region(center, 200.0, force);
        println!("Fold applied!");
    }
    
    // Réinitialiser avec la touche 'R'
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        sim_resource.state = SimulationState::new(1000.0, 720.0, 1000.0);
        sim_resource.state.initialize_classic_recipe();
        println!("Simulation reset!");
    }
}