// Manifold Dual Contouring
// https://github.com/Lin20/isosurface/tree/master/Isosurface/Isosurface/ManifoldDC

use glam::Vec3;
use std::time::Instant;

use crate::{
    octree::{NodeType, OctreeNode, VertexPositionColorNormalNormal},
    tables::TRANSFORMED_EDGES_TABLE,
};

pub struct MDC {
    pub name: String,
    pub flat_shading: bool,
    pub vertices_dn: Vec<VertexPositionColorNormalNormal>,
    pub tree: Option<Box<OctreeNode>>,
    pub enforce_manifold: bool,
    pub resolution: i32,
    pub size: i32,
    pub indices: Vec<i32>,
    pub vertex_count: usize,
    pub index_count: usize,
    pub wireframe_count: usize,
    pub wireframe_vertex_count: usize,
    pub outline_location: usize,
}

impl MDC {
    pub const FLAT_SHADING: bool = true;

    pub fn new(resolution: i32, size: i32) -> Self {
        for i in 0..256 {
            let mut found = [false; 16];
            for k in 0..16 {
                let edge = TRANSFORMED_EDGES_TABLE[i][k];
                if edge < 0 {
                    continue;
                }
                found[edge as usize] = true;
            }
        }

        let enforce_manifold = true;
        OctreeNode::set_enforce_manifold(enforce_manifold);
        MDC {
            name: "Manifold Dual Contouring".to_string(),
            flat_shading: Self::FLAT_SHADING,
            vertices_dn: Vec::new(),
            tree: None,
            enforce_manifold,
            resolution,
            size,
            indices: Vec::new(),
            vertex_count: 0,
            index_count: 0,
            wireframe_count: 0,
            wireframe_vertex_count: 0,
            outline_location: 0,
        }
    }

    pub fn set_enforce_manifold(&mut self, enforce: bool) {
        self.enforce_manifold = enforce;
        OctreeNode::set_enforce_manifold(enforce);
    }

    pub fn get_extra_information(&self) -> String {
        format!("Manifold: {}", self.enforce_manifold)
    }

    pub fn contour(&mut self, threshold: f32) -> u128 {
        let start = Instant::now();
        if self.tree.is_none() {
            self.vertices_dn.clear();
            let mut tree = Box::new(OctreeNode::new());
            let mut vs = Vec::new();
            tree.construct_base(self.resolution, 0.0, &mut vs);
            tree.cluster_cell_base(0.0);
            tree.generate_vertex_buffer(&mut self.vertices_dn);
            self.vertex_count = self.vertices_dn.len();
            self.tree = Some(tree);
        }
        self.outline_location = 0;
        self.calculate_indexes(threshold);
        let elapsed = start.elapsed();
        elapsed.as_millis()
    }

    pub fn construct_tree_grid(&mut self, node: Option<&OctreeNode>) {
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
            self.outline_location += 24;
            if n.node_type == NodeType::Internal && n.vertices.is_empty() {
                for i in 0..8 {
                    if let Some(ref child) = n.children[i] {
                        self.construct_tree_grid(Some(child.as_ref()));
                    }
                }
            }
        }
    }

    pub fn calculate_indexes(&mut self, threshold: f32) {
        if !Self::FLAT_SHADING {
            self.indices.clear();
        } else {
            self.indices = Vec::new();
        }
        let mut tri_count = Vec::new();
        if let Some(ref tree) = self.tree {
            tree.process_cell(&mut self.indices, &mut tri_count, threshold);
        }
        if !Self::FLAT_SHADING {
            self.index_count = self.indices.len();
            if self.indices.is_empty() {
                return;
            }
            for i in 0..self.indices.len() {
                self.indices[i] = self.indices[i] & 0x0FFFFFFF;
            }
        } else {
            let mut new_vertices = Vec::new();
            let mut t_index = 0;
            let mut i = 0;
            while i < self.indices.len() {
                let count = tri_count[t_index];
                t_index += 1;
                let n = if count == 1 {
                    self.get_normal_q(&[
                        (self.indices[i + 2] & 0x0FFFFFFF) as usize,
                        (self.indices[i + 0] & 0x0FFFFFFF) as usize,
                        (self.indices[i + 1] & 0x0FFFFFFF) as usize,
                    ])
                } else {
                    self.get_normal_q(&[
                        (self.indices[i + 2] & 0x0FFFFFFF) as usize,
                        (self.indices[i + 0] & 0x0FFFFFFF) as usize,
                        (self.indices[i + 1] & 0x0FFFFFFF) as usize,
                        (self.indices[i + 5] & 0x0FFFFFFF) as usize,
                        (self.indices[i + 3] & 0x0FFFFFFF) as usize,
                        (self.indices[i + 4] & 0x0FFFFFFF) as usize,
                    ])
                };
                let nc = n * 0.5 + Vec3::ONE * 0.5;
                let nc_normalized = nc.normalize();
                let c = [nc_normalized.x, nc_normalized.y, nc_normalized.z, 1.0];
                let idx0 = (self.indices[i + 0] & 0x0FFFFFFF) as usize;
                let idx1 = (self.indices[i + 1] & 0x0FFFFFFF) as usize;
                let idx2 = (self.indices[i + 2] & 0x0FFFFFFF) as usize;
                let v0 = VertexPositionColorNormalNormal::new(
                    self.vertices_dn[idx0].position,
                    c,
                    n,
                    self.vertices_dn[idx0].normal,
                );
                let v1 = VertexPositionColorNormalNormal::new(
                    self.vertices_dn[idx1].position,
                    c,
                    n,
                    self.vertices_dn[idx1].normal,
                );
                let v2 = VertexPositionColorNormalNormal::new(
                    self.vertices_dn[idx2].position,
                    c,
                    n,
                    self.vertices_dn[idx2].normal,
                );
                new_vertices.push(v0);
                new_vertices.push(v1);
                new_vertices.push(v2);
                if count > 1 {
                    let idx3 = (self.indices[i + 3] & 0x0FFFFFFF) as usize;
                    let idx4 = (self.indices[i + 4] & 0x0FFFFFFF) as usize;
                    let idx5 = (self.indices[i + 5] & 0x0FFFFFFF) as usize;
                    let v3 = VertexPositionColorNormalNormal::new(
                        self.vertices_dn[idx3].position,
                        c,
                        n,
                        self.vertices_dn[idx3].normal,
                    );
                    let v4 = VertexPositionColorNormalNormal::new(
                        self.vertices_dn[idx4].position,
                        c,
                        n,
                        self.vertices_dn[idx4].normal,
                    );
                    let v5 = VertexPositionColorNormalNormal::new(
                        self.vertices_dn[idx5].position,
                        c,
                        n,
                        self.vertices_dn[idx5].normal,
                    );
                    new_vertices.push(v3);
                    new_vertices.push(v4);
                    new_vertices.push(v5);
                    i += 3;
                }
                i += 3;
            }
            self.vertices_dn = new_vertices;
            self.vertex_count = self.vertices_dn.len();
        }
    }

    fn get_normal_q(&self, indexes: &[usize]) -> Vec3 {
        let a = self.vertices_dn[indexes[2]].position - self.vertices_dn[indexes[1]].position;
        let b = self.vertices_dn[indexes[2]].position - self.vertices_dn[indexes[0]].position;
        let mut c = a.cross(b);
        if indexes.len() == 6 {
            let a = self.vertices_dn[indexes[5]].position - self.vertices_dn[indexes[4]].position;
            let b = self.vertices_dn[indexes[5]].position - self.vertices_dn[indexes[3]].position;
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
}
