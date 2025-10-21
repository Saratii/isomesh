//This is a work in progress. It more closely resembles MDC from the original paper but is slow and doesn't produce correct vertices.

use crate::{
    manifold_dual_contouring::{
        sampler::Sampler,
        tables::{
            T_CELL_PROC_EDGE_MASK, T_CORNER_DELTAS, T_EDGE_PAIRS, T_EDGE_PROC_EDGE_MASK,
            T_EXTERNAL_EDGES, T_FACE_PROC_EDGE_MASK, T_FACE_PROC_FACE_MASK, T_FACES,
            T_INTERNAL_EDGES, T_PROCESS_EDGE_MASK, TRANSFORMED_EDGES_TABLE,
        },
    },
    manifold_dual_contouring_2::{
        octree::{MeshVertex, OctreeNodeType},
        qef::QefData,
        solver::LevenQefSolver,
    },
};
use glam::{IVec3, Vec3};

pub fn mdc_mesh_generation<S: Sampler>(sampler: S, resolution: i32) -> (Vec<MeshVertex>, Vec<i32>) {
    let half_res = resolution / 2;
    let mut root = build_octree(
        IVec3::new(-half_res, -half_res, -half_res),
        resolution,
        sampler,
    );
    cluster_cell_base(&mut root, 0.5);
    let mut vertices = Vec::new();
    generate_vertex_buffer(&mut root, &mut vertices);
    let mut indexes = Vec::new();
    let mut tri_count = Vec::new();
    process_cell(&root, &mut indexes, &mut tri_count, 0.5, true);
    (vertices, indexes)
}

pub(crate) fn generate_vertex_buffer(node: &mut MdcOctreeNode, vertices: &mut Vec<MeshVertex>) {
    if node.node_type != OctreeNodeType::NodeLeaf {
        for i in 0..8 {
            if let Some(child) = &mut node.children[i] {
                generate_vertex_buffer(child, vertices);
            }
        }
    }
    if node.vertices.is_empty() {
        return;
    }
    for (_, vertex) in node.vertices.iter_mut().enumerate() {
        if vertex.is_none() {
            continue;
        }
        let v = vertex.as_mut().unwrap();
        v.index = vertices.len() as i32;
        let mut nc = v.normal * 0.5 + Vec3::new(1.0, 1.0, 1.0) * 0.5;
        nc = nc.normalize();
        let color = nc;
        let pos_vec4 = v.qef.as_mut().unwrap().solve();
        let pos = Vec3::new(pos_vec4.x, pos_vec4.y, pos_vec4.z);
        let mesh_vertex = MeshVertex::new(pos, v.normal, color);
        vertices.push(mesh_vertex);
    }
}

pub(crate) fn build_octree<S: Sampler>(min: IVec3, size: i32, sampler: S) -> MdcOctreeNode {
    let mut root = MdcOctreeNode::new(min, size, OctreeNodeType::NodeInternal, 0);
    let mut n_index = 1;
    construct_nodes(&mut root, &mut n_index, &sampler);
    root
}

pub(crate) fn construct_nodes<S: Sampler>(
    node: &mut MdcOctreeNode,
    n_index: &mut i32,
    sampler: &S,
) {
    if node.size == 1 {
        construct_leaf(node, n_index, sampler);
        return;
    }
    let child_size = node.size / 2;
    for i in 0..8 {
        node.index = *n_index;
        *n_index += 1;
        let child_min = (node.min.as_vec3() + T_CORNER_DELTAS[i] * child_size as f32).as_ivec3();
        let mut child = MdcOctreeNode::new(
            child_min,
            child_size,
            OctreeNodeType::NodeInternal,
            i as i32,
        );
        construct_nodes(&mut child, n_index, sampler);
        if !child.vertices.is_empty() || !child.children.iter().all(|c| c.is_none()) {
            node.children[i] = Some(Box::new(child));
        }
    }
}

