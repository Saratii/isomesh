use std::collections::HashMap;

use glam::Vec3;

use crate::marching_cubes::color_provider::{ColorProvider, NormalColorProvider, normal_to_color};
use crate::marching_cubes::tables::TRIANGLE_TABLE;

pub const SDF_VALUES_PER_CHUNK_DIM: usize = 64; // Number of voxel sample points
pub const VOXEL_SIZE: f32 = 2.0; // Size of each voxel
pub const CHUNK_SIZE: f32 = CUBES_PER_CHUNK_DIM as f32 * VOXEL_SIZE;
pub const CUBES_PER_CHUNK_DIM: usize = SDF_VALUES_PER_CHUNK_DIM - 1;

pub const VOXELS_PER_CHUNK: usize =
    SDF_VALUES_PER_CHUNK_DIM * SDF_VALUES_PER_CHUNK_DIM * SDF_VALUES_PER_CHUNK_DIM;
pub const HALF_CHUNK: f32 = CHUNK_SIZE / 2.0;

pub struct MeshBuffers {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    pub uvs: Vec<[f32; 2]>,
}

impl MeshBuffers {
    #[inline]
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
            uvs: Vec::new(),
        }
    }
}

const EDGE_VERTICES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0), // Bottom
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4), // Top
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7), // Vertical
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EdgeId {
    x: usize,
    y: usize,
    z: usize,
    direction: u8,
}

struct VertexCache<'a> {
    edge_to_vertex: HashMap<EdgeId, u32>,
    vertices: Vec<Vec3>,
    colors: Vec<[f32; 4]>,
    uvs: Vec<[f32; 2]>,
    color_provider: &'a dyn ColorProvider,
}

impl<'a> VertexCache<'a> {
    fn new(color_provider: &'a dyn ColorProvider) -> Self {
        Self {
            edge_to_vertex: HashMap::new(),
            vertices: Vec::new(),
            colors: Vec::new(),
            uvs: Vec::new(),
            color_provider,
        }
    }

    fn get_or_create_vertex(&mut self, edge_id: EdgeId, position: Vec3, material: u8) -> u32 {
        if let Some(&vertex_index) = self.edge_to_vertex.get(&edge_id) {
            vertex_index
        } else {
            let vertex_index = self.vertices.len() as u32;
            let color = self.color_provider.get_color(material, position);
            let uv = generate_uv_coordinates(position, material);
            self.vertices.push(position);
            self.colors.push(color);
            self.uvs.push(uv);
            self.edge_to_vertex.insert(edge_id, vertex_index);
            vertex_index
        }
    }
}

fn voxel_data_from_index(
    cube_x: usize,
    cube_y: usize,
    cube_z: usize,
    corner: usize,
    materials: &[u8],
    samples_per_chunk_dim: usize,
) -> u8 {
    let (dx, dy, dz) = match corner {
        0 => (0, 0, 0),
        1 => (1, 0, 0),
        2 => (1, 1, 0),
        3 => (0, 1, 0),
        4 => (0, 0, 1),
        5 => (1, 0, 1),
        6 => (1, 1, 1),
        7 => (0, 1, 1),
        _ => (0, 0, 0),
    };
    let x = cube_x + dx;
    let y = cube_y + dy;
    let z = cube_z + dz;
    let idx = z * samples_per_chunk_dim * samples_per_chunk_dim + y * samples_per_chunk_dim + x;
    materials[idx]
}

pub fn mc_mesh_generation(
    mesh_buffers: &mut MeshBuffers,
    densities: &[i16],
    materials: &[u8],
    cubes_per_chunk_dim: usize,
    samples_per_chunk_dim: usize,
) {
    mc_mesh_generation_with_color(
        mesh_buffers,
        densities,
        materials,
        cubes_per_chunk_dim,
        samples_per_chunk_dim,
        &NormalColorProvider,
    )
}

pub fn mc_mesh_generation_with_color(
    mesh_buffers: &mut MeshBuffers,
    densities: &[i16],
    materials: &[u8],
    cubes_per_chunk_dim: usize,
    samples_per_chunk_dim: usize,
    color_provider: &dyn ColorProvider,
) {
    let mut vertex_cache = VertexCache::new(color_provider);
    let mut indices = Vec::new();
    for x in 0..cubes_per_chunk_dim {
        for y in 0..cubes_per_chunk_dim {
            for z in 0..cubes_per_chunk_dim {
                process_cube_with_cache(
                    x,
                    y,
                    z,
                    &mut vertex_cache,
                    &mut indices,
                    densities,
                    materials,
                    samples_per_chunk_dim,
                );
            }
        }
    }
    build_mesh_from_cache_and_indices(
        vertex_cache,
        indices,
        densities,
        samples_per_chunk_dim,
        mesh_buffers,
        color_provider,
    );
}

