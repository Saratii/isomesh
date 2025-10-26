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
use isomesh::manifold_dual_contouring::sampler::SphereSampler;
use isomesh::manifold_dual_contouring::{
    mdc::{mdc_mesh_generation, MeshBuffers},
    sampler::CuboidSampler,
};

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
    //create sphere
    let resolution = 32;
    let sphere_sampler = SphereSampler::new(Vec3::new(0.0, 0.0, 0.0), 20.0);
    let mut mesh_buffers = MeshBuffers::new();
    mdc_mesh_generation(
        0.5,
        &mut mesh_buffers,
        false,
        resolution,
        40.0,
        true,
        sphere_sampler,
    );
    let sphere_mesh = generate_bevy_mesh(mesh_buffers);
    commands.spawn((
        Mesh3d(meshes.add(sphere_mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(20.0, 0.0, -20.0),
    ));
    //create cuboid
    let resolution = 64;
    let size = Vec3::new(10.0, 15.0, 20.0);
    let cuboid_sampler = CuboidSampler::new(Vec3::new(0.0, 0.0, 0.0), size);
    let mut mesh_buffers = MeshBuffers::new();
    mdc_mesh_generation(
        0.5,
        &mut mesh_buffers,
        false,
        resolution,
        40.0,
        true,
        cuboid_sampler,
    );
    let cuboid_mesh = generate_bevy_mesh(mesh_buffers);
    commands.spawn((
        Mesh3d(meshes.add(cuboid_mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(-20.0, 0.0, 20.0),
    ));
    //light
    commands.spawn((
        PointLight {
            intensity: 15000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(40.0, 80.0, 40.0),
    ));
    //wireframe
    commands.insert_resource(WireframeConfig {
        global: true,
        default_color: Color::WHITE,
    });
    //camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(60.0, 60.0, 60.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));
}

pub fn generate_bevy_mesh(mesh_buffers: MeshBuffers) -> Mesh {
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
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
