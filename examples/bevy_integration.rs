use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        RenderPlugin,
        mesh::Indices,
        render_asset::RenderAssetUsages,
        render_resource::{PrimitiveTopology, WgpuFeatures},
        settings::WgpuSettings,
    },
};

use isomesh::mdc::MDC;

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
    let mut mdc = MDC::new(resolution, resolution);
    println!("Starting MDC contour...");
    let duration = mdc.contour(0.5);
    println!("MDC contour completed in {} ms", duration);
    // Convert MDC vertices to Bevy mesh
    if mdc.vertex_count > 0 {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        // Extract positions
        let positions: Vec<[f32; 3]> = mdc
            .vertices_dn
            .iter()
            .map(|v| [v.position.x, v.position.y, v.position.z])
            .collect();
        // Extract normals
        let normals: Vec<[f32; 3]> = mdc
            .vertices_dn
            .iter()
            .map(|v| [v.normal.x, v.normal.y, v.normal.z])
            .collect();
        let colors: Vec<[f32; 4]> = mdc.vertices_dn.iter().map(|v| v.color).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        if MDC::FLAT_SHADING {
            let indices: Vec<u32> = (0..mdc.vertex_count as u32).collect();
            mesh.insert_indices(Indices::U32(indices));
        } else {
            let indices: Vec<u32> = mdc.indices.iter().map(|&i| i as u32).collect();
            mesh.insert_indices(Indices::U32(indices));
        }
        println!(
            "Mesh created with {} vertices and {} triangles",
            mdc.vertex_count,
            mdc.vertex_count / 3
        );
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
    }
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
