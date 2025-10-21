// https://github.com/Lin20/isosurface/tree/master/Isosurface/Isosurface/ManifoldDC

use glam::Vec3;

use crate::mdc::{smat3::SMat3, svd::solve_symmetric};

#[derive(Clone)]
pub(crate) struct QEFSolver {
    pub(crate) ata: SMat3,
    pub(crate) atb_x: f32,
    pub(crate) atb_y: f32,
    pub(crate) atb_z: f32,
    pub(crate) btb: f32,
    pub(crate) mass_point_x: f32,
    pub(crate) mass_point_y: f32,
    pub(crate) mass_point_z: f32,
    pub(crate) num_points: i32,
    pub(crate) atb: Vec3,
    pub(crate) x: Vec3,
    pub(crate) mass_point: Vec3,
    pub(crate) has_solution: bool,
    pub(crate) last_error: f32,
}

impl QEFSolver {
    pub(crate) fn new() -> Self {
        Self {
            ata: SMat3::ZERO,
            atb_x: 0.0,
            atb_y: 0.0,
            atb_z: 0.0,
            btb: 0.0,
            mass_point_x: 0.0,
            mass_point_y: 0.0,
            mass_point_z: 0.0,
            num_points: 0,
            atb: Vec3::ZERO,
            x: Vec3::ZERO,
            mass_point: Vec3::ZERO,
            has_solution: false,
            last_error: 0.0,
        }
    }

    pub(crate) fn add(&mut self, p: Vec3, n: Vec3) {
        self.has_solution = false;
        let n = n.normalize();
        self.ata.m00 += n.x * n.x;
        self.ata.m01 += n.x * n.y;
        self.ata.m02 += n.x * n.z;
        self.ata.m11 += n.y * n.y;
        self.ata.m12 += n.y * n.z;
        self.ata.m22 += n.z * n.z;
        let dot = n.x * p.x + n.y * p.y + n.z * p.z;
        self.atb_x += dot * n.x;
        self.atb_y += dot * n.y;
        self.atb_z += dot * n.z;
        self.btb += dot * dot;
        self.mass_point_x += p.x;
        self.mass_point_y += p.y;
        self.mass_point_z += p.z;
        self.num_points += 1;
    }

    pub(crate) fn add_qef(&mut self, rhs: &QEFSolver) {
        self.ata.add(&rhs.ata);
        self.atb_x += rhs.atb_x;
        self.atb_y += rhs.atb_y;
        self.atb_z += rhs.atb_z;
        self.btb += rhs.btb;
        self.mass_point_x += rhs.mass_point_x;
        self.mass_point_y += rhs.mass_point_y;
        self.mass_point_z += rhs.mass_point_z;
        self.num_points += rhs.num_points;
    }

    pub(crate) fn get_error(&mut self) -> f32 {
        let pos = self.x;
        if !self.has_solution {
            self.atb = Vec3::new(self.atb_x, self.atb_y, self.atb_z);
        }
        let atax = self.ata.vmul(pos);
        self.last_error = pos.dot(atax) - 2.0 * pos.dot(self.atb) + self.btb;
        if self.last_error.is_nan() {
            self.last_error = 10000.0;
        }
        self.last_error
    }

    pub(crate) fn solve(&mut self, svd_tol: f32, svd_sweeps: i32, pinv_tol: f32) -> Vec3 {
        if self.num_points == 0 {
            panic!("QEFSolver: no points to solve");
        }
        self.mass_point = Vec3::new(self.mass_point_x, self.mass_point_y, self.mass_point_z);
        self.mass_point /= self.num_points as f32;
        self.atb = Vec3::new(self.atb_x, self.atb_y, self.atb_z);
        let tmpv = self.ata.vmul(self.mass_point);
        self.atb = self.atb - tmpv;
        self.x = Vec3::ZERO;
        let result = solve_symmetric(
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
        self.atb = Vec3::new(self.atb_x, self.atb_y, self.atb_z);
        self.has_solution = true;
        self.x
    }
}

impl Default for QEFSolver {
    fn default() -> Self {
        Self::new()
    }
}
