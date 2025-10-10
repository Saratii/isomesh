// https://github.com/Lin20/isosurface/tree/master/Isosurface/Isosurface/ManifoldDC

use glam::Vec3;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::mdc::mdc::MeshBuffers;
use crate::mdc::qef_solver::QEFSolver;
use crate::mdc::sampler::Sampler;
use crate::mdc::sampler::get_intersection;
use crate::mdc::sampler::get_normal;
use crate::mdc::tables::T_CELL_PROC_EDGE_MASK;
use crate::mdc::tables::T_CORNER_DELTAS;
use crate::mdc::tables::T_EDGE_PAIRS;
use crate::mdc::tables::T_EDGE_PROC_EDGE_MASK;
use crate::mdc::tables::T_EXTERNAL_EDGES;
use crate::mdc::tables::T_FACE_PROC_EDGE_MASK;
use crate::mdc::tables::T_FACE_PROC_FACE_MASK;
use crate::mdc::tables::T_FACES;
use crate::mdc::tables::T_INTERNAL_EDGES;
use crate::mdc::tables::T_PROCESS_EDGE_MASK;
use crate::mdc::tables::TRANSFORMED_EDGES_TABLE;
use crate::mdc::tables::TRANSFORMED_VERTICES_NUMBER_TABLE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    None,
    Internal,
    Leaf,
    Collapsed,
}

pub(crate) struct Vertex {
    pub(crate) parent: Option<Arc<Mutex<Vertex>>>,
    pub(crate) index: i32,
    pub(crate) collapsible: bool,
    pub(crate) qef: Option<QEFSolver>,
    pub(crate) normal: Vec3,
    pub(crate) surface_index: i32,
    pub(crate) error: f32,
    pub(crate) euler: i32,
    pub(crate) eis: Option<[i32; 12]>,
    pub(crate) in_cell: i32,
    pub(crate) face_prop2: bool,
}

impl Clone for Vertex {
    fn clone(&self) -> Self {
        Vertex {
            parent: self.parent.clone(),
            index: self.index,
            collapsible: self.collapsible,
            qef: self.qef.clone(),
            normal: self.normal,
            surface_index: self.surface_index,
            error: self.error,
            euler: self.euler,
            eis: self.eis,
            in_cell: self.in_cell,
            face_prop2: self.face_prop2,
        }
    }
}

impl Vertex {
    pub fn new() -> Self {
        Vertex {
            parent: None,
            index: -1,
            collapsible: true,
            qef: None,
            normal: Vec3::ZERO,
            surface_index: -1,
            error: 0.0,
            euler: 0,
            eis: None,
            in_cell: 0,
            face_prop2: false,
        }
    }
}

impl Debug for Vertex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Surface = {}, parent = {}",
            self.surface_index,
            self.parent.is_some()
        )
    }
}

pub(crate) struct OctreeNode {
    pub(crate) index: i32,
    pub(crate) position: Vec3,
    pub(crate) size: i32,
    pub(crate) children: Vec<Option<Box<OctreeNode>>>,
    pub(crate) node_type: NodeType,
    pub(crate) vertices: Vec<Arc<Mutex<Vertex>>>,
    pub(crate) corners: u8,
    pub(crate) child_index: i32,
}

static mut ENFORCE_MANIFOLD: bool = false;

impl OctreeNode {
    pub(crate) fn new() -> Self {
        OctreeNode {
            index: 0,
            position: Vec3::ZERO,
            size: 0,
            children: vec![None, None, None, None, None, None, None, None],
            node_type: NodeType::None,
            vertices: Vec::new(),
            corners: 0,
            child_index: 0,
        }
    }

    pub(crate) fn with_params(position: Vec3, size: i32, node_type: NodeType) -> Self {
        OctreeNode {
            index: 0,
            position,
            size,
            children: vec![None, None, None, None, None, None, None, None],
            node_type,
            vertices: Vec::new(),
            corners: 0,
            child_index: 0,
        }
    }

    pub(crate) fn set_enforce_manifold(enforce: bool) {
        unsafe {
            ENFORCE_MANIFOLD = enforce;
        }
    }