pub(crate) fn construct_leaf<S: Sampler>(leaf: &mut MdcOctreeNode, n_index: &mut i32, sampler: &S) {
    if leaf.size != 1 {
        return;
    }
    leaf.index = *n_index;
    *n_index += 1;
    leaf.node_type = OctreeNodeType::NodeLeaf;
    let mut corners = 0;
    let mut samples = [0.0; 8];
    for i in 0..8 {
        let vertex_pos = leaf.min.as_vec3() + T_CORNER_DELTAS[i];
        let sample = sampler.sample(vertex_pos);
        samples[i] = sample;
        if sample < 0.0 {
            corners |= 1 << i;
        }
    }
    leaf.corners = corners;
    if corners == 0 || corners == 255 {
        return;
    }
    let mut v_edges: Vec<Vec<i32>> = vec![vec![-1; 13]];
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
            v_edges.push(vec![-1; 13]);
            continue;
        }
        v_edges[v_index][e_index] = code;
        e_index += 1;
    }
    leaf.vertices = vec![None; v_index];
    for i in 0..v_index {
        let mut mdc_vertex = MdcVertex::new();
        mdc_vertex.qef = Some(QefData::new(LevenQefSolver::new()));
        let mut normal = Vec3::ZERO;
        let mut ei = [0; 12];
        let mut k = 0;
        while v_edges[i][k] != -1 {
            let edge_code = v_edges[i][k] as usize;
            ei[edge_code] = 1;
            let edge_pair = T_EDGE_PAIRS[edge_code];
            let a_iv =
                leaf.min.as_vec3() + T_CORNER_DELTAS[edge_pair[0] as usize] * leaf.size as f32;
            let b_iv =
                leaf.min.as_vec3() + T_CORNER_DELTAS[edge_pair[1] as usize] * leaf.size as f32;
            let intersect = get_intersection(
                &a_iv,
                &b_iv,
                samples[edge_pair[0] as usize],
                samples[edge_pair[1] as usize],
            );
            let n = calculate_surface_normal(&intersect, sampler);
            normal = normal + n;
            mdc_vertex
                .qef
                .as_mut()
                .unwrap()
                .qef_add_point3(intersect, n);
            k += 1;
        }
        normal = normal / k as f32;
        normal = normal.normalize();
        mdc_vertex.index = 0;
        mdc_vertex.parent = None;
        mdc_vertex.collapsible = true;
        mdc_vertex.normal = normal;
        mdc_vertex.euler = 1;
        mdc_vertex.eis = ei;
        mdc_vertex.in_cell = leaf.child_index;
        mdc_vertex.face_prop2 = true;
        let solved = mdc_vertex.qef.as_mut().unwrap().solve();
        mdc_vertex.pos = Vec3::new(solved.x, solved.y, solved.z);
        mdc_vertex.error = mdc_vertex.qef.as_mut().unwrap().get_error();
        leaf.vertices[i] = Some(Box::new(mdc_vertex));
    }
}

pub fn get_intersection(p1: &Vec3, p2: &Vec3, d1: f32, d2: f32) -> Vec3 {
    *p1 + (*p2 - *p1) * (-d1) / (d2 - d1)
}

pub fn calculate_surface_normal<S>(p: &Vec3, sampler: &S) -> Vec3
where
    S: Sampler,
{
    let h = 0.001;
    let x_offset = Vec3::new(h, 0.0, 0.0);
    let y_offset = Vec3::new(0.0, h, 0.0);
    let z_offset = Vec3::new(0.0, 0.0, h);
    let dx = sampler.sample(*p + x_offset) - sampler.sample(*p - x_offset);
    let dy = sampler.sample(*p + y_offset) - sampler.sample(*p - y_offset);
    let dz = sampler.sample(*p + z_offset) - sampler.sample(*p - z_offset);
    let mut v = Vec3::new(dx, dy, dz);
    v = v.normalize();
    v
}

pub(crate) fn process_cell(
    node: &MdcOctreeNode,
    indexes: &mut Vec<i32>,
    tri_count: &mut Vec<i32>,
    threshold: f32,
    enforce_manifold: bool,
) {
    if node.node_type == OctreeNodeType::NodeInternal {
        for i in 0..8 {
            if let Some(child) = &node.children[i] {
                process_cell(child, indexes, tri_count, threshold, enforce_manifold);
            }
        }
        for i in 0..12 {
            let face_nodes = [
                node.children[T_EDGE_PAIRS[i][0] as usize].as_ref(),
                node.children[T_EDGE_PAIRS[i][1] as usize].as_ref(),
            ];
            process_face(
                &face_nodes,
                T_EDGE_PAIRS[i][2] as usize,
                indexes,
                tri_count,
                threshold,
                enforce_manifold,
            );
        }
        for i in 0..6 {
            let edge_nodes = [
                node.children[T_CELL_PROC_EDGE_MASK[i][0] as usize].as_ref(),
                node.children[T_CELL_PROC_EDGE_MASK[i][1] as usize].as_ref(),
                node.children[T_CELL_PROC_EDGE_MASK[i][2] as usize].as_ref(),
                node.children[T_CELL_PROC_EDGE_MASK[i][3] as usize].as_ref(),
            ];
            process_edge(
                &edge_nodes,
                T_CELL_PROC_EDGE_MASK[i][4] as usize,
                indexes,
                tri_count,
                threshold,
                enforce_manifold,
            );
        }
    }
}

