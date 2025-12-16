use bevy::prelude::*;
use bevy::asset::AssetServer;
use bevy::ui::*;
// Resource pour stocker l'entité du texte d'UI
#[derive(Resource, Default)]
struct UiTextEntity(Option<Entity>);
use bevy::input::mouse::{MouseMotion, MouseButtonInput};
// Composant pour la caméra orbitale
#[derive(Component)]
struct OrbitCamera {
    pub radius: f32,
    pub azimuth: f32,
    pub elevation: f32,
}
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
            .init_resource::<SimulationResource>()
            .add_plugins(PhysicsPlugins::default())
            .add_systems(Startup, setup_ui_panel)
            .add_systems(Startup, setup)
            .add_systems(Update, (
                update_particles,
                update_bonds,
                handle_user_input,
                orbit_camera_control,
                update_ui_panel,
            ))
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_plugins(LogDiagnosticsPlugin::default());
    // Système d'initialisation du panneau d'UI
    fn setup_ui_panel(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        let font: Handle<Font> = asset_server.load("fonts/FiraMono-Medium.ttf");
        let ui_entity = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(340.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            background_color: Color::rgba(0.08, 0.08, 0.12, 0.92).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Chargement...",
                    TextStyle {
                        font: font.clone(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                ),
                style: Style {
                    margin: UiRect::all(Val::Px(18.0)),
                    ..default()
                },
                ..default()
            });
        })
        .id();
        commands.insert_resource(UiTextEntity(Some(ui_entity)));
    }
    // Système pour mettre à jour le panneau d'UI avec les données de la simulation
    fn update_ui_panel(
        sim_resource: Res<SimulationResource>,
        ui_text: Res<UiTextEntity>,
        mut text_query: Query<&mut Text>,
        children_query: Query<&Children>,
    ) {
        if let Some(panel_entity) = ui_text.0 {
            if let Ok(children) = children_query.get(panel_entity) {
                if let Some(&text_entity) = children.first() {
                    if let Ok(mut text) = text_query.get_mut(text_entity) {
                        let state = &sim_resource.state;
                        let flour = state.grid.get_all_molecules().iter().filter(|m| matches!(m.mol_type, MoleculeType::Gliadin | MoleculeType::Glutenin { .. })).count();
                        let water = state.grid.get_all_molecules().iter().filter(|m| matches!(m.mol_type, MoleculeType::Water)).count();
                        let yeast = state.grid.get_all_molecules().iter().filter(|m| matches!(m.mol_type, MoleculeType::Yeast)).count();
                        let co2 = state.grid.get_all_molecules().iter().filter(|m| matches!(m.mol_type, MoleculeType::CO2)).count();
                        let ethanol = state.grid.get_all_molecules().iter().filter(|m| matches!(m.mol_type, MoleculeType::Ethanol)).count();
                        let sugar = state.grid.get_all_molecules().iter().filter(|m| matches!(m.mol_type, MoleculeType::Sugar)).count();
                        let salt = state.grid.get_all_molecules().iter().filter(|m| matches!(m.mol_type, MoleculeType::Salt)).count();
                        let ash = state.grid.get_all_molecules().iter().filter(|m| matches!(m.mol_type, MoleculeType::Ash)).count();
                        let bonds = state.bonds.len();
                        let time = state.time_elapsed;
                        let temp = state.temperature;
                        // Détermination de la phase
                        let phase = if !state.salt_added && !state.yeast_added {
                            "Autolyse"
                        } else if state.salt_added && !state.yeast_added {
                            "Après sel, avant levure"
                        } else if state.salt_added && state.yeast_added {
                            "Fermentation"
                        } else {
                            "Préparation"
                        };
                        text.sections[0].value = format!(
                            "House of pain 3D - Simulation\n\n[Appuyez sur S pour ajouter du sel]\n[Appuyez sur Y pour ajouter de la levure]\n\nPhase: {phase}\nTempérature: {temp:.1} °C\nTemps: {time:.1} s\nFarine: {flour}\nEau: {water}\nLevure: {yeast}\nCO₂: {co2}\nEthanol: {ethanol}\nSucre: {sugar}\nSel: {salt}\nCendres: {ash}\nLiaisons gluten: {bonds}",
                            phase=phase, temp=temp, time=time, flour=flour, water=water, yeast=yeast, co2=co2, ethanol=ethanol, sugar=sugar, salt=salt, ash=ash, bonds=bonds
                        );
                    }
                }
            }
        }
    }
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
    // Caméra orbitale initiale
    let center = Vec3::new(500.0, 360.0, 500.0);
    let radius = 1200.0;
    let azimuth = std::f32::consts::FRAC_PI_4; // 45°
    let elevation = std::f32::consts::FRAC_PI_6; // 30°
    let (x, y, z) = (
        center.x + radius * azimuth.cos() * elevation.cos(),
        center.y + radius * elevation.sin(),
        center.z + radius * azimuth.sin() * elevation.cos(),
    );
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(x, y, z).looking_at(center, Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                far: 5000.0,
                ..default()
            }),
            ..default()
        },
        OrbitCamera { radius, azimuth, elevation },
    ));
}

