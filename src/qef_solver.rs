// https://github.com/Lin20/isosurface/tree/master/Isosurface/Isosurface/ManifoldDC

use glam::Vec3;

use crate::{smat3::SMat3, svd};

#[derive(Debug, Clone)]
pub struct QEFData {
    pub ata_00: f32,
    pub ata_01: f32,
    pub ata_02: f32,
    pub ata_11: f32,
    pub ata_12: f32,
    pub ata_22: f32,
    pub atb_x: f32,
    pub atb_y: f32,
    pub atb_z: f32,
    pub btb: f32,
    pub mass_point_x: f32,
    pub mass_point_y: f32,
    pub mass_point_z: f32,
    pub num_points: i32,
}

impl QEFData {
    pub fn new() -> Self {
        Self {
            ata_00: 0.0,
            ata_01: 0.0,
            ata_02: 0.0,
            ata_11: 0.0,
            ata_12: 0.0,
            ata_22: 0.0,
            atb_x: 0.0,
            atb_y: 0.0,
            atb_z: 0.0,
            btb: 0.0,
            mass_point_x: 0.0,
            mass_point_y: 0.0,
            mass_point_z: 0.0,
            num_points: 0,
        }
    }

    pub fn with_values(
        ata_00: f32,
        ata_01: f32,
        ata_02: f32,
        ata_11: f32,
        ata_12: f32,
        ata_22: f32,
        atb_x: f32,
        atb_y: f32,
        atb_z: f32,
        btb: f32,
        mass_point_x: f32,
        mass_point_y: f32,
        mass_point_z: f32,
        num_points: i32,
    ) -> Self {
        Self {
            ata_00,
            ata_01,
            ata_02,
            ata_11,
            ata_12,
            ata_22,
            atb_x,
            atb_y,
            atb_z,
            btb,
            mass_point_x,
            mass_point_y,
            mass_point_z,
            num_points,
        }
    }

    pub fn add(&mut self, rhs: &QEFData) {
        self.ata_00 += rhs.ata_00;
        self.ata_01 += rhs.ata_01;
        self.ata_02 += rhs.ata_02;
        self.ata_11 += rhs.ata_11;
        self.ata_12 += rhs.ata_12;
        self.ata_22 += rhs.ata_22;
        self.atb_x += rhs.atb_x;
        self.atb_y += rhs.atb_y;
        self.atb_z += rhs.atb_z;
        self.btb += rhs.btb;
        self.mass_point_x += rhs.mass_point_x;
        self.mass_point_y += rhs.mass_point_y;
        self.mass_point_z += rhs.mass_point_z;
        self.num_points += rhs.num_points;
    }

    pub fn clear(&mut self) {
        self.set(
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0,
        );
    }

    pub fn set(
        &mut self,
        ata_00: f32,
        ata_01: f32,
        ata_02: f32,
        ata_11: f32,
        ata_12: f32,
        ata_22: f32,
        atb_x: f32,
        atb_y: f32,
        atb_z: f32,
        btb: f32,
        mass_point_x: f32,
        mass_point_y: f32,
        mass_point_z: f32,
        num_points: i32,
    ) {
        self.ata_00 = ata_00;
        self.ata_01 = ata_01;
        self.ata_02 = ata_02;
        self.ata_11 = ata_11;
        self.ata_12 = ata_12;
        self.ata_22 = ata_22;
        self.atb_x = atb_x;
        self.atb_y = atb_y;
        self.atb_z = atb_z;
        self.btb = btb;
        self.mass_point_x = mass_point_x;
        self.mass_point_y = mass_point_y;
        self.mass_point_z = mass_point_z;
        self.num_points = num_points;
    }

    pub fn set_from(&mut self, rhs: &QEFData) {
        self.set(
            rhs.ata_00,
            rhs.ata_01,
            rhs.ata_02,
            rhs.ata_11,
            rhs.ata_12,
            rhs.ata_22,
            rhs.atb_x,
            rhs.atb_y,
            rhs.atb_z,
            rhs.btb,
            rhs.mass_point_x,
            rhs.mass_point_y,
            rhs.mass_point_z,
            rhs.num_points,
        );
    }
}

