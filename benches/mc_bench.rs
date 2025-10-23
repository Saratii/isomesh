use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::Vec3;
use isomesh::{
    manifold_dual_contouring::sampler::{CuboidSampler, SphereSampler},
    marching_cubes::{
        color_provider::NormalColorProvider,
        mc::{MeshBuffers, mc_mesh_generation},
    },
};

const SAMPLES_PER_CHUNK_DIM: usize = 64; // Number of voxel sample points
const VOXEL_SIZE: f32 = 1.0; // Size of each voxel in meters
const CUBES_PER_CHUNK_DIM: usize = SAMPLES_PER_CHUNK_DIM - 1; // 63 cubes
const CHUNK_SIZE: f32 = CUBES_PER_CHUNK_DIM as f32 * VOXEL_SIZE; // 7.875 meters
const HALF_CHUNK: f32 = CHUNK_SIZE / 2.0;

fn bench_single_sphere_small_normal(c: &mut Criterion) {
    let densities = SphereSampler::new(Vec3::ZERO, 10.0).bake_quantized(
        Vec3::new(-HALF_CHUNK, -HALF_CHUNK, -HALF_CHUNK),
        Vec3::new(HALF_CHUNK, HALF_CHUNK, HALF_CHUNK),
        (
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM];
    c.bench_function("single_sphere_small_normal", |b| {
        b.iter(|| {
            let mut mesh_buffers = MeshBuffers::new();
            mc_mesh_generation(
                &mut mesh_buffers,
                &densities,
                &materials,
                CUBES_PER_CHUNK_DIM,
                SAMPLES_PER_CHUNK_DIM,
                &NormalColorProvider,
                HALF_CHUNK,
                VOXEL_SIZE,
            );
            black_box(mesh_buffers);
        });
    });
}

fn bench_single_sphere_large_normal(c: &mut Criterion) {
    let densities = SphereSampler::new(Vec3::ZERO, 30.0).bake_quantized(
        Vec3::new(-HALF_CHUNK, -HALF_CHUNK, -HALF_CHUNK),
        Vec3::new(HALF_CHUNK, HALF_CHUNK, HALF_CHUNK),
        (
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM];
    c.bench_function("single_sphere_large_normal", |b| {
        b.iter(|| {
            let mut mesh_buffers = MeshBuffers::new();
            mc_mesh_generation(
                &mut mesh_buffers,
                &densities,
                &materials,
                CUBES_PER_CHUNK_DIM,
                SAMPLES_PER_CHUNK_DIM,
                &NormalColorProvider,
                HALF_CHUNK,
                VOXEL_SIZE,
            );
            black_box(mesh_buffers);
        });
    });
}

fn bench_bulk_spheres_small_normal(c: &mut Criterion) {
    let densities = SphereSampler::new(Vec3::ZERO, 10.0).bake_quantized(
        Vec3::new(-HALF_CHUNK, -HALF_CHUNK, -HALF_CHUNK),
        Vec3::new(HALF_CHUNK, HALF_CHUNK, HALF_CHUNK),
        (
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM];
    c.bench_function("bulk_spheres_small_10x_normal", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut mesh_buffers = MeshBuffers::new();
                mc_mesh_generation(
                    &mut mesh_buffers,
                    &densities,
                    &materials,
                    CUBES_PER_CHUNK_DIM,
                    SAMPLES_PER_CHUNK_DIM,
                    &NormalColorProvider,
                    HALF_CHUNK,
                    VOXEL_SIZE,
                );
                black_box(mesh_buffers);
            }
        });
    });
}

fn bench_bulk_spheres_large_normal(c: &mut Criterion) {
    let densities = SphereSampler::new(Vec3::ZERO, 30.0).bake_quantized(
        Vec3::new(-HALF_CHUNK, -HALF_CHUNK, -HALF_CHUNK),
        Vec3::new(HALF_CHUNK, HALF_CHUNK, HALF_CHUNK),
        (
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM];
    c.bench_function("bulk_spheres_large_10x_normal", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut mesh_buffers = MeshBuffers::new();
                mc_mesh_generation(
                    &mut mesh_buffers,
                    &densities,
                    &materials,
                    CUBES_PER_CHUNK_DIM,
                    SAMPLES_PER_CHUNK_DIM,
                    &NormalColorProvider,
                    HALF_CHUNK,
                    VOXEL_SIZE,
                );
                black_box(mesh_buffers);
            }
        });
    });
}

