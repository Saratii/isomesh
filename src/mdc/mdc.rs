// Manifold Dual Contouring
// https://github.com/Lin20/isosurface/tree/master/Isosurface/Isosurface/ManifoldDC

use std::sync::{Arc, atomic::Ordering};

use glam::Vec3;

use crate::mdc::{
    octree::{ENFORCE_MANIFOLD, OctreeNode},
    sampler::Sampler,
};

pub struct MeshBuffers {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl MeshBuffers {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for MeshBuffers {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
        }
    }
}

pub fn mdc_mesh_generation<S>(
    threshold: f32,
    mesh_buffers: &mut MeshBuffers,
    flat_shading: bool,
    resolution: i32,
    enforce_manifold: bool,
    sampler: S,
) where
    S: Sampler + Send + Sync + 'static,
{
    ENFORCE_MANIFOLD.store(enforce_manifold, Ordering::Relaxed);
    let mut tree = Box::new(OctreeNode::new());
    tree.construct_base(resolution, mesh_buffers, Arc::new(sampler));
    tree.cluster_cell_base(0.0);
    tree.generate_vertex_buffer(mesh_buffers);
    calculate_indexes(&tree, threshold, mesh_buffers, flat_shading);
}

pub(crate) fn calculate_indexes(
    tree: &Box<OctreeNode>,
    threshold: f32,
    mesh_buffers: &mut MeshBuffers,
    flat_shading: bool,
) {
    mesh_buffers.indices.clear();
    let enforce_manifold = ENFORCE_MANIFOLD.load(Ordering::Relaxed);
    let mut tri_count = Vec::new();
    tree.process_cell(
        &mut mesh_buffers.indices,
        &mut tri_count,
        threshold,
        enforce_manifold,
    );
    if !flat_shading {
        if mesh_buffers.indices.is_empty() {
            return;
        }
    } else {
        let mut new_positions = Vec::with_capacity(mesh_buffers.indices.len());
        let mut new_normals = Vec::with_capacity(mesh_buffers.indices.len());
        let mut new_colors = Vec::with_capacity(mesh_buffers.indices.len());
        let mut t_index = 0;
        let mut i = 0;
        while i < mesh_buffers.indices.len() {
            let count = tri_count[t_index];
            t_index += 1;
            let idx0 = (mesh_buffers.indices[i + 0] & 0x0FFFFFFF) as usize;
            let idx1 = (mesh_buffers.indices[i + 1] & 0x0FFFFFFF) as usize;
            let idx2 = (mesh_buffers.indices[i + 2] & 0x0FFFFFFF) as usize;
            new_positions.push(mesh_buffers.positions[idx0]);
            new_positions.push(mesh_buffers.positions[idx1]);
            new_positions.push(mesh_buffers.positions[idx2]);
            if count == 1 {
                let n = get_normal_q(&[idx2, idx0, idx1], &mesh_buffers.positions);
                let nc = n * 0.5 + Vec3::ONE * 0.5;
                let c = [nc.x, nc.y, nc.z, 1.0];
                let normal = [n.x, n.y, n.z];
                new_normals.push(normal);
                new_colors.push(c);
                new_normals.push(normal);
                new_colors.push(c);
                new_normals.push(normal);
                new_colors.push(c);
                i += 3;
            } else {
                let idx3 = (mesh_buffers.indices[i + 3] & 0x0FFFFFFF) as usize;
                let idx4 = (mesh_buffers.indices[i + 4] & 0x0FFFFFFF) as usize;
                let idx5 = (mesh_buffers.indices[i + 5] & 0x0FFFFFFF) as usize;
                let n = get_normal_q(
                    &[idx2, idx0, idx1, idx5, idx3, idx4],
                    &mesh_buffers.positions,
                );
                let nc = n * 0.5 + Vec3::ONE * 0.5;
                let c = [nc.x, nc.y, nc.z, 1.0];
                let normal = [n.x, n.y, n.z];
                new_normals.push(normal);
                new_colors.push(c);
                new_normals.push(normal);
                new_colors.push(c);
                new_normals.push(normal);
                new_colors.push(c);
                new_positions.push(mesh_buffers.positions[idx3]);
                new_normals.push(normal);
                new_colors.push(c);
                new_positions.push(mesh_buffers.positions[idx4]);
                new_normals.push(normal);
                new_colors.push(c);
                new_positions.push(mesh_buffers.positions[idx5]);
                new_normals.push(normal);
                new_colors.push(c);
                i += 6;
            }
        }
        let vertex_count = new_positions.len();
        mesh_buffers.positions = new_positions;
        mesh_buffers.normals = new_normals;
        mesh_buffers.colors = new_colors;
        mesh_buffers.indices = (0..vertex_count as u32).collect();
    }
}

fn get_normal_q(indexes: &[usize], vertices_buffer: &[[f32; 3]]) -> Vec3 {
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

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::mdc::{
        mdc::{MeshBuffers, mdc_mesh_generation},
        sampler::SphereSampler,
        test_data::{EXPECTED_COLORS, EXPECTED_INDICES, EXPECTED_NORMALS, EXPECTED_POSITIONS},
    };

    #[test]
    fn test_mdc_sphere_no_flat_shading() {
        let resolution = 16;
        let mut mesh_buffers = MeshBuffers::new();
        let sphere = SphereSampler::new(Vec3::new(0.0, 0.0, 0.0), 5.0);
        mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, sphere);
        // Test positions
        assert_eq!(mesh_buffers.positions.len(), EXPECTED_POSITIONS.len());
        for (i, expected_pos) in EXPECTED_POSITIONS.iter().enumerate() {
            assert_eq!(mesh_buffers.positions[i], *expected_pos);
        }
        // Test normals
        assert_eq!(mesh_buffers.normals.len(), EXPECTED_NORMALS.len());
        for (i, expected_normal) in EXPECTED_NORMALS.iter().enumerate() {
            assert_eq!(mesh_buffers.normals[i], *expected_normal);
        }
        // Test colors
        assert_eq!(mesh_buffers.colors.len(), EXPECTED_COLORS.len());
        for (i, expected_color) in EXPECTED_COLORS.iter().enumerate() {
            assert_eq!(mesh_buffers.colors[i], *expected_color);
        }
        // Test indices
        assert_eq!(mesh_buffers.indices.len(), EXPECTED_INDICES.len());
        for (i, expected_idx) in EXPECTED_INDICES.iter().enumerate() {
            assert_eq!(mesh_buffers.indices[i], *expected_idx);
        }
    }
}
