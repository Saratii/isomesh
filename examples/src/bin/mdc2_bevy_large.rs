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
    mdc::sampler::{FunSurfaceSampler, Sampler},
    mdc2::{mdc::{build_octree, cluster_cell_base, generate_vertex_buffer, process_cell}, octree::MeshVertex},
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
    let resolution = 512;
    let fun_blob = FunSurfaceSampler::new(Vec3::new(0.0, 0.0, 0.0), 40.0);
    let mesh = generate_mesh_from_sampler(fun_blob, resolution);
    if let Some(mesh) = mesh {
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.5, 0.8),
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(20.0, 0.0, -20.0),
        ));
    }
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
        Transform::from_xyz(60.0, 60.0, 60.0).looking_at(Vec3::new(32.0, 32.0, 32.0), Vec3::Y),
    ));
}

fn generate_mesh_from_sampler<S: Sampler>(sampler: S, resolution: i32) -> Option<Mesh> {
    // Center the octree at origin
    let half_res = resolution / 2;
    let mut vertices = Vec::new();
    let mut root = build_octree(
        IVec3::new(-half_res, -half_res, -half_res),
        resolution,
        sampler,
    );
    cluster_cell_base(&mut root, 0.5);
    generate_vertex_buffer(&mut root, &mut vertices);
    let mut indexes = Vec::new();
    let mut tri_count = Vec::new();
    process_cell(&root, &mut indexes, &mut tri_count, 0.5, true);
    if vertices.is_empty() || indexes.is_empty() {
        println!("WARNING: No geometry generated!");
        return None;
    }
    println!(
        "Generated {} vertices, {} triangles",
        vertices.len(),
        tri_count.len()
    );
    Some(generate_bevy_mesh(vertices, indexes))
}

pub fn generate_bevy_mesh(vertices: Vec<MeshVertex>, indices: Vec<i32>) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let positions: Vec<[f32; 3]> = vertices.iter().map(|v| v.pos.to_array()).collect();
    let normals: Vec<[f32; 3]> = vertices.iter().map(|v| v.normal.to_array()).collect();
    let colors: Vec<[f32; 4]> = vertices
        .iter()
        .map(|v| [v.color.x, v.color.y, v.color.z, 1.0])
        .collect();

    // Decode indices - remove the flip bit marker (0x10000000)
    let indices_u32: Vec<u32> = indices
        .iter()
        .map(|i| {
            let idx = (i & 0x0FFFFFFF) as u32;
            idx
        })
        .collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices_u32));

    mesh
}
