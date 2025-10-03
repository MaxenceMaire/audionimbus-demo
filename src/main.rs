use std::io::Read as _;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    post_process::bloom::Bloom,
    prelude::*,
};

use crate::camera_controller::CameraController;

mod audio;
mod camera_controller;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "audionimbus".to_string(),
                mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(audio::Plugin)
        .add_plugins(camera_controller::CameraControllerPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut audio: ResMut<audio::Audio>,
) {
    let sphere = meshes.add(Sphere { radius: 0.1 });
    let sphere_material = materials.add(StandardMaterial {
        emissive: LinearRgba {
            red: 0.0,
            green: 0.0,
            blue: 1000.0,
            alpha: 1.0,
        },
        ..default()
    });
    let simulation_flags =
        audionimbus::SimulationFlags::DIRECT | audionimbus::SimulationFlags::REFLECTIONS;
    let source = audionimbus::Source::try_new(
        &audio.simulator,
        &audionimbus::SourceSettings {
            flags: simulation_flags,
        },
    )
    .unwrap();
    audio.simulator.add_source(&source);
    let source = audionimbus::Source::try_new(
        &audio.simulator,
        &audionimbus::SourceSettings {
            flags: simulation_flags,
        },
    )
    .unwrap();
    audio.simulator.add_source(&source);
    audio.simulator.commit();

    let assets = std::path::Path::new(env!("OUT_DIR")).join("assets");
    let file = std::fs::File::open(assets.join("piano.raw")).unwrap();
    let mut reader = std::io::BufReader::new(file);
    let mut samples: Vec<f32> = Vec::new();
    let mut buffer = [0u8; 4]; // f32 is 4 bytes
    while reader.read_exact(&mut buffer).is_ok() {
        let sample = f32::from_le_bytes(buffer);
        samples.push(sample);
    }

    #[cfg(not(any(feature = "direct", feature = "reverb")))]
    {
        let source_position = Transform::from_xyz(0.0, 2.0, 0.0);
        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(sphere_material.clone()),
            source_position,
            audio::AudioSource {
                source,
                data: samples,
                is_repeating: true,
                position: 0,
            },
        ));
        commands.spawn((
            source_position,
            PointLight {
                color: Color::Srgba(Srgba {
                    red: 0.8,
                    green: 0.8,
                    blue: 1.0,
                    alpha: 1.0,
                }),
                ..Default::default()
            },
        ));

        commands.spawn((
            Transform::from_xyz(28.0, 10.0, -8.0),
            PointLight {
                intensity: 5000000.0,
                color: Color::Srgba(Srgba {
                    red: 0.8,
                    green: 0.8,
                    blue: 1.0,
                    alpha: 1.0,
                }),
                ..Default::default()
            },
        ));
    }
    #[cfg(feature = "direct")]
    {
        let source_position = Transform::from_xyz(0.0, 2.0, 0.0);
        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(sphere_material.clone()),
            source_position,
            audio::AudioSource {
                source,
                data: samples,
                is_repeating: true,
                position: 0,
            },
        ));
        commands.spawn((
            source_position,
            PointLight {
                intensity: 500000.0,
                color: Color::Srgba(Srgba {
                    red: 0.8,
                    green: 0.8,
                    blue: 1.0,
                    alpha: 1.0,
                }),
                ..Default::default()
            },
        ));
    }
    #[cfg(feature = "reverb")]
    {
        let source_position = Transform::from_xyz(0.0, 8.0, -10.0);
        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(sphere_material.clone()),
            source_position,
            audio::AudioSource {
                source,
                data: samples,
                is_repeating: true,
                position: 0,
            },
        ));
        commands.spawn((
            source_position,
            PointLight {
                intensity: 30000000.0,
                color: Color::Srgba(Srgba {
                    red: 0.8,
                    green: 0.8,
                    blue: 1.0,
                    alpha: 1.0,
                }),
                ..Default::default()
            },
        ));
    }

    for (vertices, normal) in TOPOLOGY {
        let normal = [normal[1], normal[2], normal[0]];
        commands.spawn((
            Mesh3d(
                meshes.add(
                    Mesh::new(
                        PrimitiveTopology::TriangleList,
                        RenderAssetUsages::default(),
                    )
                    .with_inserted_attribute(
                        Mesh::ATTRIBUTE_POSITION,
                        vertices
                            .iter()
                            .map(|vertex| [vertex[1], vertex[2], vertex[0]])
                            .collect::<Vec<_>>(),
                    )
                    .with_inserted_indices(Indices::U32(vec![0, 3, 1, 1, 3, 2]))
                    .with_inserted_attribute(
                        Mesh::ATTRIBUTE_NORMAL,
                        vec![normal, normal, normal, normal],
                    ),
                ),
            ),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::Srgba(bevy::color::palettes::basic::SILVER),
                double_sided: true,
                cull_mode: None,
                ..default()
            })),
        ));

        let surface = audionimbus::StaticMesh::try_new(
            &audio.scene,
            &audionimbus::StaticMeshSettings {
                vertices: &vertices
                    .iter()
                    .map(|vertex| audionimbus::Point::new(vertex[1], vertex[2], vertex[0]))
                    .collect::<Vec<_>>(),
                triangles: &[
                    audionimbus::Triangle::new(0, 1, 2),
                    audionimbus::Triangle::new(0, 2, 3),
                ],
                material_indices: &[0, 0, 0, 0, 0, 0, 0, 0],
                materials: &[audionimbus::Material::WOOD],
            },
        )
        .unwrap();
        audio.scene.add_static_mesh(&surface);
    }
    audio.scene.commit();

    commands.insert_resource(AmbientLight {
        brightness: 200.0,
        ..Default::default()
    });

    commands.spawn((
        CameraController::default(),
        Camera3d::default(),
        Bloom::NATURAL,
        Transform::from_xyz(-0.45, 2.17, 10.0),
    ));
}

