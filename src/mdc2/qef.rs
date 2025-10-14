use glam::{Vec3, Vec4};
use std::cell::Cell;

use crate::mdc2::solver::LevenQefSolver;

#[derive(Clone)]
pub struct QefData {
    pub mat3x3_tri_ata: [f32; 6],
    pub atb: Vec4,
    pub mass_point: Vec4,
    x: Cell<Vec4>,
    solver: LevenQefSolver,
    btb: f32,
}

impl QefData {
    pub fn new(solver: LevenQefSolver) -> Self {
        QefData {
            mat3x3_tri_ata: [0.0; 6],
            atb: Vec4::ZERO,
            mass_point: Vec4::ZERO,
            x: Cell::new(Vec4::ZERO),
            solver,
            btb: 0.0,
        }
    }

    pub fn add(&mut self, rhs: &QefData) {
        for i in 0..6 {
            self.mat3x3_tri_ata[i] += rhs.mat3x3_tri_ata[i];
        }
        self.atb += rhs.atb;
        self.btb += rhs.btb;
        self.mass_point += rhs.mass_point;
    }

    pub fn qef_add_point3(&mut self, p: Vec3, mut n: Vec3) {
        n = n.normalize();
        self.mat3x3_tri_ata[0] += n.x * n.x;
        self.mat3x3_tri_ata[1] += n.x * n.y;
        self.mat3x3_tri_ata[2] += n.x * n.z;
        self.mat3x3_tri_ata[3] += n.y * n.y;
        self.mat3x3_tri_ata[4] += n.y * n.z;
        self.mat3x3_tri_ata[5] += n.z * n.z;
        let dot = n.x * p.x + n.y * p.y + n.z * p.z;
        self.atb.x += dot * n.x;
        self.atb.y += dot * n.y;
        self.atb.z += dot * n.z;
        self.btb += dot * dot;
        self.mass_point.x += p.x;
        self.mass_point.y += p.y;
        self.mass_point.z += p.z;
        self.mass_point.w += 1.0;
    }

    pub fn solve(&self) -> Vec4 {
        let result = self
            .solver
            .solve(&self.mat3x3_tri_ata, self.atb, self.mass_point);
        self.x.set(result);
        result
    }

    pub fn get_error(&self) -> f32 {
        self.get_error_for(self.x.get())
    }

    fn get_error_for(&self, pos: Vec4) -> f32 {
        let atax = self.svd_vmul_sym(&self.mat3x3_tri_ata, pos);
        let result = pos.dot(atax) - 2.0 * pos.dot(self.atb) + self.btb;
        result.max(0.0)
    }

    fn svd_vmul_sym(&self, mat: &[f32; 6], v: Vec4) -> Vec4 {
        let x = mat[0] * v.x + mat[1] * v.y + mat[2] * v.z;
        let y = mat[1] * v.x + mat[3] * v.y + mat[4] * v.z;
        let z = mat[2] * v.x + mat[4] * v.y + mat[5] * v.z;
        Vec4::new(x, y, z, 0.0)
    }
}