fn calculate_cube_index(values: &[f32; 8]) -> u8 {
    let mut cube_index = 0;
    for i in 0..8 {
        if values[i] > 0.0 {
            cube_index |= 1 << i;
        }
    }
    cube_index
}

fn get_cube_vertices(x: usize, y: usize, z: usize) -> [Vec3; 8] {
    let start_pos = Vec3::splat(-HALF_CHUNK);
    let base_x = start_pos.x + x as f32 * VOXEL_SIZE;
    let base_y = start_pos.y + y as f32 * VOXEL_SIZE;
    let base_z = start_pos.z + z as f32 * VOXEL_SIZE;
    [
        Vec3::new(base_x, base_y, base_z),
        Vec3::new(base_x + VOXEL_SIZE, base_y, base_z),
        Vec3::new(base_x + VOXEL_SIZE, base_y + VOXEL_SIZE, base_z),
        Vec3::new(base_x, base_y + VOXEL_SIZE, base_z),
        Vec3::new(base_x, base_y, base_z + VOXEL_SIZE),
        Vec3::new(base_x + VOXEL_SIZE, base_y, base_z + VOXEL_SIZE),
        Vec3::new(
            base_x + VOXEL_SIZE,
            base_y + VOXEL_SIZE,
            base_z + VOXEL_SIZE,
        ),
        Vec3::new(base_x, base_y + VOXEL_SIZE, base_z + VOXEL_SIZE),
    ]
}

fn process_cube_with_cache(
    x: usize,
    y: usize,
    z: usize,
    vertex_cache: &mut VertexCache,
    indices: &mut Vec<u32>,
    densities: &[i16],
    materials: &[u8],
    samples_per_chunk_dim: usize,
) {
    let cube_vertices = get_cube_vertices(x, y, z);
    let cube_values = sample_cube_values_from_sdf(x, y, z, densities, samples_per_chunk_dim);
    let cube_index = calculate_cube_index(&cube_values);
    if cube_index == 0 || cube_index == 255 {
        return;
    }
    let triangles = triangulate_cube_with_cache(
        cube_index,
        &cube_vertices,
        &cube_values,
        x,
        y,
        z,
        vertex_cache,
        materials,
        samples_per_chunk_dim,
    );
    for triangle in triangles {
        indices.extend_from_slice(&triangle);
    }
}

fn get_edge_table_for_cube(cube_index: u8) -> &'static [i32] {
    &TRIANGLE_TABLE[cube_index as usize]
}

fn triangulate_cube_with_cache(
    cube_index: u8,
    vertices: &[Vec3; 8],
    values: &[f32; 8],
    cube_x: usize,
    cube_y: usize,
    cube_z: usize,
    vertex_cache: &mut VertexCache,
    materials: &[u8],
    samples_per_chunk_dim: usize,
) -> Vec<[u32; 3]> {
    let edge_table = get_edge_table_for_cube(cube_index);
    let mut result = Vec::new();
    let mut i = 0;
    while i < edge_table.len() && edge_table[i] != -1 {
        if i + 2 < edge_table.len() && edge_table[i + 1] != -1 && edge_table[i + 2] != -1 {
            let v1 = get_or_create_edge_vertex(
                edge_table[i] as usize,
                vertices,
                values,
                cube_x,
                cube_y,
                cube_z,
                vertex_cache,
                materials,
                samples_per_chunk_dim,
            );
            let v2 = get_or_create_edge_vertex(
                edge_table[i + 1] as usize,
                vertices,
                values,
                cube_x,
                cube_y,
                cube_z,
                vertex_cache,
                materials,
                samples_per_chunk_dim,
            );
            let v3 = get_or_create_edge_vertex(
                edge_table[i + 2] as usize,
                vertices,
                values,
                cube_x,
                cube_y,
                cube_z,
                vertex_cache,
                materials,
                samples_per_chunk_dim,
            );
            result.push([v1, v2, v3]);
            i += 3;
        } else {
            break;
        }
    }
    result
}

fn get_or_create_edge_vertex(
    edge_index: usize,
    vertices: &[Vec3; 8],
    values: &[f32; 8],
    cube_x: usize,
    cube_y: usize,
    cube_z: usize,
    vertex_cache: &mut VertexCache,
    materials: &[u8],
    samples_per_chunk_dim: usize,
) -> u32 {
    let edge_id = get_canonical_edge_id(edge_index, cube_x, cube_y, cube_z);
    let position = interpolate_edge(edge_index, vertices, values);
    let material = if vertex_cache.color_provider.needs_material() {
        let (v1_idx, v2_idx) = EDGE_VERTICES[edge_index];
        let val1 = values[v1_idx];
        let solid_corner = if val1 >= 0.0 { v1_idx } else { v2_idx };
        voxel_data_from_index(
            cube_x,
            cube_y,
            cube_z,
            solid_corner,
            materials,
            samples_per_chunk_dim,
        )
    } else {
        0
    };

    vertex_cache.get_or_create_vertex(edge_id, position, material)
}