// Blender vertex coordinates
#[cfg(not(any(feature = "direct", feature = "reverb")))]
const TOPOLOGY: [([[f32; 3]; 4], [f32; 3]); 23] = [
    (
        [
            // Start cooridor floor
            [-2.0, -2.0, 0.0],
            [-2.0, 2.0, 0.0],
            [14.0, 2.0, 0.0],
            [14.0, -2.0, 0.0],
        ],
        [0.0, 0.0, 1.0],
    ),
    (
        [
            // Start cooridor ceiling
            [-2.0, -2.0, 4.0],
            [-2.0, 2.0, 4.0],
            [14.0, 2.0, 4.0],
            [14.0, -2.0, 4.0],
        ],
        [0.0, 0.0, 1.0],
    ),
    (
        [
            // Start cooridor left wall
            [-2.0, -2.0, 4.0],
            [-2.0, -2.0, 0.0],
            [14.0, -2.0, 0.0],
            [14.0, -2.0, 4.0],
        ],
        [0.0, 1.0, 0.0],
    ),
    (
        [
            // Start cooridor right wall
            [2.0, 2.0, 4.0],
            [2.0, 2.0, 0.0],
            [14.0, 2.0, 0.0],
            [14.0, 2.0, 4.0],
        ],
        [0.0, 1.0, 0.0],
    ),
    (
        [
            // Start cooridor front wall
            [-2.0, -2.0, 0.0],
            [-2.0, 2.0, 0.0],
            [-2.0, 2.0, 4.0],
            [-2.0, -2.0, 4.0],
        ],
        [-1.0, 0.0, 0.0],
    ),
    (
        [
            // Start cooridor back wall
            [14.0, -2.0, 0.0],
            [14.0, 2.0, 0.0],
            [14.0, 2.0, 4.0],
            [14.0, -2.0, 4.0],
        ],
        [1.0, 0.0, 0.0],
    ),
    (
        [
            // Start transition floor
            [2.0, 2.0, 0.0],
            [2.0, 6.0, 0.0],
            [-2.0, 6.0, 0.0],
            [-2.0, 2.0, 0.0],
        ],
        [0.0, 0.0, -1.0],
    ),
    (
        [
            // Start transition ceiling
            [2.0, 2.0, 4.0],
            [2.0, 6.0, 4.0],
            [-2.0, 6.0, 4.0],
            [-2.0, 2.0, 4.0],
        ],
        [0.0, 0.0, -1.0],
    ),
    (
        [
            // Start transition back wall
            [2.0, 2.0, 0.0],
            [2.0, 6.0, 0.0],
            [2.0, 6.0, 4.0],
            [2.0, 2.0, 4.0],
        ],
        [-1.0, 0.0, 0.0],
    ),
    (
        [
            // Snake floor
            [-2.0, -6.0, 0.0],
            [-2.0, 6.0, 0.0],
            [-10.0, 6.0, 0.0],
            [-10.0, -6.0, 0.0],
        ],
        [0.0, 0.0, -1.0],
    ),
    (
        [
            // Snake ceiling
            [-2.0, -6.0, 4.0],
            [-2.0, 6.0, 4.0],
            [-10.0, 6.0, 4.0],
            [-10.0, -6.0, 4.0],
        ],
        [0.0, 0.0, -1.0],
    ),
    (
        [
            // Snake left wall
            [-2.0, -6.0, 0.0],
            [-10.0, -6.0, 0.0],
            [-10.0, -6.0, 4.0],
            [-2.0, -6.0, 4.0],
        ],
        [0.0, 0.0, -1.0],
    ),
    (
        [
            // Snake front wall
            [-10.0, -6.0, 0.0],
            [-10.0, 6.0, 0.0],
            [-10.0, 6.0, 4.0],
            [-10.0, -6.0, 4.0],
        ],
        [-1.0, 0.0, 0.0],
    ),
    (
        [
            // Snake separation wall
            [-6.0, -2.0, 0.0],
            [-6.0, 6.0, 0.0],
            [-6.0, 6.0, 4.0],
            [-6.0, -2.0, 4.0],
        ],
        [-1.0, 0.0, 0.0],
    ),
    (
        [
            // Snake back wall
            [-2.0, -6.0, 0.0],
            [-2.0, -2.0, 0.0],
            [-2.0, -2.0, 4.0],
            [-2.0, -6.0, 4.0],
        ],
        [-1.0, 0.0, 0.0],
    ),
    (
        [
            // Cathedral floor
            [2.0, 6.0, 0.0],
            [-18.0, 6.0, 0.0],
            [-18.0, 38.0, 0.0],
            [2.0, 38.0, 0.0],
        ],
        [0.0, 0.0, 1.0],
    ),
    (
        [
            // Cathedral ceiling
            [2.0, 6.0, 20.0],
            [-18.0, 6.0, 20.0],
            [-18.0, 38.0, 20.0],
            [2.0, 38.0, 20.0],
        ],
        [0.0, 0.0, 1.0],
    ),
    (
        [
            // Cathedral left wall 0
            [2.0, 6.0, 0.0],
            [-6.0, 6.0, 0.0],
            [-6.0, 6.0, 4.0],
            [2.0, 6.0, 4.0],
        ],
        [0.0, 1.0, 0.0],
    ),
    (
        [
            // Cathedral left wall 1
            [-10.0, 6.0, 0.0],
            [-18.0, 6.0, 0.0],
            [-18.0, 6.0, 4.0],
            [-10.0, 6.0, 4.0],
        ],
        [0.0, 1.0, 0.0],
    ),
    (
        [
            // Cathedral left wall upper
            [2.0, 6.0, 4.0],
            [-18.0, 6.0, 4.0],
            [-18.0, 6.0, 20.0],
            [2.0, 6.0, 20.0],
        ],
        [0.0, 1.0, 0.0],
    ),
    (
        [
            // Cathedral right wall
            [-18.0, 38.0, 0.0],
            [2.0, 38.0, 0.0],
            [2.0, 38.0, 20.0],
            [-18.0, 38.0, 20.0],
        ],
        [0.0, 1.0, 0.0],
    ),
    (
        [
            // Cathedral front wall
            [-18.0, 6.0, 0.0],
            [-18.0, 38.0, 0.0],
            [-18.0, 38.0, 20.0],
            [-18.0, 6.0, 20.0],
        ],
        [-1.0, 0.0, 0.0],
    ),
    (
        [
            // Cathedral back wall
            [2.0, 6.0, 0.0],
            [2.0, 38.0, 0.0],
            [2.0, 38.0, 20.0],
            [2.0, 6.0, 20.0],
        ],
        [-1.0, 0.0, 0.0],
    ),
];
#[cfg(feature = "direct")]
const TOPOLOGY: [([[f32; 3]; 4], [f32; 3]); 4] = [
    (
        [
            // Floor
            [2.0, -2.0, 0.0],
            [2.0, 2.0, 0.0],
            [-2.0, 2.0, 0.0],
            [-2.0, -2.0, 0.0],
        ],
        [0.0, 0.0, -1.0],
    ),
    (
        [
            // Ceiling
            [2.0, -2.0, 4.0],
            [2.0, 2.0, 4.0],
            [-2.0, 2.0, 4.0],
            [-2.0, -2.0, 4.0],
        ],
        [0.0, 0.0, -1.0],
    ),
    (
        [
            // Left wall
            [-2.0, -2.0, 0.0],
            [2.0, -2.0, 0.0],
            [2.0, -2.0, 4.0],
            [-2.0, -2.0, 4.0],
        ],
        [0.0, 1.0, 0.0],
    ),
    (
        [
            // Front wall
            [-2.0, -2.0, 0.0],
            [-2.0, 2.0, 0.0],
            [-2.0, 2.0, 4.0],
            [-2.0, -2.0, 4.0],
        ],
        [-1.0, 0.0, 0.0],
    ),
];
#[cfg(feature = "reverb")]
const TOPOLOGY: [([[f32; 3]; 4], [f32; 3]); 6] = [
    (
        [
            // Cathedral floor
            [20.0, -10.0, 0.0],
            [20.0, 10.0, 0.0],
            [-20.0, 10.0, 0.0],
            [-20.0, -10.0, 0.0],
        ],
        [0.0, 0.0, -1.0],
    ),
    (
        [
            // Cathedral ceiling
            [20.0, -10.0, 20.0],
            [20.0, 10.0, 20.0],
            [-20.0, 10.0, 20.0],
            [-20.0, -10.0, 20.0],
        ],
        [0.0, 0.0, -1.0],
    ),
    (
        [
            // Cathedral left wall
            [-20.0, -10.0, 0.0],
            [20.0, -10.0, 0.0],
            [20.0, -10.0, 20.0],
            [-20.0, -10.0, 20.0],
        ],
        [0.0, 1.0, 0.0],
    ),
    (
        [
            // Cathedral right wall
            [20.0, 10.0, 0.0],
            [-20.0, 10.0, 0.0],
            [-20.0, 10.0, 20.0],
            [20.0, 10.0, 20.0],
        ],
        [0.0, -1.0, 0.0],
    ),
    (
        [
            // Cathedral front wall
            [-20.0, 10.0, 0.0],
            [-20.0, -10.0, 0.0],
            [-20.0, -10.0, 20.0],
            [-20.0, 10.0, 20.0],
        ],
        [1.0, 0.0, 0.0],
    ),
    (
        [
            // Cathedral back wall
            [20.0, -10.0, 0.0],
            [20.0, 10.0, 0.0],
            [20.0, 10.0, 20.0],
            [20.0, -10.0, 20.0],
        ],
        [-1.0, 0.0, 0.0],
    ),
];