    pub(crate) fn get_enforce_manifold() -> bool {
        unsafe { ENFORCE_MANIFOLD }
    }

    pub(crate) fn construct_base<S>(
        &mut self,
        size: i32,
        mesh_buffers: &mut MeshBuffers,
        sampler: &S,
    ) where
        S: Sampler + Send + Sync + Clone + 'static,
    {
        self.index = 0;
        self.position = Vec3::splat(-(size as f32) * 0.5);
        self.size = size;
        self.node_type = NodeType::Internal;
        self.children = vec![None, None, None, None, None, None, None, None];
        self.vertices = Vec::new();
        self.child_index = 0;
        let mut n_index = 1;
        self.construct_nodes(mesh_buffers, &mut n_index, 4, sampler);
    }

    pub(crate) fn generate_vertex_buffer(&self, mesh_buffers: &mut MeshBuffers) {
        if self.node_type != NodeType::Leaf {
            for i in 0..8 {
                if let Some(ref child) = self.children[i] {
                    child.generate_vertex_buffer(mesh_buffers);
                }
            }
        }
        if self.vertices.is_empty() {
            return;
        }
        for i in 0..self.vertices.len() {
            let mut vertex = self.vertices[i].lock().unwrap();
            vertex.index = mesh_buffers.positions.len() as i32;
            let nc = vertex.normal * 0.5 + Vec3::ONE * 0.5;
            let nc_normalized = nc.normalize();
            let color = [nc_normalized.x, nc_normalized.y, nc_normalized.z, 1.0];

            if let Some(ref mut qef) = vertex.qef {
                let position = qef.solve(1e-6, 4, 1e-6);
                mesh_buffers
                    .positions
                    .push([position.x, position.y, position.z]);
                mesh_buffers
                    .normals
                    .push([vertex.normal.x, vertex.normal.y, vertex.normal.z]);
                mesh_buffers.colors.push(color);
            }
        }
    }

    fn construct_nodes<S>(
        &mut self,
        mesh_buffers: &mut MeshBuffers,
        n_index: &mut i32,
        threaded: i32,
        sampler: &S,
    ) -> bool
    where
        S: Sampler + Send + Sync + Clone + 'static,
    {
        if self.size == 1 {
            return self.construct_leaf(n_index, sampler);
        }
        self.node_type = NodeType::Internal;
        let child_size = self.size / 2;
        let mut has_children = false;
        if threaded > 0 && self.size > 2 {
            let mut handles = Vec::new();
            let mut return_values = vec![false; 8];
            for i in 0..8 {
                self.index = *n_index;
                *n_index += 1;
                let child_pos = T_CORNER_DELTAS[i];
                let mut child = Box::new(OctreeNode::with_params(
                    self.position + child_pos * child_size as f32,
                    child_size,
                    NodeType::Internal,
                ));
                child.child_index = i as i32;
                let sampler_clone = (*sampler).clone();
                let handle = thread::spawn(move || {
                    let mut temp = 0;
                    let mut temp_buffers = MeshBuffers {
                        positions: Vec::new(),
                        normals: Vec::new(),
                        colors: Vec::new(),
                        indices: Vec::new(),
                    };
                    let result = child.construct_nodes(
                        &mut temp_buffers,
                        &mut temp,
                        threaded - 1,
                        &sampler_clone,
                    );
                    (result, child, temp_buffers)
                });
                handles.push(handle);
            }
            for (i, handle) in handles.into_iter().enumerate() {
                let (result, child, _temp_buffers) = handle.join().unwrap();
                return_values[i] = result;
                if result {
                    self.children[i] = Some(child);
                    has_children = true;
                }
            }
        } else {
            for i in 0..8 {
                self.index = *n_index;
                *n_index += 1;
                let child_pos = T_CORNER_DELTAS[i];
                let mut child = Box::new(OctreeNode::with_params(
                    self.position + child_pos * child_size as f32,
                    child_size,
                    NodeType::Internal,
                ));
                child.child_index = i as i32;

                if child.construct_nodes(mesh_buffers, n_index, 0, sampler) {
                    has_children = true;
                    self.children[i] = Some(child);
                }
            }
        }
        has_children
    }

