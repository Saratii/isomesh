use glam::Vec3;

use crate::mdc::smat3::SMat3;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Mat3 {
    pub(crate) m00: f32,
    pub(crate) m01: f32,
    pub(crate) m02: f32,
    pub(crate) m10: f32,
    pub(crate) m11: f32,
    pub(crate) m12: f32,
    pub(crate) m20: f32,
    pub(crate) m21: f32,
    pub(crate) m22: f32,
}

impl Default for Mat3 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Mat3 {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self::ZERO
    }

    pub(crate) const ZERO: Self = Self {
        m00: 0.0,
        m01: 0.0,
        m02: 0.0,
        m10: 0.0,
        m11: 0.0,
        m12: 0.0,
        m20: 0.0,
        m21: 0.0,
        m22: 0.0,
    };

    pub(crate) fn set(
        &mut self,
        m00: f32,
        m01: f32,
        m02: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m20: f32,
        m21: f32,
        m22: f32,
    ) {
        self.m00 = m00;
        self.m01 = m01;
        self.m02 = m02;
        self.m10 = m10;
        self.m11 = m11;
        self.m12 = m12;
        self.m20 = m20;
        self.m21 = m21;
        self.m22 = m22;
    }

    pub(crate) fn set_symmetric_values(
        &mut self,
        a00: f32,
        a01: f32,
        a02: f32,
        a11: f32,
        a12: f32,
        a22: f32,
    ) {
        self.set(a00, a01, a02, a01, a11, a12, a02, a12, a22);
    }

    pub(crate) fn set_symmetric(&mut self, rhs: &SMat3) {
        self.set_symmetric_values(rhs.m00, rhs.m01, rhs.m02, rhs.m11, rhs.m12, rhs.m22);
    }

    #[inline]
    pub(crate) fn vmul(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.m00 * v.x + self.m01 * v.y + self.m02 * v.z,
            self.m10 * v.x + self.m11 * v.y + self.m12 * v.z,
            self.m20 * v.x + self.m21 * v.y + self.m22 * v.z,
        )
    }

    #[inline]
    pub(crate) fn rot01_post(&mut self, c: f32, s: f32) {
        let m00 = self.m00;
        let m01 = self.m01;
        let m10 = self.m10;
        let m11 = self.m11;
        let m20 = self.m20;
        let m21 = self.m21;
        self.set(
            c * m00 - s * m01,
            s * m00 + c * m01,
            self.m02,
            c * m10 - s * m11,
            s * m10 + c * m11,
            self.m12,
            c * m20 - s * m21,
            s * m20 + c * m21,
            self.m22,
        );
    }

    #[inline]
    pub(crate) fn rot02_post(&mut self, c: f32, s: f32) {
        let m00 = self.m00;
        let m02 = self.m02;
        let m10 = self.m10;
        let m12 = self.m12;
        let m20 = self.m20;
        let m22 = self.m22;
        self.set(
            c * m00 - s * m02,
            self.m01,
            s * m00 + c * m02,
            c * m10 - s * m12,
            self.m11,
            s * m10 + c * m12,
            c * m20 - s * m22,
            self.m21,
            s * m20 + c * m22,
        );
    }

    #[inline]
    pub(crate) fn rot12_post(&mut self, c: f32, s: f32) {
        let m01 = self.m01;
        let m02 = self.m02;
        let m11 = self.m11;
        let m12 = self.m12;
        let m21 = self.m21;
        let m22 = self.m22;
        self.set(
            self.m00,
            c * m01 - s * m02,
            s * m01 + c * m02,
            self.m10,
            c * m11 - s * m12,
            s * m11 + c * m12,
            self.m20,
            c * m21 - s * m22,
            s * m21 + c * m22,
        );
    }

    #[inline]
    pub(crate) fn calc_symmetric_givens_coefficients(
        a_pp: f32,
        a_pq: f32,
        a_qq: f32,
        c: &mut f32,
        s: &mut f32,
    ) {
        if a_pq == 0.0 {
            *c = 1.0;
            *s = 0.0;
            return;
        }
        let tau = (a_qq - a_pp) / (2.0 * a_pq);
        let stt = (1.0 + tau * tau).sqrt();
        let tan = 1.0 / (tau + stt.copysign(tau));
        *c = 1.0 / (1.0 + tan * tan).sqrt();
        *s = tan * *c;
    }
}
