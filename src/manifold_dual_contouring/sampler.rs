use std::sync::Arc;

use glam::Vec3;

#[allow(dead_code)]
pub(crate) fn read_data_from_file() {
    todo!()
}

#[derive(Clone)]
pub struct FunBlobSampler {
    center: Vec3,
    radius: f32,
}

impl FunBlobSampler {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl Sampler for FunBlobSampler {
    fn sample(&self, point: Vec3) -> f32 {
        let p = point - self.center;
        let mut d = p.length() - self.radius;
        let offset = (p.x * 0.3).sin() + (p.y * 0.3).sin() + (p.z * 0.3).sin();
        d += offset * 5.0;
        let twist = ((p.y + p.z) * 0.2).sin() * 3.0;
        d += twist;
        d
    }
}

#[derive(Clone)]
pub struct FunSurfaceSampler {
    center: Vec3,
    height: f32,
}

impl FunSurfaceSampler {
    pub fn new(center: Vec3, height: f32) -> Self {
        Self { center, height }
    }
}

impl Sampler for FunSurfaceSampler {
    fn sample(&self, point: Vec3) -> f32 {
        let p = point - self.center;
        let mut d = p.y - self.height;
        let wave1 = (p.x * 0.08).sin() * (p.z * 0.06).cos() * 15.0;
        let wave2 = (p.x * 0.12 + p.z * 0.15).sin() * 12.0;
        d -= wave1 + wave2;
        let ripple =
            ((p.x * 0.3).sin() + (p.z * 0.3).cos()) * (p.x * 0.05 + p.z * 0.05).cos() * 8.0;
        d -= ripple;
        let spiral_angle = (p.x * p.x + p.z * p.z).sqrt() * 0.1;
        let spiral = (spiral_angle + (p.x * 0.2).sin()).sin() * 5.0;
        d -= spiral;
        let detail = ((p.x * 0.8).sin() * (p.z * 0.7).cos() + (p.x * 1.2 - p.z * 0.9).sin()) * 3.0;
        d -= detail;
        let chaos = (p.x * 1.5).sin() * (p.z * 1.8).cos() * (p.x * 2.1 + p.z * 1.7).sin() * 2.0;
        d -= chaos;
        d
    }
}

pub fn blend(a: f32, b: f32, k: f32) -> f32 {
    let a_k = a.powf(k);
    let b_k = b.powf(k);
    ((a_k * b_k) / (a_k + b_k)).powf(1.0 / k)
}

pub(crate) fn get_normal<S: Sampler>(v: Vec3, sampler: &S) -> Vec3 {
    let h = 0.001;
    let (x, y, z) = (v.x, v.y, v.z);
    let dxp = sampler.sample(Vec3::new(x + h, y, z));
    let dxm = sampler.sample(Vec3::new(x - h, y, z));
    let dyp = sampler.sample(Vec3::new(x, y + h, z));
    let dym = sampler.sample(Vec3::new(x, y - h, z));
    let dzp = sampler.sample(Vec3::new(x, y, z + h));
    let dzm = sampler.sample(Vec3::new(x, y, z - h));
    let grad = Vec3::new(dxp - dxm, dyp - dym, dzp - dzm);
    grad.normalize()
}

#[inline]
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

    pub fn bake(&self, min: Vec3, max: Vec3, resolution: (usize, usize, usize)) -> Vec<f32> {
        let (res_x, res_y, res_z) = resolution;
        let mut baked = Vec::with_capacity(res_x * res_y * res_z);
        let step_x = (max.x - min.x) / (res_x - 1).max(1) as f32;
        let step_y = (max.y - min.y) / (res_y - 1).max(1) as f32;
        let step_z = (max.z - min.z) / (res_z - 1).max(1) as f32;
        for z in 0..res_z {
            for y in 0..res_y {
                for x in 0..res_x {
                    let point = Vec3::new(
                        min.x + x as f32 * step_x,
                        min.y + y as f32 * step_y,
                        min.z + z as f32 * step_z,
                    );
                    baked.push(self.sample(point));
                }
            }
        }
        baked
    }