    fn construct_leaf<S>(&mut self, index: &mut i32, sampler: &S) -> bool
    where
        S: Sampler,
    {
        if self.size != 1 {
            return false;
        }
        self.index = *index;
        *index += 1;
        self.node_type = NodeType::Leaf;
        let mut corners = 0u8;
        let mut samples = [0.0f32; 8];
        for i in 0..8 {
            samples[i] = sampler.sample(self.position + T_CORNER_DELTAS[i]);
            if samples[i] < 0.0 {
                corners |= 1 << i;
            }
        }
        self.corners = corners;
        if corners == 0 || corners == 255 {
            return false;
        }
        let v_count = TRANSFORMED_VERTICES_NUMBER_TABLE[corners as usize];
        let mut v_edges = vec![vec![-1i32; 13]; v_count as usize];
        self.vertices = Vec::with_capacity(v_count as usize);
        let mut v_index = 0;
        let mut e_index = 0;
        for e in 0..16 {
            let code = TRANSFORMED_EDGES_TABLE[corners as usize][e];
            if code == -2 {
                v_index += 1;
                break;
            }
            if code == -1 {
                v_index += 1;
                e_index = 0;
                continue;
            }
            v_edges[v_index][e_index] = code;
            e_index += 1;
        }
        for i in 0..v_index {
            let mut k = 0;
            let mut vertex = Vertex::new();
            vertex.qef = Some(QEFSolver::new());
            let mut normal = Vec3::ZERO;
            let mut ei = [0i32; 12];
            while v_edges[i][k] != -1 {
                let edge = v_edges[i][k] as usize;
                ei[edge] = 1;
                let a = self.position
                    + T_CORNER_DELTAS[T_EDGE_PAIRS[edge][0] as usize] * self.size as f32;
                let b = self.position
                    + T_CORNER_DELTAS[T_EDGE_PAIRS[edge][1] as usize] * self.size as f32;
                let intersection = get_intersection(
                    a,
                    b,
                    samples[T_EDGE_PAIRS[edge][0] as usize],
                    samples[T_EDGE_PAIRS[edge][1] as usize],
                );
                let n = get_normal(intersection, sampler);
                normal += n;

                if let Some(ref mut qef) = vertex.qef {
                    qef.add(intersection, n);
                }
                k += 1;
            }
            normal /= k as f32;
            normal = normal.normalize();
            vertex.index = -1;
            vertex.parent = None;
            vertex.collapsible = true;
            vertex.normal = normal;
            vertex.euler = 1;
            vertex.eis = Some(ei);
            vertex.in_cell = self.child_index;
            vertex.face_prop2 = true;
            if let Some(ref mut qef) = vertex.qef {
                qef.solve(1e-6, 4, 1e-6);
                vertex.error = qef.get_error();
            }
            self.vertices.push(Arc::new(Mutex::new(vertex)));
        }
        true
    }

    pub(crate) fn process_cell(
        &self,
        indexes: &mut Vec<u32>,
        tri_count: &mut Vec<i32>,
        threshold: f32,
    ) {
        if self.node_type == NodeType::Internal {
            for i in 0..8 {
                if let Some(ref child) = self.children[i] {
                    child.process_cell(indexes, tri_count, threshold);
                }
            }
            for i in 0..12 {
                let mut face_nodes = [None, None];
                let c1 = T_EDGE_PAIRS[i][0];
                let c2 = T_EDGE_PAIRS[i][1];
                face_nodes[0] = self.children[c1 as usize].as_ref().map(|b| b.as_ref());
                face_nodes[1] = self.children[c2 as usize].as_ref().map(|b| b.as_ref());
                Self::process_face(
                    &face_nodes,
                    T_EDGE_PAIRS[i][2] as i32,
                    indexes,
                    tri_count,
                    threshold,
                );
            }
            for i in 0..6 {
                let edge_nodes = [
                    self.children[T_CELL_PROC_EDGE_MASK[i][0] as usize]
                        .as_ref()
                        .map(|b| b.as_ref()),
                    self.children[T_CELL_PROC_EDGE_MASK[i][1] as usize]
                        .as_ref()
                        .map(|b| b.as_ref()),
                    self.children[T_CELL_PROC_EDGE_MASK[i][2] as usize]
                        .as_ref()
                        .map(|b| b.as_ref()),
                    self.children[T_CELL_PROC_EDGE_MASK[i][3] as usize]
                        .as_ref()
                        .map(|b| b.as_ref()),
                ];
                Self::process_edge(
                    &edge_nodes,
                    T_CELL_PROC_EDGE_MASK[i][4] as i32,
                    indexes,
                    tri_count,
                    threshold,
                );
            }
        }
    }

