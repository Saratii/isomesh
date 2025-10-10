// Manifold Dual Contouring
// https://github.com/Lin20/isosurface/tree/master/Isosurface/Isosurface/ManifoldDC

use glam::Vec3;

use crate::octree::{NodeType, OctreeNode};

pub struct MeshBuffers {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<i32>,
}

impl MeshBuffers {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
        }
    }
}

pub fn set_enforce_manifold(enforce: bool) {
    OctreeNode::set_enforce_manifold(enforce);
}

pub fn mdc_mesh_generation(
    threshold: f32,
    mesh_buffers: &mut MeshBuffers,
    flat_shading: bool,
    resolution: i32,
    enforce_manifold: bool,
) {
    OctreeNode::set_enforce_manifold(enforce_manifold);
    let mut tree = Box::new(OctreeNode::new());
    tree.construct_base(resolution, mesh_buffers);
    tree.cluster_cell_base(0.0);
    tree.generate_vertex_buffer(mesh_buffers);
    calculate_indexes(&tree, threshold, mesh_buffers, flat_shading);
}

pub fn construct_tree_grid(node: Option<&OctreeNode>) {
    if let Some(n) = node {
        let x = n.position.x as i32;
        let y = n.position.y as i32;
        let z = n.position.z as i32;
        let size = n.size as f32;
        let mut vs = Vec::new();
        let c = [0.69, 0.77, 0.87, 1.0];
        vs.push((
            Vec3::new(
                x as f32 + 0.0 * size,
                y as f32 + 0.0 * size,
                z as f32 + 0.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 1.0 * size,
                y as f32 + 0.0 * size,
                z as f32 + 0.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 1.0 * size,
                y as f32 + 0.0 * size,
                z as f32 + 0.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 1.0 * size,
                y as f32 + 1.0 * size,
                z as f32 + 0.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 1.0 * size,
                y as f32 + 1.0 * size,
                z as f32 + 0.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 0.0 * size,
                y as f32 + 1.0 * size,
                z as f32 + 0.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 0.0 * size,
                y as f32 + 1.0 * size,
                z as f32 + 0.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 0.0 * size,
                y as f32 + 0.0 * size,
                z as f32 + 0.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 0.0 * size,
                y as f32 + 0.0 * size,
                z as f32 + 1.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 1.0 * size,
                y as f32 + 0.0 * size,
                z as f32 + 1.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 1.0 * size,
                y as f32 + 0.0 * size,
                z as f32 + 1.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 1.0 * size,
                y as f32 + 1.0 * size,
                z as f32 + 1.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 1.0 * size,
                y as f32 + 1.0 * size,
                z as f32 + 1.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 0.0 * size,
                y as f32 + 1.0 * size,
                z as f32 + 1.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 0.0 * size,
                y as f32 + 1.0 * size,
                z as f32 + 1.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 0.0 * size,
                y as f32 + 0.0 * size,
                z as f32 + 1.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 0.0 * size,
                y as f32 + 0.0 * size,
                z as f32 + 0.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 0.0 * size,
                y as f32 + 0.0 * size,
                z as f32 + 1.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 0.0 * size,
                y as f32 + 1.0 * size,
                z as f32 + 0.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 0.0 * size,
                y as f32 + 1.0 * size,
                z as f32 + 1.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 1.0 * size,
                y as f32 + 0.0 * size,
                z as f32 + 0.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 1.0 * size,
                y as f32 + 0.0 * size,
                z as f32 + 1.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 1.0 * size,
                y as f32 + 1.0 * size,
                z as f32 + 0.0 * size,
            ),
            c,
        ));
        vs.push((
            Vec3::new(
                x as f32 + 1.0 * size,
                y as f32 + 1.0 * size,
                z as f32 + 1.0 * size,
            ),
            c,
        ));
        if n.node_type == NodeType::Internal && n.vertices.is_empty() {
            for i in 0..8 {
                if let Some(ref child) = n.children[i] {
                    construct_tree_grid(Some(child.as_ref()));
                }
            }
        }
    }
}

