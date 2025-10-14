// Rust version of MdcOctreeNode (from Java Manifold Dual Contouring by John Lin20)
// Equivalent to: manifoldDC.MdcOctreeNode

use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OctreeNodeType {
    NodeInternal,
    NodeLeaf,
}

pub struct MeshVertex {
    pub pos: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
}

impl MeshVertex {
    pub fn new(pos: Vec3, normal: Vec3, color: Vec3) -> Self {
        MeshVertex { pos, normal, color }
    }
}
