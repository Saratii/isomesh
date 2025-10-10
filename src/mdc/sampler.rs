use glam::Vec3;

pub(crate) const RESOLUTION: i32 = 64;

#[allow(dead_code)]
pub(crate) fn read_data_from_file() {
    todo!()
}

pub fn sphere_sdf(point: Vec3, center: Vec3, radius: f32) -> f32 {
    (point - center).length() - radius
}

pub fn cuboid_sdf(pos: Vec3, radius: f32) -> f32 {
    let b = Vec3::splat(radius);
    let d = pos.abs() - b;
    let outside = Vec3::max(d, Vec3::ZERO).length();
    let inside = d.x.max(d.y).max(d.z).min(0.0);
    outside + inside
}

pub fn fun_blob_sdf(pos: Vec3) -> f32 {
    let res = RESOLUTION as f32;
    let center = Vec3::splat((res - 2.0) * 0.5);
    let radius = res * 0.4;
    let mut d = pos.distance_squared(center).sqrt() - radius;
    let offset = (pos.x * 0.3).sin() + (pos.y * 0.3).sin() + (pos.z * 0.3).sin();
    d += offset * 5.0;
    let twist = ((pos.y + pos.z) * 0.2).sin() * 3.0;
    d += twist;
    d
}

pub fn fun_surface(pos: Vec3) -> f32 {
    let mut d = pos.y - RESOLUTION as f32 * 0.3;
    let wave1 = (pos.x * 0.08).sin() * (pos.z * 0.06).cos() * 15.0;
    let wave2 = (pos.x * 0.12 + pos.z * 0.15).sin() * 12.0;
    d -= wave1 + wave2;
    let ripple =
        ((pos.x * 0.3).sin() + (pos.z * 0.3).cos()) * (pos.x * 0.05 + pos.z * 0.05).cos() * 8.0;
    d -= ripple;
    let spiral_angle = (pos.x * pos.x + pos.z * pos.z).sqrt() * 0.1;
    let spiral = (spiral_angle + (pos.x * 0.2).sin()).sin() * 5.0;
    d -= spiral;
    let detail =
        ((pos.x * 0.8).sin() * (pos.z * 0.7).cos() + (pos.x * 1.2 - pos.z * 0.9).sin()) * 3.0;
    d -= detail;
    let chaos = (pos.x * 1.5).sin() * (pos.z * 1.8).cos() * (pos.x * 2.1 + pos.z * 1.7).sin() * 2.0;
    d -= chaos;
    d
}

pub fn blend(a: f32, b: f32, k: f32) -> f32 {
    let a_k = a.powf(k);
    let b_k = b.powf(k);
    ((a_k * b_k) / (a_k + b_k)).powf(1.0 / k)
}

pub(crate) fn get_normal<S>(v: Vec3, sampler: &S) -> Vec3
where
    S: Sampler,
{
    let h = 0.001;
    let dxp = sampler.sample(Vec3::new(v.x + h, v.y, v.z));
    let dxm = sampler.sample(Vec3::new(v.x - h, v.y, v.z));
    let dyp = sampler.sample(Vec3::new(v.x, v.y + h, v.z));
    let dym = sampler.sample(Vec3::new(v.x, v.y - h, v.z));
    let dzp = sampler.sample(Vec3::new(v.x, v.y, v.z + h));
    let dzm = sampler.sample(Vec3::new(v.x, v.y, v.z - h));
    let grad = Vec3::new(dxp - dxm, dyp - dym, dzp - dzm);
    grad.normalize()
}

pub(crate) fn get_intersection(p1: Vec3, p2: Vec3, d1: f32, d2: f32) -> Vec3 {
    p1 + (-d1) * (p2 - p1) / (d2 - d1)
}

pub trait Sampler {
    fn sample(&self, point: Vec3) -> f32;
}

#[derive(Clone)]
pub struct SphereSampler {
    center: Vec3,
    radius: f32,
}

impl SphereSampler {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl Sampler for SphereSampler {
    fn sample(&self, point: Vec3) -> f32 {
        (point - self.center).length() - self.radius
    }
}

#[derive(Clone)]
pub struct CubeSampler {
    center: Vec3,
    size: Vec3,
}

impl CubeSampler {
    pub fn new(center: Vec3, size: Vec3) -> Self {
        Self { center, size }
    }
}

impl Sampler for CubeSampler {
    fn sample(&self, point: Vec3) -> f32 {
        let p = (point - self.center).abs() - self.size;
        p.max(Vec3::ZERO).length() + p.max_element().min(0.0)
    }
}
