use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::Vec3;
use isomesh::{
    manifold_dual_contouring::sampler::{CuboidSampler, SphereSampler},
    marching_cubes::{
        color_provider::NormalColorProvider,
        mc::{MeshBuffers, mc_mesh_generation},
    },
};

const SAMPLES_PER_CHUNK_DIM_LARGE: usize = 64;
const SAMPLES_PER_CHUNK_DIM_SMALL: usize = 16;
const BOUNDING_WIDTH: f32 = 64.0;
const HALF_EXTENT: f32 = BOUNDING_WIDTH / 2.0;

fn bench_single_sphere_small_normal(c: &mut Criterion) {
    let densities = SphereSampler::new(Vec3::ZERO, HALF_EXTENT).bake_quantized(
        Vec3::new(-HALF_EXTENT, -HALF_EXTENT, -HALF_EXTENT),
        Vec3::new(HALF_EXTENT, HALF_EXTENT, HALF_EXTENT),
        (
            SAMPLES_PER_CHUNK_DIM_SMALL,
            SAMPLES_PER_CHUNK_DIM_SMALL,
            SAMPLES_PER_CHUNK_DIM_SMALL,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM_SMALL
        * SAMPLES_PER_CHUNK_DIM_SMALL
        * SAMPLES_PER_CHUNK_DIM_SMALL];
    c.bench_function("single_sphere_small_normal", |b| {
        b.iter(|| {
            let mut mesh_buffers = MeshBuffers::new();
            mc_mesh_generation(
                &mut mesh_buffers,
                &densities,
                &materials,
                SAMPLES_PER_CHUNK_DIM_SMALL,
                &NormalColorProvider,
                HALF_EXTENT,
            );
            black_box(mesh_buffers);
        });
    });
}

fn bench_single_sphere_large_normal(c: &mut Criterion) {
    let densities = SphereSampler::new(Vec3::ZERO, HALF_EXTENT).bake_quantized(
        Vec3::new(-HALF_EXTENT, -HALF_EXTENT, -HALF_EXTENT),
        Vec3::new(HALF_EXTENT, HALF_EXTENT, HALF_EXTENT),
        (
            SAMPLES_PER_CHUNK_DIM_LARGE,
            SAMPLES_PER_CHUNK_DIM_LARGE,
            SAMPLES_PER_CHUNK_DIM_LARGE,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM_LARGE
        * SAMPLES_PER_CHUNK_DIM_LARGE
        * SAMPLES_PER_CHUNK_DIM_LARGE];
    c.bench_function("single_sphere_large_normal", |b| {
        b.iter(|| {
            let mut mesh_buffers = MeshBuffers::new();
            mc_mesh_generation(
                &mut mesh_buffers,
                &densities,
                &materials,
                SAMPLES_PER_CHUNK_DIM_LARGE,
                &NormalColorProvider,
                HALF_EXTENT,
            );
            black_box(mesh_buffers);
        });
    });
}

fn bench_bulk_spheres_small_normal(c: &mut Criterion) {
    let densities = SphereSampler::new(Vec3::ZERO, HALF_EXTENT).bake_quantized(
        Vec3::new(-HALF_EXTENT, -HALF_EXTENT, -HALF_EXTENT),
        Vec3::new(HALF_EXTENT, HALF_EXTENT, HALF_EXTENT),
        (
            SAMPLES_PER_CHUNK_DIM_SMALL,
            SAMPLES_PER_CHUNK_DIM_SMALL,
            SAMPLES_PER_CHUNK_DIM_SMALL,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM_SMALL
        * SAMPLES_PER_CHUNK_DIM_SMALL
        * SAMPLES_PER_CHUNK_DIM_SMALL];
    c.bench_function("bulk_spheres_small_10x_normal", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut mesh_buffers = MeshBuffers::new();
                mc_mesh_generation(
                    &mut mesh_buffers,
                    &densities,
                    &materials,
                    SAMPLES_PER_CHUNK_DIM_SMALL,
                    &NormalColorProvider,
                    HALF_EXTENT,
                );
                black_box(mesh_buffers);
            }
        });
    });
}

fn bench_bulk_spheres_large_normal(c: &mut Criterion) {
    let densities = SphereSampler::new(Vec3::ZERO, HALF_EXTENT).bake_quantized(
        Vec3::new(-HALF_EXTENT, -HALF_EXTENT, -HALF_EXTENT),
        Vec3::new(HALF_EXTENT, HALF_EXTENT, HALF_EXTENT),
        (
            SAMPLES_PER_CHUNK_DIM_LARGE,
            SAMPLES_PER_CHUNK_DIM_LARGE,
            SAMPLES_PER_CHUNK_DIM_LARGE,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM_LARGE
        * SAMPLES_PER_CHUNK_DIM_LARGE
        * SAMPLES_PER_CHUNK_DIM_LARGE];
    c.bench_function("bulk_spheres_large_10x_normal", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut mesh_buffers = MeshBuffers::new();
                mc_mesh_generation(
                    &mut mesh_buffers,
                    &densities,
                    &materials,
                    SAMPLES_PER_CHUNK_DIM_LARGE,
                    &NormalColorProvider,
                    HALF_EXTENT,
                );
                black_box(mesh_buffers);
            }
        });
    });
}