pub(crate) fn process_face(
    nodes: &[Option<&Box<MdcOctreeNode>>; 2],
    direction: usize,
    indexes: &mut Vec<i32>,
    tri_count: &mut Vec<i32>,
    threshold: f32,
    enforce_manifold: bool,
) {
    if nodes[0].is_none() || nodes[1].is_none() {
        return;
    }
    let n0 = nodes[0].unwrap();
    let n1 = nodes[1].unwrap();
    if n0.node_type != OctreeNodeType::NodeLeaf || n1.node_type != OctreeNodeType::NodeLeaf {
        for i in 0..4 {
            let mut face_nodes: [Option<&Box<MdcOctreeNode>>; 2] = [None, None];
            for j in 0..2 {
                if nodes[j].is_none() {
                    continue;
                }
                let n = nodes[j].unwrap();
                if n.node_type == OctreeNodeType::NodeLeaf {
                    face_nodes[j] = nodes[j];
                } else {
                    face_nodes[j] =
                        n.children[T_FACE_PROC_FACE_MASK[direction][i][j] as usize].as_ref();
                }
            }
            process_face(
                &face_nodes,
                T_FACE_PROC_FACE_MASK[direction][i][2] as usize,
                indexes,
                tri_count,
                threshold,
                enforce_manifold,
            );
        }
        let orders = [[0, 0, 1, 1], [0, 1, 0, 1]];
        for i in 0..4 {
            let mut edge_nodes = [None, None, None, None];
            for j in 0..4 {
                let order_idx = T_FACE_PROC_EDGE_MASK[direction][i][0];
                let node_idx = orders[order_idx as usize][j];
                if nodes[node_idx].is_none() {
                    continue;
                }
                let n = nodes[node_idx].unwrap();
                if n.node_type == OctreeNodeType::NodeLeaf {
                    edge_nodes[j] = Some(n);
                } else {
                    edge_nodes[j] =
                        n.children[T_FACE_PROC_EDGE_MASK[direction][i][1 + j] as usize].as_ref();
                }
            }
            process_edge(
                &edge_nodes,
                T_FACE_PROC_EDGE_MASK[direction][i][5] as usize,
                indexes,
                tri_count,
                threshold,
                enforce_manifold,
            );
        }
    }
}