fn bench_single_cube_small_normal(c: &mut Criterion) {
    let densities = CuboidSampler::new(Vec3::ZERO, Vec3::new(2.0, 4.0, 8.0)).bake_quantized(
        Vec3::new(-HALF_CHUNK, -HALF_CHUNK, -HALF_CHUNK),
        Vec3::new(HALF_CHUNK, HALF_CHUNK, HALF_CHUNK),
        (
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM];
    c.bench_function("single_cube_small_normal", |b| {
        b.iter(|| {
            let mut mesh_buffers = MeshBuffers::new();
            mc_mesh_generation(
                &mut mesh_buffers,
                &densities,
                &materials,
                CUBES_PER_CHUNK_DIM,
                SAMPLES_PER_CHUNK_DIM,
                &NormalColorProvider,
                HALF_CHUNK,
                VOXEL_SIZE,
            );
            black_box(mesh_buffers);
        });
    });
}

fn bench_single_cube_large_normal(c: &mut Criterion) {
    let densities = CuboidSampler::new(Vec3::ZERO, Vec3::new(10.0, 15.0, 20.0)).bake_quantized(
        Vec3::new(-HALF_CHUNK, -HALF_CHUNK, -HALF_CHUNK),
        Vec3::new(HALF_CHUNK, HALF_CHUNK, HALF_CHUNK),
        (
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM];
    c.bench_function("single_cube_large_normal", |b| {
        b.iter(|| {
            let mut mesh_buffers = MeshBuffers::new();
            mc_mesh_generation(
                &mut mesh_buffers,
                &densities,
                &materials,
                CUBES_PER_CHUNK_DIM,
                SAMPLES_PER_CHUNK_DIM,
                &NormalColorProvider,
                HALF_CHUNK,
                VOXEL_SIZE,
            );
            black_box(mesh_buffers);
        });
    });
}

fn bench_bulk_cubes_small_normal(c: &mut Criterion) {
    let densities = CuboidSampler::new(Vec3::ZERO, Vec3::new(2.0, 4.0, 8.0)).bake_quantized(
        Vec3::new(-HALF_CHUNK, -HALF_CHUNK, -HALF_CHUNK),
        Vec3::new(HALF_CHUNK, HALF_CHUNK, HALF_CHUNK),
        (
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM];
    c.bench_function("bulk_cubes_small_10x_normal", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut mesh_buffers = MeshBuffers::new();
                mc_mesh_generation(
                    &mut mesh_buffers,
                    &densities,
                    &materials,
                    CUBES_PER_CHUNK_DIM,
                    SAMPLES_PER_CHUNK_DIM,
                    &NormalColorProvider,
                    HALF_CHUNK,
                    VOXEL_SIZE,
                );
                black_box(mesh_buffers);
            }
        });
    });
}

fn bench_bulk_cubes_large_normal(c: &mut Criterion) {
    let densities = CuboidSampler::new(Vec3::ZERO, Vec3::new(10.0, 15.0, 20.0)).bake_quantized(
        Vec3::new(-HALF_CHUNK, -HALF_CHUNK, -HALF_CHUNK),
        Vec3::new(HALF_CHUNK, HALF_CHUNK, HALF_CHUNK),
        (
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
            SAMPLES_PER_CHUNK_DIM,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM * SAMPLES_PER_CHUNK_DIM];
    c.bench_function("bulk_cubes_large_10x_normal", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut mesh_buffers = MeshBuffers::new();
                mc_mesh_generation(
                    &mut mesh_buffers,
                    &densities,
                    &materials,
                    CUBES_PER_CHUNK_DIM,
                    SAMPLES_PER_CHUNK_DIM,
                    &NormalColorProvider,
                    HALF_CHUNK,
                    VOXEL_SIZE,
                );
                black_box(mesh_buffers);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_single_sphere_small_normal,
    bench_single_sphere_large_normal,
    bench_bulk_spheres_small_normal,
    bench_bulk_spheres_large_normal,
    bench_single_cube_small_normal,
    bench_single_cube_large_normal,
    bench_bulk_cubes_small_normal,
    bench_bulk_cubes_large_normal,
);

criterion_main!(benches);
