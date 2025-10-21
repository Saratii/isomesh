use glam::Vec3;

use crate::mdc::mat3::calc_symmetric_givens_coefficients;

#[derive(Debug, Clone, Copy)]
pub(crate) struct SMat3 {
    pub(crate) m00: f32,
    pub(crate) m01: f32,
    pub(crate) m02: f32,
    pub(crate) m11: f32,
    pub(crate) m12: f32,
    pub(crate) m22: f32,
}

impl SMat3 {
    pub(crate) const ZERO: Self = Self {
        m00: 0.0,
        m01: 0.0,
        m02: 0.0,
        m11: 0.0,
        m12: 0.0,
        m22: 0.0,
    };

    #[inline]
    #[must_use]
    pub(crate) fn fnorm(&self) -> f32 {
        (self.m00 * self.m00
            + 2.0 * (self.m01 * self.m01 + self.m02 * self.m02 + self.m12 * self.m12)
            + self.m11 * self.m11
            + self.m22 * self.m22)
            .sqrt()
    }

    #[inline]
    #[must_use]
    pub(crate) fn vmul(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.m00 * v.x + self.m01 * v.y + self.m02 * v.z,
            self.m01 * v.x + self.m11 * v.y + self.m12 * v.z,
            self.m02 * v.x + self.m12 * v.y + self.m22 * v.z,
        )
    }

    #[inline]
    pub(crate) fn rot01(&mut self) {
        let (c, s) = calc_symmetric_givens_coefficients(self.m00, self.m01, self.m11);
        let cc = c * c;
        let ss = s * s;
        let mix = 2.0 * c * s * self.m01;
        self.m00 = cc * self.m00 - mix + ss * self.m11;
        self.m01 = 0.0;
        self.m02 = c * self.m02 - s * self.m12;
        self.m11 = ss * self.m00 + mix + cc * self.m11;
        self.m12 = s * self.m02 + c * self.m12;
    }

    #[inline]
    pub(crate) fn rot02(&mut self) {
        let (c, s) = calc_symmetric_givens_coefficients(self.m00, self.m02, self.m22);
        let cc = c * c;
        let ss = s * s;
        let mix = 2.0 * c * s * self.m02;
        self.m00 = cc * self.m00 - mix + ss * self.m22;
        self.m01 = c * self.m01 - s * self.m12;
        self.m02 = 0.0;
        self.m12 = s * self.m01 + c * self.m12;
        self.m22 = ss * self.m00 + mix + cc * self.m22;
    }

    #[inline]
    pub(crate) fn rot12(&mut self) {
        let (c, s) = calc_symmetric_givens_coefficients(self.m11, self.m12, self.m22);
        let cc = c * c;
        let ss = s * s;
        let mix = 2.0 * c * s * self.m12;
        self.m01 = c * self.m01 - s * self.m02;
        self.m02 = s * self.m01 + c * self.m02;
        self.m11 = cc * self.m11 - mix + ss * self.m22;
        self.m12 = 0.0;
        self.m22 = ss * self.m11 + mix + cc * self.m22;
    }
}