pub(crate) fn cluster_cell_base(node: &mut MdcOctreeNode, error: f32) {
    if node.node_type != OctreeNodeType::NodeInternal {
        return;
    }
    for i in 0..8 {
        if let Some(child) = &mut node.children[i] {
            cluster_cell(child, error);
        }
    }
}
pub(crate) fn cluster_cell(node: &mut MdcOctreeNode, error: f32) {
    use std::collections::HashMap;
    if node.node_type != OctreeNodeType::NodeInternal {
        return;
    }
    let mut signs = [-1; 8];
    let mut mid_sign = -1;
    for i in 0..8 {
        if node.children[i].is_none() {
            continue;
        }
        if let Some(child) = &mut node.children[i] {
            cluster_cell(child, error);
            if child.node_type != OctreeNodeType::NodeInternal {
                mid_sign = ((child.corners >> (7 - i)) & 1) as i32;
                signs[i] = ((child.corners >> i) & 1) as i32;
            }
        }
    }
    node.corners = 0;
    for i in 0..8 {
        if signs[i] == -1 {
            node.corners |= (mid_sign << i) as u32;
        } else {
            node.corners |= (signs[i] << i) as u32;
        }
    }
    let mut surface_index = 0;
    let mut collected_vertices: Vec<Box<MdcVertex>> = Vec::new();
    for i in 0..12 {
        let c1 = T_EDGE_PAIRS[i][0];
        let c2 = T_EDGE_PAIRS[i][1];
        let face_nodes = [
            node.children[c1 as usize].as_ref(),
            node.children[c2 as usize].as_ref(),
        ];
        cluster_face(
            &face_nodes,
            T_EDGE_PAIRS[i][2] as usize,
            &mut surface_index,
            &mut collected_vertices,
        );
    }
    for i in 0..6 {
        let edge_nodes = [
            node.children[T_CELL_PROC_EDGE_MASK[i][0] as usize].as_ref(),
            node.children[T_CELL_PROC_EDGE_MASK[i][1] as usize].as_ref(),
            node.children[T_CELL_PROC_EDGE_MASK[i][2] as usize].as_ref(),
            node.children[T_CELL_PROC_EDGE_MASK[i][3] as usize].as_ref(),
        ];
        cluster_edge(
            &edge_nodes,
            T_CELL_PROC_EDGE_MASK[i][4] as usize,
            &mut surface_index,
            &mut collected_vertices,
        );
    }
    let mut highest_index = surface_index;
    if highest_index == -1 {
        highest_index = 0;
    }
    for child_opt in &mut node.children {
        if let Some(child) = child_opt {
            for v_opt in &mut child.vertices {
                if let Some(v) = v_opt {
                    if v.surface_index == -1 {
                        v.surface_index = highest_index;
                        highest_index += 1;
                        collected_vertices.push(v.clone());
                    }
                }
            }
        }
    }
    if collected_vertices.is_empty() {
        return;
    }
    let mut surface_vertices: HashMap<i32, Vec<Box<MdcVertex>>> = HashMap::new();
    for v in &collected_vertices {
        surface_vertices
            .entry(v.surface_index)
            .or_insert_with(Vec::new)
            .push(v.clone());
    }
    let mut new_vertices: Vec<Box<MdcVertex>> = Vec::new();
    for (_, verts_for_surface) in surface_vertices.iter() {
        if verts_for_surface.is_empty() {
            continue;
        }
        let mut qef = QefData::new(LevenQefSolver::new());
        let mut normal = Vec3::ZERO;
        let mut count = 0;
        let mut edges = [0; 12];
        let mut euler = 0;
        let mut e = 0;
        for v in verts_for_surface {
            for k in 0..3 {
                let edge_idx = T_EXTERNAL_EDGES[v.in_cell as usize][k];
                edges[edge_idx as usize] += v.eis[edge_idx as usize];
            }
            for k in 0..9 {
                let edge_idx = T_INTERNAL_EDGES[v.in_cell as usize][k];
                e += v.eis[edge_idx as usize];
            }
            euler += v.euler;
            if let Some(qef_solver) = &v.qef {
                qef.add(&qef_solver);
            }
            normal = normal + v.normal;
            count += 1;
        }
        if count == 0 {
            continue;
        }
        let mut face_prop2 = true;
        for f in 0..6 {
            if !face_prop2 {
                break;
            }
            let mut intersections = 0;
            for ei_idx in 0..4 {
                intersections += edges[T_FACES[f][ei_idx] as usize];
            }
            if !(intersections == 0 || intersections == 2) {
                face_prop2 = false;
            }
        }
        let mut new_vertex = MdcVertex::new();
        normal = normal / count as f32;
        normal = normal.normalize();
        new_vertex.normal = normal;
        new_vertex.qef = Some(qef);
        new_vertex.eis = edges;
        new_vertex.euler = euler - e / 4;
        new_vertex.in_cell = node.child_index;
        new_vertex.face_prop2 = face_prop2;
        if let Some(qef) = &mut new_vertex.qef {
            let pos_vec4 = qef.solve();
            new_vertex.pos = Vec3::new(pos_vec4.x, pos_vec4.y, pos_vec4.z);
            let err = qef.get_error();
            new_vertex.collapsible = err <= error;
            new_vertex.error = err;
        }
        new_vertices.push(Box::new(new_vertex));
    }
    node.vertices = new_vertices.iter().map(|v| Some(v.clone())).collect();
}