    fn process_face(
        nodes: &[Option<&OctreeNode>; 2],
        direction: i32,
        indexes: &mut Vec<u32>,
        tri_count: &mut Vec<i32>,
        threshold: f32,
    ) {
        if nodes[0].is_none() || nodes[1].is_none() {
            return;
        }
        let node0 = nodes[0].unwrap();
        let node1 = nodes[1].unwrap();
        if node0.node_type != NodeType::Leaf || node1.node_type != NodeType::Leaf {
            for i in 0..4 {
                let mut face_nodes = [None, None];
                for j in 0..2 {
                    if let Some(node) = nodes[j] {
                        if node.node_type == NodeType::Leaf {
                            face_nodes[j] = Some(node);
                        } else {
                            let idx = T_FACE_PROC_FACE_MASK[direction as usize][i][j];
                            face_nodes[j] =
                                node.children[idx as usize].as_ref().map(|b| b.as_ref());
                        }
                    }
                }
                Self::process_face(
                    &face_nodes,
                    T_FACE_PROC_FACE_MASK[direction as usize][i][2] as i32,
                    indexes,
                    tri_count,
                    threshold,
                );
            }
            let orders = [[0, 0, 1, 1], [0, 1, 0, 1]];
            for i in 0..4 {
                let mut edge_nodes = [None, None, None, None];

                for j in 0..4 {
                    let order_idx = T_FACE_PROC_EDGE_MASK[direction as usize][i][0];
                    if let Some(node) = nodes[orders[order_idx as usize][j]] {
                        if node.node_type == NodeType::Leaf {
                            edge_nodes[j] = Some(node);
                        } else {
                            let idx = T_FACE_PROC_EDGE_MASK[direction as usize][i][1 + j];
                            edge_nodes[j] =
                                node.children[idx as usize].as_ref().map(|b| b.as_ref());
                        }
                    }
                }
                Self::process_edge(
                    &edge_nodes,
                    T_FACE_PROC_EDGE_MASK[direction as usize][i][5] as i32,
                    indexes,
                    tri_count,
                    threshold,
                );
            }
        }
    }

    fn process_edge(
        nodes: &[Option<&OctreeNode>; 4],
        direction: i32,
        indexes: &mut Vec<u32>,
        tri_count: &mut Vec<i32>,
        threshold: f32,
    ) {
        if nodes[0].is_none() || nodes[1].is_none() || nodes[2].is_none() || nodes[3].is_none() {
            return;
        }
        if nodes[0].unwrap().node_type == NodeType::Leaf
            && nodes[1].unwrap().node_type == NodeType::Leaf
            && nodes[2].unwrap().node_type == NodeType::Leaf
            && nodes[3].unwrap().node_type == NodeType::Leaf
        {
            Self::process_indexes(nodes, direction, indexes, tri_count, threshold);
        } else {
            for i in 0..2 {
                let mut edge_nodes = [None, None, None, None];
                for j in 0..4 {
                    if let Some(node) = nodes[j] {
                        if node.node_type == NodeType::Leaf {
                            edge_nodes[j] = Some(node);
                        } else {
                            let idx = T_EDGE_PROC_EDGE_MASK[direction as usize][i][j];
                            edge_nodes[j] =
                                node.children[idx as usize].as_ref().map(|b| b.as_ref());
                        }
                    }
                }
                Self::process_edge(
                    &edge_nodes,
                    T_EDGE_PROC_EDGE_MASK[direction as usize][i][4] as i32,
                    indexes,
                    tri_count,
                    threshold,
                );
            }
        }
    }