pub fn calculate_indexes(
    tree: &Box<OctreeNode>,
    threshold: f32,
    mesh_buffers: &mut MeshBuffers,
    flat_shading: bool,
) {
    mesh_buffers.indices.clear();
    let mut tri_count = Vec::new();
    tree.process_cell(&mut mesh_buffers.indices, &mut tri_count, threshold);
    if !flat_shading {
        if mesh_buffers.indices.is_empty() {
            return;
        }
    } else {
        let mut new_positions = Vec::new();
        let mut new_normals = Vec::new();
        let mut new_colors = Vec::new();
        let mut t_index = 0;
        let mut i = 0;
        while i < mesh_buffers.indices.len() {
            let count = tri_count[t_index];
            t_index += 1;
            let n = if count == 1 {
                get_normal_q(
                    &[
                        (mesh_buffers.indices[i + 2] & 0x0FFFFFFF) as usize,
                        (mesh_buffers.indices[i + 0] & 0x0FFFFFFF) as usize,
                        (mesh_buffers.indices[i + 1] & 0x0FFFFFFF) as usize,
                    ],
                    &mesh_buffers.positions,
                )
            } else {
                get_normal_q(
                    &[
                        (mesh_buffers.indices[i + 2] & 0x0FFFFFFF) as usize,
                        (mesh_buffers.indices[i + 0] & 0x0FFFFFFF) as usize,
                        (mesh_buffers.indices[i + 1] & 0x0FFFFFFF) as usize,
                        (mesh_buffers.indices[i + 5] & 0x0FFFFFFF) as usize,
                        (mesh_buffers.indices[i + 3] & 0x0FFFFFFF) as usize,
                        (mesh_buffers.indices[i + 4] & 0x0FFFFFFF) as usize,
                    ],
                    &mesh_buffers.positions,
                )
            };
            let nc = n * 0.5 + Vec3::ONE * 0.5;
            let nc_normalized = nc.normalize();
            let c = [nc_normalized.x, nc_normalized.y, nc_normalized.z, 1.0];
            let normal = [n.x, n.y, n.z];
            let idx0 = (mesh_buffers.indices[i + 0] & 0x0FFFFFFF) as usize;
            let idx1 = (mesh_buffers.indices[i + 1] & 0x0FFFFFFF) as usize;
            let idx2 = (mesh_buffers.indices[i + 2] & 0x0FFFFFFF) as usize;
            new_positions.push(mesh_buffers.positions[idx0]);
            new_normals.push(normal);
            new_colors.push(c);
            new_positions.push(mesh_buffers.positions[idx1]);
            new_normals.push(normal);
            new_colors.push(c);
            new_positions.push(mesh_buffers.positions[idx2]);
            new_normals.push(normal);
            new_colors.push(c);
            if count > 1 {
                let idx3 = (mesh_buffers.indices[i + 3] & 0x0FFFFFFF) as usize;
                let idx4 = (mesh_buffers.indices[i + 4] & 0x0FFFFFFF) as usize;
                let idx5 = (mesh_buffers.indices[i + 5] & 0x0FFFFFFF) as usize;
                new_positions.push(mesh_buffers.positions[idx3]);
                new_normals.push(normal);
                new_colors.push(c);
                new_positions.push(mesh_buffers.positions[idx4]);
                new_normals.push(normal);
                new_colors.push(c);
                new_positions.push(mesh_buffers.positions[idx5]);
                new_normals.push(normal);
                new_colors.push(c);
                i += 3;
            }
            i += 3;
        }
        let vertex_count = new_positions.len();
        mesh_buffers.positions = new_positions;
        mesh_buffers.normals = new_normals;
        mesh_buffers.colors = new_colors;
        mesh_buffers.indices.clear();
        mesh_buffers.indices.extend(0..vertex_count as i32);
    }
}

fn get_normal_q(indexes: &[usize], vertices_buffer: &Vec<[f32; 3]>) -> Vec3 {
    let p0 = Vec3::from_array(vertices_buffer[indexes[0]]);
    let p1 = Vec3::from_array(vertices_buffer[indexes[1]]);
    let p2 = Vec3::from_array(vertices_buffer[indexes[2]]);
    let a = p2 - p1;
    let b = p2 - p0;
    let mut c = a.cross(b);
    if indexes.len() == 6 {
        let p3 = Vec3::from_array(vertices_buffer[indexes[3]]);
        let p4 = Vec3::from_array(vertices_buffer[indexes[4]]);
        let p5 = Vec3::from_array(vertices_buffer[indexes[5]]);
        let a = p5 - p4;
        let b = p5 - p3;
        let d = a.cross(b);
        if c.x.is_nan() {
            c = Vec3::ZERO;
        }
        let d = if d.x.is_nan() { Vec3::ZERO } else { d };
        c += d;
        c /= 2.0;
    }
    c = c.normalize();
    -c
}