pub(crate) fn cluster_face(
    nodes: &[Option<&Box<MdcOctreeNode>>; 2],
    direction: usize,
    int_holder: &mut i32,
    collected_vertices: &mut Vec<Box<MdcVertex>>,
) {
    if nodes[0].is_none() || nodes[1].is_none() {
        return;
    }
    let n0 = nodes[0].unwrap();
    let n1 = nodes[1].unwrap();
    if n0.node_type != OctreeNodeType::NodeLeaf || n1.node_type != OctreeNodeType::NodeLeaf {
        for i in 0..4 {
            let mut face_nodes = [None, None];
            for j in 0..2 {
                if nodes[j].is_none() {
                    continue;
                }
                let n = nodes[j].unwrap();
                if n.node_type != OctreeNodeType::NodeInternal {
                    face_nodes[j] = Some(n);
                } else {
                    face_nodes[j] =
                        n.children[T_FACE_PROC_FACE_MASK[direction][i][j] as usize].as_ref();
                }
            }
            cluster_face(
                &face_nodes,
                T_FACE_PROC_FACE_MASK[direction][i][2] as usize,
                int_holder,
                collected_vertices,
            );
        }
    }
    let orders = [[0, 0, 1, 1], [0, 1, 0, 1]];
    for i in 0..4 {
        let mut edge_nodes = [None, None, None, None];
        for j in 0..4 {
            let order_idx = T_FACE_PROC_EDGE_MASK[direction][i][0];
            let node_idx = orders[order_idx as usize][j];
            if nodes[node_idx].is_none() {
                continue;
            }
            let n = nodes[node_idx].unwrap();
            if n.node_type != OctreeNodeType::NodeInternal {
                edge_nodes[j] = Some(n);
            } else {
                edge_nodes[j] =
                    n.children[T_FACE_PROC_EDGE_MASK[direction][i][1 + j] as usize].as_ref();
            }
        }
        cluster_edge(
            &edge_nodes,
            T_FACE_PROC_EDGE_MASK[direction][i][5] as usize,
            int_holder,
            collected_vertices,
        );
    }
}
pub(crate) fn cluster_edge(
    nodes: &[Option<&Box<MdcOctreeNode>>; 4],
    direction: usize,
    int_holder: &mut i32,
    collected_vertices: &mut Vec<Box<MdcVertex>>,
) {
    let all_non_internal = (nodes[0].is_none()
        || nodes[0].unwrap().node_type != OctreeNodeType::NodeInternal)
        && (nodes[1].is_none() || nodes[1].unwrap().node_type != OctreeNodeType::NodeInternal)
        && (nodes[2].is_none() || nodes[2].unwrap().node_type != OctreeNodeType::NodeInternal)
        && (nodes[3].is_none() || nodes[3].unwrap().node_type != OctreeNodeType::NodeInternal);
    if all_non_internal {
        cluster_indexes(nodes, direction, int_holder, collected_vertices);
    } else {
        for i in 0..2 {
            let mut edge_nodes = [None, None, None, None];
            for j in 0..4 {
                if nodes[j].is_none() {
                    continue;
                }
                let n = nodes[j].unwrap();
                if n.node_type == OctreeNodeType::NodeLeaf {
                    edge_nodes[j] = Some(n);
                } else {
                    edge_nodes[j] =
                        n.children[T_EDGE_PROC_EDGE_MASK[direction][i][j] as usize].as_ref();
                }
            }
            cluster_edge(
                &edge_nodes,
                T_EDGE_PROC_EDGE_MASK[direction][i][4] as usize,
                int_holder,
                collected_vertices,
            );
        }
    }
}

