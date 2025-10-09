use glam::Vec3;

use crate::mat3::Mat3;

#[derive(Debug, Clone, Copy)]
pub struct SMat3 {
    pub m00: f32,
    pub m01: f32,
    pub m02: f32,
    pub m11: f32,
    pub m12: f32,
    pub m22: f32,
}

impl SMat3 {
    pub fn new() -> Self {
        Self {
            m00: 0.0,
            m01: 0.0,
            m02: 0.0,
            m11: 0.0,
            m12: 0.0,
            m22: 0.0,
        }
    }

    pub fn with_values(m00: f32, m01: f32, m02: f32, m11: f32, m12: f32, m22: f32) -> Self {
        Self {
            m00,
            m01,
            m02,
            m11,
            m12,
            m22,
        }
    }

    pub fn clear(&mut self) {
        self.set_symmetric(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    }

    pub fn set_symmetric(&mut self, a00: f32, a01: f32, a02: f32, a11: f32, a12: f32, a22: f32) {
        self.m00 = a00;
        self.m01 = a01;
        self.m02 = a02;
        self.m11 = a11;
        self.m12 = a12;
        self.m22 = a22;
    }

    pub fn set_symmetric_from(&mut self, rhs: &SMat3) {
        self.set_symmetric(rhs.m00, rhs.m01, rhs.m02, rhs.m11, rhs.m12, rhs.m22);
    }

    pub fn fnorm(&self) -> f32 {
        ((self.m00 * self.m00)
            + (self.m01 * self.m01)
            + (self.m02 * self.m02)
            + (self.m01 * self.m01)
            + (self.m11 * self.m11)
            + (self.m12 * self.m12)
            + (self.m02 * self.m02)
            + (self.m12 * self.m12)
            + (self.m22 * self.m22))
            .sqrt()
    }

    pub fn off(&self) -> f32 {
        (2.0 * ((self.m01 * self.m01) + (self.m02 * self.m02) + (self.m12 * self.m12))).sqrt()
    }

    pub fn mul_ata(&self, a: &Mat3) -> SMat3 {
        SMat3::with_values(
            a.m00 * a.m00 + a.m10 * a.m10 + a.m20 * a.m20,
            a.m00 * a.m01 + a.m10 * a.m11 + a.m20 * a.m21,
            a.m00 * a.m02 + a.m10 * a.m12 + a.m20 * a.m22,
            a.m01 * a.m01 + a.m11 * a.m11 + a.m21 * a.m21,
            a.m01 * a.m02 + a.m11 * a.m12 + a.m21 * a.m22,
            a.m02 * a.m02 + a.m12 * a.m12 + a.m22 * a.m22,
        )
    }

    pub fn vmul(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            (self.m00 * v.x) + (self.m01 * v.y) + (self.m02 * v.z),
            (self.m01 * v.x) + (self.m11 * v.y) + (self.m12 * v.z),
            (self.m02 * v.x) + (self.m12 * v.y) + (self.m22 * v.z),
        )
    }

    pub fn rot01(&mut self, c: &mut f32, s: &mut f32) {
        Mat3::calc_symmetric_givens_coefficients(self.m00, self.m01, self.m11, c, s);
        let cc = *c * *c;
        let ss = *s * *s;
        let mix = 2.0 * *c * *s * self.m01;

        self.set_symmetric(
            cc * self.m00 - mix + ss * self.m11,
            0.0,
            *c * self.m02 - *s * self.m12,
            ss * self.m00 + mix + cc * self.m11,
            *s * self.m02 + *c * self.m12,
            self.m22,
        );
    }

    pub fn rot02(&mut self, c: &mut f32, s: &mut f32) {
        Mat3::calc_symmetric_givens_coefficients(self.m00, self.m02, self.m22, c, s);
        let cc = *c * *c;
        let ss = *s * *s;
        let mix = 2.0 * *c * *s * self.m02;

        self.set_symmetric(
            cc * self.m00 - mix + ss * self.m22,
            *c * self.m01 - *s * self.m12,
            0.0,
            self.m11,
            *s * self.m01 + *c * self.m12,
            ss * self.m00 + mix + cc * self.m22,
        );
    }

    pub fn rot12(&mut self, c: &mut f32, s: &mut f32) {
        Mat3::calc_symmetric_givens_coefficients(self.m11, self.m12, self.m22, c, s);
        let cc = *c * *c;
        let ss = *s * *s;
        let mix = 2.0 * *c * *s * self.m12;

        self.set_symmetric(
            self.m00,
            *c * self.m01 - *s * self.m02,
            *s * self.m01 + *c * self.m02,
            cc * self.m11 - mix + ss * self.m22,
            0.0,
            ss * self.m11 + mix + cc * self.m22,
        );
    }
}

impl Default for SMat3 {
    fn default() -> Self {
        Self::new()
    }
}
