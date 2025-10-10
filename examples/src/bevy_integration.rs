use std::time::Instant;

use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        mesh::Indices,
        render_asset::RenderAssetUsages,
        render_resource::{PrimitiveTopology, WgpuFeatures},
        settings::WgpuSettings,
        RenderPlugin,
    },
};

use isomesh::mdc::{mdc_mesh_generation, MeshBuffers};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }
                .into(),
                ..default()
            }),
            WireframePlugin,
        ))
        .add_systems(Startup, setup_mdc)
        .run();
}

fn setup_mdc(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let resolution = 64;
    let mut mesh_buffers = MeshBuffers::new();
    println!("Starting MDC contour...");
    let t0 = Instant::now();
    mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true);
    println!("MDC contour finished in {} ms", t0.elapsed().as_millis());
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let MeshBuffers {
        positions,
        normals,
        colors,
        indices,
    } = mesh_buffers;
    let vertex_count = positions.len();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    println!(
        "Mesh created with {} vertices and {} triangles",
        vertex_count,
        vertex_count / 3
    );
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    commands.spawn((
        PointLight {
            intensity: 15000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(40.0, 80.0, 40.0),
    ));
    commands.insert_resource(WireframeConfig {
        global: true,
        default_color: Color::WHITE,
    });
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(80.0, 80.0, 80.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));
}
