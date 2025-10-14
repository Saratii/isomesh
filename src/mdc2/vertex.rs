// Rust version of MdcVertex (from Java Manifold Dual Contouring by John Lin20)
// Equivalent to: manifoldDC.MdcVertex

use glam::Vec3;

use crate::mdc::qef_solver::QEFData;

#[derive(Clone)]
pub(crate) struct MdcVertex {
    pub(crate) parent: Option<Box<MdcVertex>>,
    pub(crate) index: i32,
    pub(crate) collapsible: bool,
    pub(crate) qef: Option<QEFData>,
    pub(crate) pos: Vec3,
    pub(crate) normal: Vec3,
    pub(crate) surface_index: i32,
    pub(crate) error: f32,
    pub(crate) euler: i32,
    pub(crate) eis: Option<Vec<i32>>,
    pub(crate) in_cell: i32,
    pub(crate) face_prop2: bool,
    pub(crate) debug_flag: bool,
}

impl Default for MdcVertex {
    fn default() -> Self {
        Self {
            parent: None,
            index: -1,
            collapsible: true,
            qef: None,
            pos: Vec3::ZERO,
            normal: Vec3::ZERO,
            surface_index: -1,
            error: 0.0,
            euler: 0,
            eis: None,
            in_cell: 0,
            face_prop2: false,
            debug_flag: false,
        }
    }
}

impl MdcVertex {
    pub fn new() -> Self {
        Self::default()
    }
}