    fn process_indexes(
        nodes: &[Option<&OctreeNode>; 4],
        direction: i32,
        indexes: &mut Vec<u32>,
        tri_count: &mut Vec<i32>,
        threshold: f32,
    ) {
        let mut min_size = 10000000;
        let mut indices = [-1i32; 4];
        let mut flip = false;
        let mut sign_changed = false;
        for i in 0..4 {
            if let Some(node) = nodes[i] {
                let edge = T_PROCESS_EDGE_MASK[direction as usize][i];
                let c1 = T_EDGE_PAIRS[edge as usize][0];
                let c2 = T_EDGE_PAIRS[edge as usize][1];
                let m1 = (node.corners >> c1) & 1;
                let m2 = (node.corners >> c2) & 1;
                if node.size < min_size {
                    min_size = node.size;
                    flip = m1 == 1;
                    sign_changed = (m1 == 0 && m2 != 0) || (m1 != 0 && m2 == 0);
                }
                let mut index = 0;
                let mut skip = false;
                for k in 0..16 {
                    let e = TRANSFORMED_EDGES_TABLE[node.corners as usize][k];
                    if e == -1 {
                        index += 1;
                        continue;
                    }
                    if e == -2 {
                        skip = true;
                        break;
                    }
                    if e == edge as i32 {
                        break;
                    }
                }
                if skip {
                    continue;
                }
                if index >= node.vertices.len() {
                    return;
                }
                let v = node.vertices[index].lock().unwrap();
                let mut highest = (*v).clone();
                loop {
                    let parent_arc = match &highest.parent {
                        Some(arc) => Arc::clone(arc),
                        None => break,
                    };

                    let parent = parent_arc.lock().unwrap();
                    if parent.error <= threshold
                        && (!Self::get_enforce_manifold()
                            || (parent.euler == 1 && parent.face_prop2))
                    {
                        highest = (*parent).clone();
                    } else {
                        break;
                    }
                }
                indices[i] = highest.index;
            }
        }
        if sign_changed {
            let mut count = 0;
            if !flip {
                if indices[0] != -1
                    && indices[1] != -1
                    && indices[2] != -1
                    && indices[0] != indices[1]
                    && indices[1] != indices[3]
                {
                    indexes.push((indices[0] & 0x0FFFFFFF) as u32);
                    indexes.push((indices[1] & 0x0FFFFFFF) as u32);
                    indexes.push((indices[3] & 0x0FFFFFFF) as u32);
                    count += 1;
                }
                if indices[0] != -1
                    && indices[2] != -1
                    && indices[3] != -1
                    && indices[0] != indices[2]
                    && indices[2] != indices[3]
                {
                    indexes.push((indices[0] & 0x0FFFFFFF) as u32);
                    indexes.push((indices[3] & 0x0FFFFFFF) as u32);
                    indexes.push((indices[2] & 0x0FFFFFFF) as u32);
                    count += 1;
                }
            } else {
                if indices[0] != -1
                    && indices[3] != -1
                    && indices[1] != -1
                    && indices[0] != indices[1]
                    && indices[1] != indices[3]
                {
                    indexes.push((indices[0] & 0x0FFFFFFF) as u32);
                    indexes.push((indices[3] & 0x0FFFFFFF) as u32);
                    indexes.push((indices[1] & 0x0FFFFFFF) as u32);
                    count += 1;
                }
                if indices[0] != -1
                    && indices[2] != -1
                    && indices[3] != -1
                    && indices[0] != indices[2]
                    && indices[2] != indices[3]
                {
                    indexes.push((indices[0] & 0x0FFFFFFF) as u32);
                    indexes.push((indices[2] & 0x0FFFFFFF) as u32);
                    indexes.push((indices[3] & 0x0FFFFFFF) as u32);
                    count += 1;
                }
            }
            if count > 0 {
                tri_count.push(count);
            }
        }
    }

    pub(crate) fn cluster_cell_base(&mut self, error: f32) {
        if self.node_type != NodeType::Internal {
            return;
        }
        for i in 0..8 {
            if let Some(ref mut child) = self.children[i] {
                child.cluster_cell(error);
            }
        }
    }

