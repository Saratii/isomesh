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
use isomesh::{
    manifold_dual_contouring::sampler::CuboidSampler,
    marching_cubes::{
        color_provider::MaterialColorProvider,
        mc::{mc_mesh_generation, MeshBuffers},
    },
};

const SAMPLES_PER_CHUNK_DIM: usize = 64;
const BOUNDING_WIDTH: f32 = 64.0;
const HALF_EXTENT: f32 = BOUNDING_WIDTH / 2.0;

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
    let sphere_sampler = SphereSampler::new(Vec3::ZERO, 32.0);
    let mut mesh_buffers = MeshBuffers::new();
    let densities = sphere_sampler.bake_quantized(
        Vec3::new(-HALF_EXTENT, -HALF_EXTENT, -HALF_EXTENT),
        Vec3::new(HALF_EXTENT, HALF_EXTENT, HALF_EXTENT),
        (
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
        ),
    );
    mc_mesh_generation(
        &mut mesh_buffers,
        &densities,
        &[1; SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM],
        SAMPLES_PER_CHUNK_DIM,
        &MaterialColorProvider,
        HALF_EXTENT,
    );
    let sphere_mesh = generate_bevy_mesh(mesh_buffers);
    commands.spawn((
        Mesh3d(meshes.add(sphere_mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(40.0, 0.0, -40.0),
    ));
    // Create cuboid with material 2 (grass color)
    //1.1 is for padding so the cuboid fits in the bounding box
    let size = Vec3::new(HALF_EXTENT / 1.1, HALF_EXTENT / 1.1, HALF_EXTENT / 1.1);
    let cuboid_sampler = CuboidSampler::new(Vec3::ZERO, size);
    let densities = cuboid_sampler.bake_quantized(
        Vec3::new(-HALF_EXTENT, -HALF_EXTENT, -HALF_EXTENT),
        Vec3::new(HALF_EXTENT, HALF_EXTENT, HALF_EXTENT),
        (
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
        ),
    );
    let mut mesh_buffers = MeshBuffers::new();
    mc_mesh_generation(
        &mut mesh_buffers,
        &densities,
        &[2; SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM],
        SAMPLES_PER_CHUNK_DIM,
        &MaterialColorProvider,
        HALF_EXTENT,
    );
    let cuboid_mesh = generate_bevy_mesh(mesh_buffers);
    commands.spawn((
        Mesh3d(meshes.add(cuboid_mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(-40.0, 0.0, 40.0),
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
        Transform::from_xyz(90.0, 90.0, 90.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
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