fn get_canonical_edge_id(edge_index: usize, cube_x: usize, cube_y: usize, cube_z: usize) -> EdgeId {
    match edge_index {
        0 => EdgeId {
            x: cube_x,
            y: cube_y,
            z: cube_z,
            direction: 0,
        },
        1 => EdgeId {
            x: cube_x + 1,
            y: cube_y,
            z: cube_z,
            direction: 1,
        },
        2 => EdgeId {
            x: cube_x,
            y: cube_y + 1,
            z: cube_z,
            direction: 0,
        },
        3 => EdgeId {
            x: cube_x,
            y: cube_y,
            z: cube_z,
            direction: 1,
        },
        4 => EdgeId {
            x: cube_x,
            y: cube_y,
            z: cube_z + 1,
            direction: 0,
        },
        5 => EdgeId {
            x: cube_x + 1,
            y: cube_y,
            z: cube_z + 1,
            direction: 1,
        },
        6 => EdgeId {
            x: cube_x,
            y: cube_y + 1,
            z: cube_z + 1,
            direction: 0,
        },
        7 => EdgeId {
            x: cube_x,
            y: cube_y,
            z: cube_z + 1,
            direction: 1,
        },
        8 => EdgeId {
            x: cube_x,
            y: cube_y,
            z: cube_z,
            direction: 2,
        },
        9 => EdgeId {
            x: cube_x + 1,
            y: cube_y,
            z: cube_z,
            direction: 2,
        },
        10 => EdgeId {
            x: cube_x + 1,
            y: cube_y + 1,
            z: cube_z,
            direction: 2,
        },
        11 => EdgeId {
            x: cube_x,
            y: cube_y + 1,
            z: cube_z,
            direction: 2,
        },
        _ => EdgeId {
            x: cube_x,
            y: cube_y,
            z: cube_z,
            direction: 0,
        },
    }
}

fn interpolate_edge(edge_index: usize, vertices: &[Vec3; 8], values: &[f32; 8]) -> Vec3 {
    let (v1_idx, v2_idx) = EDGE_VERTICES[edge_index];
    let v1 = vertices[v1_idx];
    let v2 = vertices[v2_idx];
    let val1 = values[v1_idx];
    let val2 = values[v2_idx];
    let t = if (val2 - val1).abs() < 0.0001 {
        0.5
    } else {
        ((0.0 - val1) / (val2 - val1)).clamp(0.0, 1.0)
    };
    v1 + t * (v2 - v1)
}

fn sample_cube_values_from_sdf(
    x: usize,
    y: usize,
    z: usize,
    densities: &[i16],
    sdf_values_per_chunk_dim: usize,
) -> [f32; 8] {
    let get_sdf = |x: usize, y: usize, z: usize| -> f32 {
        let idx = z * sdf_values_per_chunk_dim * sdf_values_per_chunk_dim
            + y * sdf_values_per_chunk_dim
            + x;
        densities[idx] as f32
    };
    [
        get_sdf(x, y, z),
        get_sdf(x + 1, y, z),
        get_sdf(x + 1, y + 1, z),
        get_sdf(x, y + 1, z),
        get_sdf(x, y, z + 1),
        get_sdf(x + 1, y, z + 1),
        get_sdf(x + 1, y + 1, z + 1),
        get_sdf(x, y + 1, z + 1),
    ]
}

fn calculate_vertex_normal(
    point: Vec3,
    densities: &[i16],
    sdf_values_per_chunk_dim: usize,
) -> Vec3 {
    let epsilon = VOXEL_SIZE;
    let grad_x = sample_sdf_at_point_with_interpolation(
        point + Vec3::new(epsilon, 0.0, 0.0),
        densities,
        sdf_values_per_chunk_dim,
    ) - sample_sdf_at_point_with_interpolation(
        point - Vec3::new(epsilon, 0.0, 0.0),
        densities,
        sdf_values_per_chunk_dim,
    );
    let grad_y = sample_sdf_at_point_with_interpolation(
        point + Vec3::new(0.0, epsilon, 0.0),
        densities,
        sdf_values_per_chunk_dim,
    ) - sample_sdf_at_point_with_interpolation(
        point - Vec3::new(0.0, epsilon, 0.0),
        densities,
        sdf_values_per_chunk_dim,
    );
    let grad_z = sample_sdf_at_point_with_interpolation(
        point + Vec3::new(0.0, 0.0, epsilon),
        densities,
        sdf_values_per_chunk_dim,
    ) - sample_sdf_at_point_with_interpolation(
        point - Vec3::new(0.0, 0.0, epsilon),
        densities,
        sdf_values_per_chunk_dim,
    );

    Vec3::new(-grad_x, -grad_y, -grad_z).normalize_or_zero()
}

