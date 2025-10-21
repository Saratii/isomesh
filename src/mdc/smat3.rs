use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub(crate) fn add(&mut self, rhs: &SMat3) {
        self.m00 += rhs.m00;
        self.m01 += rhs.m01;
        self.m02 += rhs.m02;
        self.m11 += rhs.m11;
        self.m12 += rhs.m12;
        self.m22 += rhs.m22;
    }
}