pub(crate) fn cluster_indexes(
    nodes: &[Option<&Box<MdcOctreeNode>>; 4],
    direction: usize,
    int_holder: &mut i32,
    collected_vertices: &mut Vec<Box<MdcVertex>>,
) {
    if nodes[0].is_none() && nodes[1].is_none() && nodes[2].is_none() && nodes[3].is_none() {
        return;
    }
    let mut vertices: [Option<&Box<MdcVertex>>; 4] = [None, None, None, None];
    let mut v_count = 0;
    for i in 0..4 {
        if nodes[i].is_none() {
            continue;
        }
        let n = nodes[i].unwrap();
        let edge = T_PROCESS_EDGE_MASK[direction][i];
        let c1 = T_EDGE_PAIRS[edge as usize][0];
        let c2 = T_EDGE_PAIRS[edge as usize][1];
        let m1 = ((n.corners >> c1) & 1) as i32;
        let m2 = ((n.corners >> c2) & 1) as i32;
        let mut index = 0;
        let mut skip = false;
        for k in 0..16 {
            let e = TRANSFORMED_EDGES_TABLE[n.corners as usize][k];
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
        if !skip && index < n.vertices.len() {
            if let Some(v_opt) = &n.vertices[index] {
                let mut v = v_opt.as_ref();
                while let Some(parent) = &v.parent {
                    v = parent.as_ref();
                }
                vertices[i] = Some(v_opt);
                v_count += 1;
            }
        }
    }
    if v_count == 0 {
        return;
    }
    let mut surface_index = -1;
    for i in 0..4 {
        if let Some(v) = vertices[i] {
            if v.surface_index != -1 {
                if surface_index != -1 && surface_index != v.surface_index {
                    assign_surface(collected_vertices, v.surface_index, surface_index);
                } else if surface_index == -1 {
                    surface_index = v.surface_index;
                }
            }
        }
    }
    if surface_index == -1 {
        surface_index = *int_holder;
        *int_holder += 1;
    }
    for i in 0..4 {
        if let Some(v) = vertices[i] {
            if v.surface_index == -1 {
                collected_vertices.push(v.clone());
            }
        }
    }
    for v in collected_vertices.iter_mut() {
        v.surface_index = surface_index;
    }
}

fn assign_surface(vertices: &mut Vec<Box<MdcVertex>>, from: i32, to: i32) {
    for v in vertices.iter_mut() {
        if v.surface_index == from {
            v.surface_index = to;
        }
    }
}

pub(crate) struct MdcOctreeNode {
    pub(crate) min: IVec3,
    pub(crate) size: i32,
    pub(crate) node_type: OctreeNodeType,
    pub(crate) index: i32,
    pub(crate) child_index: i32,
    pub(crate) corners: u32,
    pub(crate) children: [Option<Box<MdcOctreeNode>>; 8],
    pub(crate) vertices: Vec<Option<Box<MdcVertex>>>,
}
impl MdcOctreeNode {
    pub(crate) fn new(min: IVec3, size: i32, node_type: OctreeNodeType, child_index: i32) -> Self {
        MdcOctreeNode {
            min,
            size,
            node_type,
            index: 0,
            child_index,
            corners: 0,
            children: [None, None, None, None, None, None, None, None],
            vertices: Vec::new(),
        }
    }
}
pub(crate) struct MdcVertex {
    pub(crate) parent: Option<Box<MdcVertex>>,
    pub(crate) index: i32,
    pub(crate) collapsible: bool,
    pub(crate) qef: Option<QefData>,
    pub(crate) pos: Vec3,
    pub(crate) normal: Vec3,
    pub(crate) surface_index: i32,
    pub(crate) error: f32,
    pub(crate) euler: i32,
    pub(crate) eis: [i32; 12],
    pub(crate) in_cell: i32,
    pub(crate) face_prop2: bool,
    pub(crate) debug_flag: bool,
}
impl MdcVertex {
    pub(crate) fn new() -> Self {
        MdcVertex {
            qef: None,
            normal: Vec3::ZERO,
            pos: Vec3::ZERO,
            error: 0.0,
            index: 0,
            parent: None,
            collapsible: false,
            euler: 0,
            eis: [0; 12],
            in_cell: 0,
            face_prop2: false,
            surface_index: -1,
            debug_flag: false,
        }
    }
}
impl Clone for MdcVertex {
    fn clone(&self) -> Self {
        MdcVertex {
            qef: self.qef.clone(),
            normal: self.normal,
            pos: self.pos,
            error: self.error,
            index: self.index,
            parent: self.parent.as_ref().map(|p| p.clone()),
            collapsible: self.collapsible,
            euler: self.euler,
            eis: self.eis.clone(),
            in_cell: self.in_cell,
            face_prop2: self.face_prop2,
            surface_index: self.surface_index,
            debug_flag: self.debug_flag,
        }
    }
}
impl PartialEq for MdcVertex {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.pos == other.pos
    }
}