fn bench_single_cube_small_normal(c: &mut Criterion) {
    let densities = CuboidSampler::new(
        Vec3::ZERO,
        Vec3::new(HALF_EXTENT / 1.1, HALF_EXTENT / 1.1, HALF_EXTENT / 1.1),
    )
    .bake_quantized(
        Vec3::new(-HALF_EXTENT, -HALF_EXTENT, -HALF_EXTENT),
        Vec3::new(HALF_EXTENT, HALF_EXTENT, HALF_EXTENT),
        (
            SAMPLES_PER_CHUNK_DIM_SMALL,
            SAMPLES_PER_CHUNK_DIM_SMALL,
            SAMPLES_PER_CHUNK_DIM_SMALL,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM_SMALL
        * SAMPLES_PER_CHUNK_DIM_SMALL
        * SAMPLES_PER_CHUNK_DIM_SMALL];
    c.bench_function("single_cube_small_normal", |b| {
        b.iter(|| {
            let mut mesh_buffers = MeshBuffers::new();
            mc_mesh_generation(
                &mut mesh_buffers,
                &densities,
                &materials,
                SAMPLES_PER_CHUNK_DIM_SMALL,
                &NormalColorProvider,
                HALF_EXTENT,
            );
            black_box(mesh_buffers);
        });
    });
}

fn bench_single_cube_large_normal(c: &mut Criterion) {
    let densities = CuboidSampler::new(
        Vec3::ZERO,
        Vec3::new(HALF_EXTENT / 1.1, HALF_EXTENT / 1.1, HALF_EXTENT / 1.1),
    )
    .bake_quantized(
        Vec3::new(-HALF_EXTENT, -HALF_EXTENT, -HALF_EXTENT),
        Vec3::new(HALF_EXTENT, HALF_EXTENT, HALF_EXTENT),
        (
            SAMPLES_PER_CHUNK_DIM_LARGE,
            SAMPLES_PER_CHUNK_DIM_LARGE,
            SAMPLES_PER_CHUNK_DIM_LARGE,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM_LARGE
        * SAMPLES_PER_CHUNK_DIM_LARGE
        * SAMPLES_PER_CHUNK_DIM_LARGE];
    c.bench_function("single_cube_large_normal", |b| {
        b.iter(|| {
            let mut mesh_buffers = MeshBuffers::new();
            mc_mesh_generation(
                &mut mesh_buffers,
                &densities,
                &materials,
                SAMPLES_PER_CHUNK_DIM_LARGE,
                &NormalColorProvider,
                HALF_EXTENT,
            );
            black_box(mesh_buffers);
        });
    });
}

fn bench_bulk_cubes_small_normal(c: &mut Criterion) {
    let densities = CuboidSampler::new(
        Vec3::ZERO,
        Vec3::new(HALF_EXTENT / 1.1, HALF_EXTENT / 1.1, HALF_EXTENT / 1.1),
    )
    .bake_quantized(
        Vec3::new(-HALF_EXTENT, -HALF_EXTENT, -HALF_EXTENT),
        Vec3::new(HALF_EXTENT, HALF_EXTENT, HALF_EXTENT),
        (
            SAMPLES_PER_CHUNK_DIM_SMALL,
            SAMPLES_PER_CHUNK_DIM_SMALL,
            SAMPLES_PER_CHUNK_DIM_SMALL,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM_SMALL
        * SAMPLES_PER_CHUNK_DIM_SMALL
        * SAMPLES_PER_CHUNK_DIM_SMALL];
    c.bench_function("bulk_cubes_small_10x_normal", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut mesh_buffers = MeshBuffers::new();
                mc_mesh_generation(
                    &mut mesh_buffers,
                    &densities,
                    &materials,
                    SAMPLES_PER_CHUNK_DIM_SMALL,
                    &NormalColorProvider,
                    HALF_EXTENT,
                );
                black_box(mesh_buffers);
            }
        });
    });
}

fn bench_bulk_cubes_large_normal(c: &mut Criterion) {
    let densities = CuboidSampler::new(
        Vec3::ZERO,
        Vec3::new(HALF_EXTENT / 1.1, HALF_EXTENT / 1.1, HALF_EXTENT / 1.1),
    )
    .bake_quantized(
        Vec3::new(-HALF_EXTENT, -HALF_EXTENT, -HALF_EXTENT),
        Vec3::new(HALF_EXTENT, HALF_EXTENT, HALF_EXTENT),
        (
            SAMPLES_PER_CHUNK_DIM_LARGE,
            SAMPLES_PER_CHUNK_DIM_LARGE,
            SAMPLES_PER_CHUNK_DIM_LARGE,
        ),
    );
    let materials = [1; SAMPLES_PER_CHUNK_DIM_LARGE
        * SAMPLES_PER_CHUNK_DIM_LARGE
        * SAMPLES_PER_CHUNK_DIM_LARGE];
    c.bench_function("bulk_cubes_large_10x_normal", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut mesh_buffers = MeshBuffers::new();
                mc_mesh_generation(
                    &mut mesh_buffers,
                    &densities,
                    &materials,
                    SAMPLES_PER_CHUNK_DIM_LARGE,
                    &NormalColorProvider,
                    HALF_EXTENT,
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