    fn cluster_cell(&mut self, error: f32) {
        if self.node_type != NodeType::Internal {
            return;
        }
        let mut signs = [-1i32; 8];
        let mut mid_sign = -1i32;
        let mut _is_collapsible = true;
        for i in 0..8 {
            if let Some(ref mut child) = self.children[i] {
                child.cluster_cell(error);
                if child.node_type == NodeType::Internal {
                    _is_collapsible = false;
                } else {
                    mid_sign = ((child.corners >> (7 - i)) & 1) as i32;
                    signs[i] = ((child.corners >> i) & 1) as i32;
                }
            }
        }
        self.corners = 0;
        for i in 0..8 {
            if signs[i] == -1 {
                self.corners |= (mid_sign << i) as u8;
            } else {
                self.corners |= (signs[i] << i) as u8;
            }
        }
        let mut surface_index = 0;
        let mut collected_vertices: Vec<Arc<Mutex<Vertex>>> = Vec::new();
        let mut new_vertices: Vec<Arc<Mutex<Vertex>>> = Vec::new();
        for i in 0..12 {
            let mut face_nodes = [None, None];
            let c1 = T_EDGE_PAIRS[i][0];
            let c2 = T_EDGE_PAIRS[i][1];
            face_nodes[0] = self.children[c1 as usize].as_ref().map(|b| b.as_ref());
            face_nodes[1] = self.children[c2 as usize].as_ref().map(|b| b.as_ref());

            Self::cluster_face(
                &face_nodes,
                T_EDGE_PAIRS[i][2] as i32,
                &mut surface_index,
                &mut collected_vertices,
            );
        }
        for i in 0..6 {
            let edge_nodes = [
                self.children[T_CELL_PROC_EDGE_MASK[i][0] as usize]
                    .as_ref()
                    .map(|b| b.as_ref()),
                self.children[T_CELL_PROC_EDGE_MASK[i][1] as usize]
                    .as_ref()
                    .map(|b| b.as_ref()),
                self.children[T_CELL_PROC_EDGE_MASK[i][2] as usize]
                    .as_ref()
                    .map(|b| b.as_ref()),
                self.children[T_CELL_PROC_EDGE_MASK[i][3] as usize]
                    .as_ref()
                    .map(|b| b.as_ref()),
            ];
            Self::cluster_edge(
                &edge_nodes,
                T_CELL_PROC_EDGE_MASK[i][4] as i32,
                &mut surface_index,
                &mut collected_vertices,
            );
        }
        let mut highest_index = surface_index;
        if highest_index == -1 {
            highest_index = 0;
        }
        for child_opt in &self.children {
            if let Some(child) = child_opt {
                for v_arc in &child.vertices {
                    let mut v = v_arc.lock().unwrap();
                    if v.surface_index == -1 {
                        v.surface_index = highest_index;
                        highest_index += 1;
                        drop(v);
                        collected_vertices.push(Arc::clone(v_arc));
                    }
                }
            }
        }
        let mut _clustered_count = 0;
        if !collected_vertices.is_empty() {
            for i in 0..=highest_index {
                let mut qef = QEFSolver::new();
                let mut normal = Vec3::ZERO;
                let mut count = 0;
                let mut edges = [0i32; 12];
                let mut euler = 0;
                let mut e = 0;
                for v_arc in &collected_vertices {
                    let v = v_arc.lock().unwrap();
                    if v.surface_index == i {
                        if let Some(ref eis) = v.eis {
                            for k in 0..3 {
                                let edge = T_EXTERNAL_EDGES[v.in_cell as usize][k];
                                edges[edge as usize] += eis[edge as usize];
                            }
                            for k in 0..9 {
                                let edge = T_INTERNAL_EDGES[v.in_cell as usize][k];
                                e += eis[edge as usize];
                            }
                        }
                        euler += v.euler;
                        if let Some(ref v_qef) = v.qef {
                            qef.add_data(&v_qef.data);
                        }
                        normal += v.normal;
                        count += 1;
                    }
                }
                if count == 0 {
                    continue;
                }
                let mut face_prop2 = true;
                for f in 0..6 {
                    let mut intersections = 0;
                    for ei in 0..4 {
                        intersections += edges[T_FACES[f][ei] as usize];
                    }
                    if intersections != 0 && intersections != 2 {
                        face_prop2 = false;
                        break;
                    }
                }
                let mut new_vertex = Vertex::new();
                normal /= count as f32;
                normal = normal.normalize();
                new_vertex.normal = normal;
                new_vertex.qef = Some(qef);
                new_vertex.eis = Some(edges);
                new_vertex.euler = euler - e / 4;
                new_vertex.in_cell = self.child_index;
                new_vertex.face_prop2 = face_prop2;
                if let Some(ref mut qef) = new_vertex.qef {
                    qef.solve(1e-6, 4, 1e-6);
                    let err = qef.get_error();
                    new_vertex.collapsible = err <= error;
                    new_vertex.error = err;
                }
                _clustered_count += 1;
                let new_vertex_arc = Arc::new(Mutex::new(new_vertex));
                for v_arc in &collected_vertices {
                    let mut v = v_arc.lock().unwrap();
                    if v.surface_index == i {
                        if !Arc::ptr_eq(v_arc, &new_vertex_arc) {
                            v.parent = Some(Arc::clone(&new_vertex_arc));
                        } else {
                            v.parent = None;
                        }
                    }
                }
                new_vertices.push(new_vertex_arc);
            }
        } else {
            return;
        }
        for v_arc in &collected_vertices {
            let mut v = v_arc.lock().unwrap();
            v.surface_index = -1;
        }
        self.vertices = new_vertices;
    }

