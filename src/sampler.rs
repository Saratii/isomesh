use glam::Vec3;
use std::fs::File;
use std::io::Read;
use std::sync::{Mutex, OnceLock};

pub struct Sampler;

static IMAGE_DATA: OnceLock<Vec<f32>> = OnceLock::new();
static IMAGE_DIMS: OnceLock<Mutex<(usize, usize, usize)>> = OnceLock::new();
static IMAGE_SCALE: OnceLock<Mutex<f32>> = OnceLock::new();
static IMAGE_FLIP: OnceLock<Mutex<bool>> = OnceLock::new();

impl Sampler {
    pub const RESOLUTION: i32 = 64;
    pub const EDGES: [(usize, usize); 4] = [(0, 2), (1, 3), (0, 1), (2, 3)];

    pub fn read_data_from_file(
        path: &str,
        width: usize,
        height: usize,
        length: usize,
        iso_level: f32,
        scale: f32,
        flip: bool,
        bytes_per_voxel: usize,
    ) -> std::io::Result<()> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let mut data = vec![0.0f32; width * height * length];
        let mut index = 0;
        for x in 0..width {
            for y in 0..height {
                for z in 0..length {
                    let value = if bytes_per_voxel == 1 {
                        buffer[index] as f32 / 255.0 - iso_level
                    } else {
                        let low = buffer[index] as u16;
                        let high = buffer[index + 1] as u16;
                        (low + (high << 8)) as f32 / 4095.0 - iso_level
                    };
                    index += bytes_per_voxel;
                    data[x * height * length + y * length + z] = value;
                }
            }
        }
        IMAGE_DATA.set(data).ok();
        IMAGE_DIMS.get_or_init(|| Mutex::new((width, height, length)));
        IMAGE_SCALE.get_or_init(|| Mutex::new(scale));
        IMAGE_FLIP.get_or_init(|| Mutex::new(flip));
        if let Some(dims) = IMAGE_DIMS.get() {
            *dims.lock().unwrap() = (width, height, length);
        }
        if let Some(s) = IMAGE_SCALE.get() {
            *s.lock().unwrap() = scale;
        }
        if let Some(f) = IMAGE_FLIP.get() {
            *f.lock().unwrap() = flip;
        }
        Ok(())
    }

    pub fn sphere(pos: Vec3) -> f32 {
        let radius = Self::RESOLUTION as f32 / 2.0 - 2.0;
        let origin = Vec3::splat((Self::RESOLUTION as f32 - 2.0) * 0.5);
        pos.distance_squared(origin) - radius * radius
    }

    pub fn sphere_r(pos: Vec3) -> f32 {
        let radius = Self::RESOLUTION as f32 / 3.0 - 2.0;
        let origin = Vec3::splat((Self::RESOLUTION as f32 - 2.0) * 0.5);
        pos.distance_squared(origin) - radius * radius
    }

    pub fn cuboid(pos: Vec3) -> f32 {
        let radius = Self::RESOLUTION as f32 / 8.0;
        let local = pos - Vec3::splat(Self::RESOLUTION as f32 / 2.0);
        let d = local.abs() - Vec3::splat(radius);
        let m = d.x.max(d.y).max(d.z);
        m.min(d.max_element())
    }

    pub fn cuboid_r(pos: Vec3, radius: f32) -> f32 {
        let local = pos - Vec3::splat(Self::RESOLUTION as f32 / 2.0);
        let d = local.abs() - Vec3::splat(radius);
        let m = d.x.max(d.y).max(d.z);
        m.min(d.max_element())
    }

    pub fn fun_blob(pos: Vec3) -> f32 {
        let res = Self::RESOLUTION as f32;
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
        let mut d = pos.y - Self::RESOLUTION as f32 * 0.3;
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
        let chaos =
            (pos.x * 1.5).sin() * (pos.z * 1.8).cos() * (pos.x * 2.1 + pos.z * 1.7).sin() * 2.0;
        d -= chaos;
        d
    }

    pub fn blend(a: f32, b: f32, k: f32) -> f32 {
        let a_k = a.powf(k);
        let b_k = b.powf(k);
        ((a_k * b_k) / (a_k + b_k)).powf(1.0 / k)
    }

    pub fn sample(pos: Vec3) -> f32 {
        if let Some(data) = IMAGE_DATA.get() {
            let (w, h, l) = *IMAGE_DIMS.get().unwrap().lock().unwrap();
            let scale = *IMAGE_SCALE.get().unwrap().lock().unwrap();
            let flip = *IMAGE_FLIP.get().unwrap().lock().unwrap();
            let mut p = pos * scale;
            p.x = p.x.clamp(0.0, (w - 1) as f32);
            p.y = if flip {
                (h - 1) as f32 - p.y
            } else {
                p.y.clamp(0.0, (h - 1) as f32)
            };
            p.z = p.z.clamp(0.0, (l - 1) as f32);
            let xi = p.x as usize;
            let yi = p.y as usize;
            let zi = p.z as usize;
            return data[xi * h * l + yi * l + zi];
        }

        // default procedural sampler
        Self::sphere(pos)
    }

    pub fn get_normal(v: Vec3) -> Vec3 {
        let h = 0.001;
        let dxp = Self::sample(Vec3::new(v.x + h, v.y, v.z));
        let dxm = Self::sample(Vec3::new(v.x - h, v.y, v.z));
        let dyp = Self::sample(Vec3::new(v.x, v.y + h, v.z));
        let dym = Self::sample(Vec3::new(v.x, v.y - h, v.z));
        let dzp = Self::sample(Vec3::new(v.x, v.y, v.z + h));
        let dzm = Self::sample(Vec3::new(v.x, v.y, v.z - h));

        let grad = Vec3::new(dxp - dxm, dyp - dym, dzp - dzm);
        grad.normalize()
    }

    pub fn get_intersection(p1: Vec3, p2: Vec3, d1: f32, d2: f32) -> Vec3 {
        p1 + (-d1) * (p2 - p1) / (d2 - d1)
    }
}
