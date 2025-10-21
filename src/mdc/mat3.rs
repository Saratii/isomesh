use glam::Vec3;

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

impl Mat3 {
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

    pub(crate) const IDENTITY: Self = Self {
        m00: 1.0,
        m01: 0.0,
        m02: 0.0,
        m10: 0.0,
        m11: 1.0,
        m12: 0.0,
        m20: 0.0,
        m21: 0.0,
        m22: 1.0,
    };

    #[must_use]
    #[inline]
    pub(crate) fn vmul(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.m00 * v.x + self.m01 * v.y + self.m02 * v.z,
            self.m10 * v.x + self.m11 * v.y + self.m12 * v.z,
            self.m20 * v.x + self.m21 * v.y + self.m22 * v.z,
        )
    }
}

#[inline]
pub(crate) fn calc_symmetric_givens_coefficients(a_pp: f32, a_pq: f32, a_qq: f32) -> (f32, f32) {
    if a_pq == 0.0 {
        return (1.0, 0.0);
    }
    let tau = (a_qq - a_pp) / (2.0 * a_pq);
    let stt = (1.0 + tau * tau).sqrt();
    let tan = 1.0 / (tau + stt.copysign(tau));
    let c = 1.0 / (1.0 + tan * tan).sqrt();
    let s = tan * c;
    (c, s)
}