    fn cluster_face(
        nodes: &[Option<&OctreeNode>; 2],
        direction: i32,
        surface_index: &mut i32,
        collected_vertices: &mut Vec<Arc<Mutex<Vertex>>>,
    ) {
        if nodes[0].is_none() || nodes[1].is_none() {
            return;
        }
        let node0 = nodes[0].unwrap();
        let node1 = nodes[1].unwrap();
        if node0.node_type != NodeType::Leaf || node1.node_type != NodeType::Leaf {
            for i in 0..4 {
                let mut face_nodes = [None, None];
                for j in 0..2 {
                    if let Some(node) = nodes[j] {
                        if node.node_type != NodeType::Internal {
                            face_nodes[j] = Some(node);
                        } else {
                            let idx = T_FACE_PROC_FACE_MASK[direction as usize][i][j];
                            face_nodes[j] =
                                node.children[idx as usize].as_ref().map(|b| b.as_ref());
                        }
                    }
                }
                Self::cluster_face(
                    &face_nodes,
                    T_FACE_PROC_FACE_MASK[direction as usize][i][2] as i32,
                    surface_index,
                    collected_vertices,
                );
            }
        }
        let orders = [[0, 0, 1, 1], [0, 1, 0, 1]];
        for i in 0..4 {
            let mut edge_nodes = [None, None, None, None];
            for j in 0..4 {
                let order_idx = T_FACE_PROC_EDGE_MASK[direction as usize][i][0];
                let node_idx = orders[order_idx as usize][j];
                if nodes[node_idx].is_none() {
                    continue;
                }
                let node = nodes[node_idx].unwrap();
                if node.node_type != NodeType::Internal {
                    edge_nodes[j] = Some(node);
                } else {
                    let idx = T_FACE_PROC_EDGE_MASK[direction as usize][i][1 + j];
                    edge_nodes[j] = node.children[idx as usize].as_ref().map(|b| b.as_ref());
                }
            }
            Self::cluster_edge(
                &edge_nodes,
                T_FACE_PROC_EDGE_MASK[direction as usize][i][5] as i32,
                surface_index,
                collected_vertices,
            );
        }
    }