pub(crate) fn process_edge(
    nodes: &[Option<&Box<MdcOctreeNode>>; 4],
    direction: usize,
    indexes: &mut Vec<i32>,
    tri_count: &mut Vec<i32>,
    threshold: f32,
    enforce_manifold: bool,
) {
    if nodes[0].is_none() || nodes[1].is_none() || nodes[2].is_none() || nodes[3].is_none() {
        return;
    }
    if nodes[0].unwrap().node_type == OctreeNodeType::NodeLeaf
        && nodes[1].unwrap().node_type == OctreeNodeType::NodeLeaf
        && nodes[2].unwrap().node_type == OctreeNodeType::NodeLeaf
        && nodes[3].unwrap().node_type == OctreeNodeType::NodeLeaf
    {
        process_indexes(
            nodes,
            direction,
            indexes,
            tri_count,
            threshold,
            enforce_manifold,
        );
    } else {
        for i in 0..2 {
            let mut edge_nodes = [None, None, None, None];
            for j in 0..4 {
                if nodes[j].is_none() {
                    continue;
                }
                let n = nodes[j].unwrap();
                if n.node_type == OctreeNodeType::NodeLeaf {
                    edge_nodes[j] = Some(n);
                } else {
                    edge_nodes[j] =
                        n.children[T_EDGE_PROC_EDGE_MASK[direction][i][j] as usize].as_ref();
                }
            }
            process_edge(
                &edge_nodes,
                T_EDGE_PROC_EDGE_MASK[direction][i][4] as usize,
                indexes,
                tri_count,
                threshold,
                enforce_manifold,
            );
        }
    }
}
pub(crate) fn process_indexes(
    nodes: &[Option<&Box<MdcOctreeNode>>; 4],
    direction: usize,
    indexes: &mut Vec<i32>,
    tri_count: &mut Vec<i32>,
    threshold: f32,
    enforce_manifold: bool,
) {
    let mut min_size = 10000000;
    let mut indices = [-1, -1, -1, -1];
    let mut flip = false;
    let mut sign_changed = false;
    for i in 0..4 {
        if nodes[i].is_none() {
            continue;
        }
        let n = nodes[i].unwrap();
        let edge = T_PROCESS_EDGE_MASK[direction][i];
        let c1 = T_EDGE_PAIRS[edge as usize][0];
        let c2 = T_EDGE_PAIRS[edge as usize][1];
        let m1 = ((n.corners >> c1) & 1) as i32;
        let m2 = ((n.corners >> c2) & 1) as i32;
        if n.size < min_size {
            min_size = n.size;
            flip = m1 == 1;
            sign_changed = (m1 == 0 && m2 != 0) || (m1 != 0 && m2 == 0);
        }
        let mut index = 0;
        let mut skip = false;
        for k in 0..16 {
            let e = TRANSFORMED_EDGES_TABLE[n.corners as usize][k];
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
        if index >= n.vertices.len() {
            return;
        }
        if let Some(v_opt) = &n.vertices[index] {
            let mut v = v_opt.as_ref();
            let mut highest = v;
            while let Some(parent) = &highest.parent {
                if parent.error <= threshold
                    && (!enforce_manifold || (parent.euler == 1 && parent.face_prop2))
                {
                    highest = parent.as_ref();
                    v = parent.as_ref();
                } else {
                    highest = parent.as_ref();
                }
            }
            indices[i] = v.index;
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
                indexes.push(indices[0]);
                indexes.push(indices[1]);
                indexes.push(indices[3]);
                count += 1;
            }
            if indices[0] != -1
                && indices[2] != -1
                && indices[3] != -1
                && indices[0] != indices[2]
                && indices[2] != indices[3]
            {
                indexes.push(indices[0]);
                indexes.push(indices[3]);
                indexes.push(indices[2]);
                count += 1;
            }
        } else {
            if indices[0] != -1
                && indices[3] != -1
                && indices[1] != -1
                && indices[0] != indices[1]
                && indices[1] != indices[3]
            {
                indexes.push(0x10000000 | indices[0]);
                indexes.push(0x10000000 | indices[3]);
                indexes.push(0x10000000 | indices[1]);
                count += 1;
            }
            if indices[0] != -1
                && indices[2] != -1
                && indices[3] != -1
                && indices[0] != indices[2]
                && indices[2] != indices[3]
            {
                indexes.push(0x10000000 | indices[0]);
                indexes.push(0x10000000 | indices[2]);
                indexes.push(0x10000000 | indices[3]);
                count += 1;
            }
        }
        if count > 0 {
            tri_count.push(count);
        }
    }
}