    pub fn bake_quantized(
        &self,
        min: Vec3,
        max: Vec3,
        resolution: (usize, usize, usize),
    ) -> Vec<i16> {
        let (res_x, res_y, res_z) = resolution;
        let mut baked = Vec::with_capacity(res_x * res_y * res_z);
        let step_x = (max.x - min.x) / (res_x - 1).max(1) as f32;
        let step_y = (max.y - min.y) / (res_y - 1).max(1) as f32;
        let step_z = (max.z - min.z) / (res_z - 1).max(1) as f32;
        for z in 0..res_z {
            for y in 0..res_y {
                for x in 0..res_x {
                    let point = Vec3::new(
                        min.x + x as f32 * step_x,
                        min.y + y as f32 * step_y,
                        min.z + z as f32 * step_z,
                    );
                    baked.push(quantize_f32_to_i16(self.sample(point)));
                }
            }
        }
        baked
    }
}

impl Sampler for SphereSampler {
    #[inline]
    fn sample(&self, point: Vec3) -> f32 {
        (point - self.center).length() - self.radius
    }
}

#[derive(Clone)]
pub struct CuboidSampler {
    center: Vec3,
    size: Vec3,
}

impl CuboidSampler {
    pub fn new(center: Vec3, size: Vec3) -> Self {
        Self { center, size }
    }

    pub fn bake(&self, min: Vec3, max: Vec3, resolution: (usize, usize, usize)) -> Vec<f32> {
        let (res_x, res_y, res_z) = resolution;
        let mut baked = Vec::with_capacity(res_x * res_y * res_z);
        let step_x = (max.x - min.x) / (res_x - 1).max(1) as f32;
        let step_y = (max.y - min.y) / (res_y - 1).max(1) as f32;
        let step_z = (max.z - min.z) / (res_z - 1).max(1) as f32;
        for z in 0..res_z {
            for y in 0..res_y {
                for x in 0..res_x {
                    let point = Vec3::new(
                        min.x + x as f32 * step_x,
                        min.y + y as f32 * step_y,
                        min.z + z as f32 * step_z,
                    );
                    baked.push(self.sample(point));
                }
            }
        }
        baked
    }

    pub fn bake_quantized(
        &self,
        min: Vec3,
        max: Vec3,
        resolution: (usize, usize, usize),
    ) -> Vec<i16> {
        let (res_x, res_y, res_z) = resolution;
        let mut baked = Vec::with_capacity(res_x * res_y * res_z);
        let step_x = (max.x - min.x) / (res_x - 1).max(1) as f32;
        let step_y = (max.y - min.y) / (res_y - 1).max(1) as f32;
        let step_z = (max.z - min.z) / (res_z - 1).max(1) as f32;
        for z in 0..res_z {
            for y in 0..res_y {
                for x in 0..res_x {
                    let point = Vec3::new(
                        min.x + x as f32 * step_x,
                        min.y + y as f32 * step_y,
                        min.z + z as f32 * step_z,
                    );
                    baked.push(quantize_f32_to_i16(self.sample(point)));
                }
            }
        }
        baked
    }
}

impl Sampler for CuboidSampler {
    #[inline]
    fn sample(&self, point: Vec3) -> f32 {
        let p = (point - self.center).abs() - self.size;
        p.max(Vec3::ZERO).length() + p.max_element().min(0.0)
    }
}

impl<S: Sampler> Sampler for Arc<S> {
    #[inline]
    fn sample(&self, point: Vec3) -> f32 {
        (**self).sample(point)
    }
}

#[inline]
pub fn quantize_f32_to_i16(value: f32) -> i16 {
    let scale = 32767.0 / 10.0; // Map [-10, 10] to [-32767, 32767]
    (value * scale).round() as i16
}