impl Default for QEFData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct QEFSolver {
    pub data: QEFData,
    pub ata: SMat3,
    pub atb: Vec3,
    pub x: Vec3,
    pub mass_point: Vec3,
    pub has_solution: bool,
    pub last_error: f32,
}

impl QEFSolver {
    pub fn new() -> Self {
        Self {
            data: QEFData::new(),
            ata: SMat3::new(),
            atb: Vec3::ZERO,
            x: Vec3::ZERO,
            mass_point: Vec3::ZERO,
            has_solution: false,
            last_error: 0.0,
        }
    }

    pub fn add(&mut self, p: Vec3, n: Vec3) {
        self.has_solution = false;
        let n = n.normalize();
        self.data.ata_00 += n.x * n.x;
        self.data.ata_01 += n.x * n.y;
        self.data.ata_02 += n.x * n.z;
        self.data.ata_11 += n.y * n.y;
        self.data.ata_12 += n.y * n.z;
        self.data.ata_22 += n.z * n.z;
        let dot = n.x * p.x + n.y * p.y + n.z * p.z;
        self.data.atb_x += dot * n.x;
        self.data.atb_y += dot * n.y;
        self.data.atb_z += dot * n.z;
        self.data.btb += dot * dot;
        self.data.mass_point_x += p.x;
        self.data.mass_point_y += p.y;
        self.data.mass_point_z += p.z;
        self.data.num_points += 1;
    }

    pub fn add_data(&mut self, rhs: &QEFData) {
        self.has_solution = false;
        self.data.add(rhs);
    }

    pub fn get_error(&mut self) -> f32 {
        self.get_error_at(self.x)
    }

    pub fn get_error_at(&mut self, pos: Vec3) -> f32 {
        if !self.has_solution {
            self.set_ata();
            self.set_atb();
        }
        let atax = self.ata.vmul(pos);
        self.last_error = pos.dot(atax) - 2.0 * pos.dot(self.atb) + self.data.btb;

        if self.last_error.is_nan() {
            self.last_error = 10000.0;
        }
        // debug_assert!(
        //     self.last_error >= -1e-2,
        //     "QEF error is negative ({}) — possible instability.",
        //     self.last_error
        // );
        self.last_error
    }

    pub fn solve(&mut self, svd_tol: f32, svd_sweeps: i32, pinv_tol: f32) -> Vec3 {
        if self.data.num_points == 0 {
            panic!("QEFSolver: no points to solve");
        }
        self.mass_point = Vec3::new(
            self.data.mass_point_x,
            self.data.mass_point_y,
            self.data.mass_point_z,
        );
        self.mass_point /= self.data.num_points as f32;
        self.set_ata();
        self.set_atb();
        let tmpv = self.ata.vmul(self.mass_point);
        self.atb = self.atb - tmpv;
        self.x = Vec3::ZERO;
        let result = svd::solve_symmetric(
            &self.ata,
            &self.atb,
            &mut self.x,
            svd_tol,
            svd_sweeps,
            pinv_tol,
        );
        if result.is_nan() {
            self.x = self.mass_point;
        } else {
            self.x += self.mass_point;
        }
        self.last_error = result;
        debug_assert!(result >= 0.0);
        self.set_atb();
        self.has_solution = true;
        self.x
    }

    fn set_ata(&mut self) {
        self.ata.set_symmetric(
            self.data.ata_00,
            self.data.ata_01,
            self.data.ata_02,
            self.data.ata_11,
            self.data.ata_12,
            self.data.ata_22,
        );
    }

    fn set_atb(&mut self) {
        self.atb = Vec3::new(self.data.atb_x, self.data.atb_y, self.data.atb_z);
    }
}

impl Default for QEFSolver {
    fn default() -> Self {
        Self::new()
    }
}