// Système pour contrôler la caméra orbitale avec la souris et le clavier
fn orbit_camera_control(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    let mut delta_azimuth = 0.0f32;
    let mut delta_elevation = 0.0f32;
    let dragging = mouse_button_input.pressed(MouseButton::Left);

    // Clavier : flèches pour tourner
    let keyboard_speed = 0.02;
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        delta_azimuth -= keyboard_speed;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        delta_azimuth += keyboard_speed;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        delta_elevation += keyboard_speed;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        delta_elevation -= keyboard_speed;
    }

    // Souris : drag pour tourner
    if dragging {
        for ev in mouse_motion_events.read() {
            delta_azimuth -= ev.delta.x * 0.005;
            delta_elevation -= ev.delta.y * 0.005;
        }
    }

    for (mut transform, mut orbit) in query.iter_mut() {
        if delta_azimuth != 0.0 || delta_elevation != 0.0 {
            orbit.azimuth += delta_azimuth;
            orbit.elevation = (orbit.elevation + delta_elevation).clamp(-1.4, 1.4);
        }
        // Calculer la nouvelle position
        let center = Vec3::new(500.0, 360.0, 500.0);
        let (x, y, z) = (
            center.x + orbit.radius * orbit.azimuth.cos() * orbit.elevation.cos(),
            center.y + orbit.radius * orbit.elevation.sin(),
            center.z + orbit.radius * orbit.azimuth.sin() * orbit.elevation.cos(),
        );
        transform.translation = Vec3::new(x, y, z);
        transform.look_at(center, Vec3::Y);
    }
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
    // Calculer le nombre de liaisons par molécule (pour les glutenines)
    let mut bond_count: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
    for bond in &sim_state.bonds {
        *bond_count.entry(bond.molecule_a_id).or_insert(0) += 1;
        *bond_count.entry(bond.molecule_b_id).or_insert(0) += 1;
    }

    for molecule in sim_state.grid.get_all_molecules() {
        let pos = Vec3::new(
            molecule.pos.x as f32,
            molecule.pos.y as f32,
            molecule.pos.z as f32,
        );
        // Taille de base
        let mut scale = Vec3::ONE;
        // Si c'est une glutenine, on grossit selon le nombre de liaisons
        if let MoleculeType::Glutenin { .. } = molecule.mol_type {
            let n_bonds = bond_count.get(&molecule.id).copied().unwrap_or(0);
            // 1.0 (seule) à 2.0 (très liée)
            scale = Vec3::splat(1.0 + (n_bonds as f32 * 0.3).min(1.0));
        }
        if let Some(entity) = entity_map.get(&molecule.id) {
            if let Ok((_, mut transform, _)) = particle_query.get_mut(*entity) {
                transform.translation = pos;
                transform.scale = scale;
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
                    transform: Transform::from_translation(pos).with_scale(scale),
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