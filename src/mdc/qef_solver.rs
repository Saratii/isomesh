// https://github.com/Lin20/isosurface/tree/master/Isosurface/Isosurface/ManifoldDC

use glam::Vec3;

use crate::mdc::{smat3::SMat3, svd::solve_symmetric};

#[derive(Clone)]
pub(crate) struct QEFSolver {
    ata: SMat3,
    atb: Vec3,
    btb: f32,
    num_points: i32,
    x: Vec3,
    mass_point: Vec3,
}

impl QEFSolver {
    pub(crate) fn new() -> Self {
        Self {
            ata: SMat3::ZERO,
            atb: Vec3::ZERO,
            btb: 0.0,
            num_points: 0,
            x: Vec3::ZERO,
            mass_point: Vec3::ZERO,
        }
    }

    pub(crate) fn add(&mut self, p: Vec3, n: Vec3) {
        let n = n.normalize();
        self.ata.m00 += n.x * n.x;
        self.ata.m01 += n.x * n.y;
        self.ata.m02 += n.x * n.z;
        self.ata.m11 += n.y * n.y;
        self.ata.m12 += n.y * n.z;
        self.ata.m22 += n.z * n.z;
        let dot = n.dot(p);
        self.atb += dot * n;
        self.btb += dot * dot;
        self.mass_point += p;
        self.num_points += 1;
    }

    pub(crate) fn add_qef(&mut self, rhs: &QEFSolver) {
        self.ata.add(&rhs.ata);
        self.atb += rhs.atb;
        self.btb += rhs.btb;
        self.mass_point += rhs.mass_point;
        self.num_points += rhs.num_points;
    }

    pub(crate) fn get_error(&self) -> f32 {
        let pos = self.x;
        let atax = self.ata.vmul(pos);
        let last_error = pos.dot(atax) - 2.0 * pos.dot(self.atb) + self.btb;
        if last_error.is_nan() {
            return 10000.0;
        }
        last_error
    }

    pub(crate) fn solve(&mut self, svd_tol: f32, svd_sweeps: i32, pinv_tol: f32) -> Vec3 {
        let mass_point_2 = self.mass_point / self.num_points as f32;
        let tmpv = self.ata.vmul(mass_point_2);
        let atb_2 = self.atb - tmpv;
        let result = solve_symmetric(
            &self.ata,
            &atb_2,
            &mut self.x,
            svd_tol,
            svd_sweeps,
            pinv_tol,
        );
        self.x = if result.is_nan() {
            mass_point_2
        } else {
            self.x + mass_point_2
        };
        self.x
    }
}
