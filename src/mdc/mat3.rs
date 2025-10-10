use glam::Vec3;

use crate::mdc::smat3::SMat3;

#[derive(Debug, Clone, Copy)]
pub struct Mat3 {
    pub m00: f32, pub m01: f32, pub m02: f32,
    pub m10: f32, pub m11: f32, pub m12: f32,
    pub m20: f32, pub m21: f32, pub m22: f32,
}

impl Mat3 {
    pub fn new() -> Self {
        Self {
            m00: 0.0, m01: 0.0, m02: 0.0,
            m10: 0.0, m11: 0.0, m12: 0.0,
            m20: 0.0, m21: 0.0, m22: 0.0,
        }
    }

    pub fn with_values(
        m00: f32, m01: f32, m02: f32,
        m10: f32, m11: f32, m12: f32,
        m20: f32, m21: f32, m22: f32
    ) -> Self {
        Self {
            m00, m01, m02,
            m10, m11, m12,
            m20, m21, m22,
        }
    }

    pub fn clear(&mut self) {
        self.set(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    }

    pub fn set(
        &mut self,
        m00: f32, m01: f32, m02: f32,
        m10: f32, m11: f32, m12: f32,
        m20: f32, m21: f32, m22: f32
    ) {
        self.m00 = m00; self.m01 = m01; self.m02 = m02;
        self.m10 = m10; self.m11 = m11; self.m12 = m12;
        self.m20 = m20; self.m21 = m21; self.m22 = m22;
    }

    pub fn set_from(&mut self, rhs: &Mat3) {
        self.set(rhs.m00, rhs.m01, rhs.m02, rhs.m10, rhs.m11, rhs.m12, rhs.m20, rhs.m21, rhs.m22);
    }

    pub fn set_symmetric_values(&mut self, a00: f32, a01: f32, a02: f32, a11: f32, a12: f32, a22: f32) {
        self.set(a00, a01, a02, a01, a11, a12, a02, a12, a22);
    }

    pub fn set_symmetric(&mut self, rhs: &SMat3) {
        self.set_symmetric_values(rhs.m00, rhs.m01, rhs.m02, rhs.m11, rhs.m12, rhs.m22);
    }

    pub fn fnorm(&self) -> f32 {
        ((self.m00 * self.m00) + (self.m01 * self.m01) + (self.m02 * self.m02)
            + (self.m10 * self.m10) + (self.m11 * self.m11) + (self.m12 * self.m12)
            + (self.m20 * self.m20) + (self.m21 * self.m21) + (self.m22 * self.m22)).sqrt()
    }

    pub fn off(&self) -> f32 {
        ((self.m01 * self.m01) + (self.m02 * self.m02) + (self.m10 * self.m10)
            + (self.m12 * self.m12) + (self.m20 * self.m20) + (self.m21 * self.m21)).sqrt()
    }

    pub fn mul(&self, b: &Mat3) -> Mat3 {
        Mat3::with_values(
            self.m00 * b.m00 + self.m01 * b.m10 + self.m02 * b.m20,
            self.m00 * b.m01 + self.m01 * b.m11 + self.m02 * b.m21,
            self.m00 * b.m02 + self.m01 * b.m12 + self.m02 * b.m22,
            self.m10 * b.m00 + self.m11 * b.m10 + self.m12 * b.m20,
            self.m10 * b.m01 + self.m11 * b.m11 + self.m12 * b.m21,
            self.m10 * b.m02 + self.m11 * b.m12 + self.m12 * b.m22,
            self.m20 * b.m00 + self.m21 * b.m10 + self.m22 * b.m20,
            self.m20 * b.m01 + self.m21 * b.m11 + self.m22 * b.m21,
            self.m20 * b.m02 + self.m21 * b.m12 + self.m22 * b.m22,
        )
    }

    pub fn mul_ata(&self) -> SMat3 {
        SMat3::with_values(
            self.m00 * self.m00 + self.m10 * self.m10 + self.m20 * self.m20,
            self.m00 * self.m01 + self.m10 * self.m11 + self.m20 * self.m21,
            self.m00 * self.m02 + self.m10 * self.m12 + self.m20 * self.m22,
            self.m01 * self.m01 + self.m11 * self.m11 + self.m21 * self.m21,
            self.m01 * self.m02 + self.m11 * self.m12 + self.m21 * self.m22,
            self.m02 * self.m02 + self.m12 * self.m12 + self.m22 * self.m22,
        )
    }

    pub fn transpose(&self) -> Mat3 {
        Mat3::with_values(
            self.m00, self.m10, self.m20,
            self.m01, self.m11, self.m21,
            self.m02, self.m12, self.m22,
        )
    }

    pub fn vmul(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            (self.m00 * v.x) + (self.m01 * v.y) + (self.m02 * v.z),
            (self.m10 * v.x) + (self.m11 * v.y) + (self.m12 * v.z),
            (self.m20 * v.x) + (self.m21 * v.y) + (self.m22 * v.z),
        )
    }

    pub fn rot01_post(&mut self, c: f32, s: f32) {
        let m00 = self.m00;
        let m01 = self.m01;
        let m10 = self.m10;
        let m11 = self.m11;
        let m20 = self.m20;
        let m21 = self.m21;
        
        self.set(
            c * m00 - s * m01, s * m00 + c * m01, self.m02,
            c * m10 - s * m11, s * m10 + c * m11, self.m12,
            c * m20 - s * m21, s * m20 + c * m21, self.m22,
        );
    }

    pub fn rot02_post(&mut self, c: f32, s: f32) {
        let m00 = self.m00;
        let m02 = self.m02;
        let m10 = self.m10;
        let m12 = self.m12;
        let m20 = self.m20;
        let m22 = self.m22;
        
        self.set(
            c * m00 - s * m02, self.m01, s * m00 + c * m02,
            c * m10 - s * m12, self.m11, s * m10 + c * m12,
            c * m20 - s * m22, self.m21, s * m20 + c * m22,
        );
    }

    pub fn rot12_post(&mut self, c: f32, s: f32) {
        let m01 = self.m01;
        let m02 = self.m02;
        let m11 = self.m11;
        let m12 = self.m12;
        let m21 = self.m21;
        let m22 = self.m22;
        
        self.set(
            self.m00, c * m01 - s * m02, s * m01 + c * m02,
            self.m10, c * m11 - s * m12, s * m11 + c * m12,
            self.m20, c * m21 - s * m22, s * m21 + c * m22,
        );
    }

    pub fn calc_symmetric_givens_coefficients(a_pp: f32, a_pq: f32, a_qq: f32, c: &mut f32, s: &mut f32) {
        if a_pq == 0.0 {
            *c = 1.0;
            *s = 0.0;
            return;
        }
        
        let tau = (a_qq - a_pp) / (2.0 * a_pq);
        let stt = (1.0 + tau * tau).sqrt();
        let tan = 1.0 / if tau >= 0.0 { tau + stt } else { tau - stt };
        *c = 1.0 / (1.0 + tan * tan).sqrt();
        *s = tan * *c;
    }
}

impl Default for Mat3 {
    fn default() -> Self {
        Self::new()
    }
}