fn sample_sdf_at_point_with_interpolation(
    point: Vec3,
    densities: &[i16],
    samples_per_chunk_dim: usize,
) -> f32 {
    let voxel_x = (point.x + HALF_CHUNK) / VOXEL_SIZE;
    let voxel_y = (point.y + HALF_CHUNK) / VOXEL_SIZE;
    let voxel_z = (point.z + HALF_CHUNK) / VOXEL_SIZE;
    let voxel_x = voxel_x.clamp(0.0, (samples_per_chunk_dim - 1) as f32);
    let voxel_y = voxel_y.clamp(0.0, (samples_per_chunk_dim - 1) as f32);
    let voxel_z = voxel_z.clamp(0.0, (samples_per_chunk_dim - 1) as f32);
    let x0 = voxel_x.floor() as usize;
    let y0 = voxel_y.floor() as usize;
    let z0 = voxel_z.floor() as usize;
    let x1 = (x0 + 1).min(samples_per_chunk_dim - 1);
    let y1 = (y0 + 1).min(samples_per_chunk_dim - 1);
    let z1 = (z0 + 1).min(samples_per_chunk_dim - 1);
    let fx = voxel_x - x0 as f32;
    let fy = voxel_y - y0 as f32;
    let fz = voxel_z - z0 as f32;
    let get_sdf = |x: usize, y: usize, z: usize| -> f32 {
        let idx = z * samples_per_chunk_dim * samples_per_chunk_dim + y * samples_per_chunk_dim + x;
        densities[idx] as f32
    };
    let c000 = get_sdf(x0, y0, z0);
    let c100 = get_sdf(x1, y0, z0);
    let c010 = get_sdf(x0, y1, z0);
    let c110 = get_sdf(x1, y1, z0);
    let c001 = get_sdf(x0, y0, z1);
    let c101 = get_sdf(x1, y0, z1);
    let c011 = get_sdf(x0, y1, z1);
    let c111 = get_sdf(x1, y1, z1);
    let c00 = c000 * (1.0 - fx) + c100 * fx;
    let c10 = c010 * (1.0 - fx) + c110 * fx;
    let c01 = c001 * (1.0 - fx) + c101 * fx;
    let c11 = c011 * (1.0 - fx) + c111 * fx;
    let c0 = c00 * (1.0 - fy) + c10 * fy;
    let c1 = c01 * (1.0 - fy) + c11 * fy;
    c0 * (1.0 - fz) + c1 * fz
}

fn build_mesh_from_cache_and_indices(
    vertex_cache: VertexCache,
    indices: Vec<u32>,
    densities: &[i16],
    sdf_values_per_chunk_dim: usize,
    mesh_buffers: &mut MeshBuffers,
    color_provider: &dyn ColorProvider,
) {
    if vertex_cache.vertices.is_empty() {
        mesh_buffers.positions = Vec::new();
        mesh_buffers.normals = Vec::new();
        mesh_buffers.colors = Vec::new();
        mesh_buffers.indices = Vec::new();
        mesh_buffers.uvs = Vec::new();
    } else {
        let positions: Vec<[f32; 3]> = vertex_cache
            .vertices
            .iter()
            .map(|v| [v.x, v.y, v.z])
            .collect();
        let normals: Vec<[f32; 3]> = vertex_cache
            .vertices
            .iter()
            .map(|v| calculate_vertex_normal(*v, densities, sdf_values_per_chunk_dim).into())
            .collect();
        let colors: Vec<[f32; 4]> = if color_provider.needs_material() {
            vertex_cache.colors
        } else {
            let using_normal_coloring = vertex_cache
                .colors
                .iter()
                .all(|c| c[0] == 1.0 && c[1] == 1.0 && c[2] == 1.0);
            if using_normal_coloring {
                normals
                    .iter()
                    .map(|n| normal_to_color(Vec3::from_array(*n)))
                    .collect()
            } else {
                vertex_cache.colors
            }
        };
        let uvs: Vec<[f32; 2]> = vertex_cache.uvs;
        mesh_buffers.colors = colors;
        mesh_buffers.indices = indices;
        mesh_buffers.normals = normals;
        mesh_buffers.positions = positions;
        mesh_buffers.uvs = uvs;
    }
}

fn wrap01(v: f32) -> f32 {
    ((v % 1.0) + 1.0) % 1.0
}

fn generate_uv_coordinates(position: Vec3, _material: u8) -> [f32; 2] {
    let scale = 0.1;
    [wrap01(position.x * scale), wrap01(position.z * scale)]
}