    fn cluster_edge(
        nodes: &[Option<&OctreeNode>; 4],
        direction: i32,
        surface_index: &mut i32,
        collected_vertices: &mut Vec<Arc<Mutex<Vertex>>>,
    ) {
        let all_not_internal = nodes
            .iter()
            .all(|node| node.map_or(true, |n| n.node_type != NodeType::Internal));

        if all_not_internal {
            Self::cluster_indexes(nodes, direction, surface_index, collected_vertices);
        } else {
            for i in 0..2 {
                let mut edge_nodes = [None, None, None, None];

                for j in 0..4 {
                    if let Some(node) = nodes[j] {
                        if node.node_type == NodeType::Leaf {
                            edge_nodes[j] = Some(node);
                        } else {
                            let idx = T_EDGE_PROC_EDGE_MASK[direction as usize][i][j];
                            edge_nodes[j] =
                                node.children[idx as usize].as_ref().map(|b| b.as_ref());
                        }
                    }
                }
                Self::cluster_edge(
                    &edge_nodes,
                    T_EDGE_PROC_EDGE_MASK[direction as usize][i][4] as i32,
                    surface_index,
                    collected_vertices,
                );
            }
        }
    }

    fn cluster_indexes(
        nodes: &[Option<&OctreeNode>; 4],
        direction: i32,
        max_surface_index: &mut i32,
        collected_vertices: &mut Vec<Arc<Mutex<Vertex>>>,
    ) {
        if nodes.iter().all(|n| n.is_none()) {
            return;
        }
        let mut vertices: [Option<Arc<Mutex<Vertex>>>; 4] = [None, None, None, None];
        let mut v_count = 0;
        let mut _node_count = 0;
        for i in 0..4 {
            if let Some(node) = nodes[i] {
                _node_count += 1;
                let edge = T_PROCESS_EDGE_MASK[direction as usize][i];
                let c1 = T_EDGE_PAIRS[edge as usize][0];
                let c2 = T_EDGE_PAIRS[edge as usize][1];
                let m1 = (node.corners >> c1) & 1;
                let m2 = (node.corners >> c2) & 1;
                let mut index = 0;
                let mut skip = false;
                for k in 0..16 {
                    let e = TRANSFORMED_EDGES_TABLE[node.corners as usize][k];
                    if e == -1 {
                        index += 1;
                        continue;
                    }
                    if e == -2 {
                        if !((m1 == 0 && m2 != 0) || (m1 != 0 && m2 == 0)) {
                            skip = true;
                        }
                        break;
                    }
                    if e == edge as i32 {
                        break;
                    }
                }
                if !skip && index < node.vertices.len() {
                    let mut vertex_arc = Arc::clone(&node.vertices[index]);
                    loop {
                        let has_parent = {
                            let v = vertex_arc.lock().unwrap();
                            v.parent.is_some()
                        };
                        if has_parent {
                            let parent_arc = {
                                let v = vertex_arc.lock().unwrap();
                                Arc::clone(v.parent.as_ref().unwrap())
                            };
                            vertex_arc = parent_arc;
                        } else {
                            break;
                        }
                    }
                    vertices[i] = Some(vertex_arc);
                    v_count += 1;
                }
            }
        }
        if v_count == 0 {
            return;
        }
        let mut surface_index = -1i32;
        for i in 0..4 {
            if let Some(ref v_arc) = vertices[i] {
                let (current_surface_index, needs_reassignment) = {
                    let v = v_arc.lock().unwrap();
                    let current = v.surface_index;
                    let needs = current != -1 && surface_index != -1 && surface_index != current;
                    (current, needs)
                };
                if needs_reassignment {
                    Self::assign_surface(collected_vertices, current_surface_index, surface_index);
                } else if current_surface_index != -1 && surface_index == -1 {
                    surface_index = current_surface_index;
                }
            }
        }
        if surface_index == -1 {
            surface_index = *max_surface_index;
            *max_surface_index += 1;
        }
        for i in 0..4 {
            if let Some(ref v_arc) = vertices[i] {
                let should_collect = {
                    let v = v_arc.lock().unwrap();
                    v.surface_index == -1
                };
                if should_collect {
                    collected_vertices.push(Arc::clone(v_arc));
                }
                {
                    let mut v = v_arc.lock().unwrap();
                    v.surface_index = surface_index;
                }
            }
        }
    }

    fn assign_surface(vertices: &mut Vec<Arc<Mutex<Vertex>>>, from: i32, to: i32) {
        for v_arc in vertices {
            let mut v = v_arc.lock().unwrap();
            if v.surface_index == from {
                v.surface_index = to;
            }
        }
    }
}

impl Debug for OctreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Index = {}, size = {}", self.index, self.size)
    }
}
