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
use isomesh::{
    manifold_dual_contouring::sampler::CuboidSampler,
    marching_cubes::{
        color_provider::MaterialColorProvider,
        mc::{mc_mesh_generation_with_color, MeshBuffers},
    },
};
use isomesh::{
    manifold_dual_contouring::sampler::SphereSampler,
    marching_cubes::mc::{CUBES_PER_CHUNK_DIM, HALF_CHUNK, SDF_VALUES_PER_CHUNK_DIM},
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
    let sphere_sampler = SphereSampler::new(Vec3::ZERO, 20.0);
    let mut mesh_buffers = MeshBuffers::new();
    let densities = sphere_sampler.bake_quantized(
        Vec3::new(-HALF_CHUNK, -HALF_CHUNK, -HALF_CHUNK),
        Vec3::new(HALF_CHUNK, HALF_CHUNK, HALF_CHUNK),
        (
            SDF_VALUES_PER_CHUNK_DIM,
            SDF_VALUES_PER_CHUNK_DIM,
            SDF_VALUES_PER_CHUNK_DIM,
        ),
    );
    mc_mesh_generation_with_color(
        &mut mesh_buffers,
        &densities,
        &[1; SDF_VALUES_PER_CHUNK_DIM * SDF_VALUES_PER_CHUNK_DIM * SDF_VALUES_PER_CHUNK_DIM],
        CUBES_PER_CHUNK_DIM,
        SDF_VALUES_PER_CHUNK_DIM,
        &MaterialColorProvider,
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

    // Create cuboid with material 2 (grass color)
    let size = Vec3::new(10.0, 15.0, 20.0);
    let cuboid_sampler = CuboidSampler::new(Vec3::ZERO, size);
    let densities = cuboid_sampler.bake_quantized(
        Vec3::new(-HALF_CHUNK, -HALF_CHUNK, -HALF_CHUNK),
        Vec3::new(HALF_CHUNK, HALF_CHUNK, HALF_CHUNK),
        (
            SDF_VALUES_PER_CHUNK_DIM,
            SDF_VALUES_PER_CHUNK_DIM,
            SDF_VALUES_PER_CHUNK_DIM,
        ),
    );
    let mut mesh_buffers = MeshBuffers::new();
    mc_mesh_generation_with_color(
        &mut mesh_buffers,
        &densities,
        &[2; SDF_VALUES_PER_CHUNK_DIM * SDF_VALUES_PER_CHUNK_DIM * SDF_VALUES_PER_CHUNK_DIM],
        CUBES_PER_CHUNK_DIM,
        SDF_VALUES_PER_CHUNK_DIM,
        &MaterialColorProvider,
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

    // Light
    commands.spawn((
        PointLight {
            intensity: 15000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(40.0, 80.0, 40.0),
    ));

    // Wireframe
    commands.insert_resource(WireframeConfig {
        global: true,
        default_color: Color::WHITE,
    });

    // Camera
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
        uvs,
    } = mesh_buffers;
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}
