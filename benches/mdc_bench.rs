use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::Vec3;
use isomesh::manifold_dual_contouring::mdc::{MeshBuffers, mdc_mesh_generation};
use isomesh::manifold_dual_contouring::sampler::{CuboidSampler, SphereSampler};

fn bench_single_sphere_small(c: &mut Criterion) {
    let resolution = 32;
    c.bench_function("single_sphere_small", |b| {
        b.iter(|| {
            let sphere = SphereSampler::new(black_box(Vec3::ZERO), black_box(10.0));
            let mut mesh_buffers = MeshBuffers::new();
            mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, sphere);
        });
    });
}

fn bench_single_sphere_large(c: &mut Criterion) {
    let resolution = 64;
    c.bench_function("single_sphere_large", |b| {
        b.iter(|| {
            let sphere = SphereSampler::new(black_box(Vec3::ZERO), black_box(30.0));
            let mut mesh_buffers = MeshBuffers::new();
            mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, sphere);
        });
    });
}

fn bench_bulk_spheres_small(c: &mut Criterion) {
    let resolution = 32;
    let radius = 10.0;
    c.bench_function("bulk_spheres_small_10x", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let sphere = SphereSampler::new(black_box(Vec3::ZERO), black_box(radius));
                let mut mesh_buffers = MeshBuffers::new();
                mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, sphere);
            }
        });
    });
}

fn bench_bulk_spheres_large(c: &mut Criterion) {
    let resolution = 64;
    let radius = 30.0;
    c.bench_function("bulk_spheres_large_10x", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let sphere = SphereSampler::new(black_box(Vec3::ZERO), black_box(radius));
                let mut mesh_buffers = MeshBuffers::new();
                mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, sphere);
                black_box(mesh_buffers);
            }
        });
    });
}

fn bench_single_cube_small(c: &mut Criterion) {
    let resolution = 32;
    let size = Vec3::new(2.0, 4.0, 8.0);
    c.bench_function("single_cube_small", |b| {
        b.iter(|| {
            let cube = CuboidSampler::new(black_box(Vec3::ZERO), black_box(size));
            let mut mesh_buffers = MeshBuffers::new();
            mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, cube);
            black_box(mesh_buffers);
        });
    });
}

fn bench_single_cube_large(c: &mut Criterion) {
    let resolution = 64;
    let size = Vec3::new(10.0, 15.0, 20.0);
    c.bench_function("single_cube_large", |b| {
        b.iter(|| {
            let cube = CuboidSampler::new(black_box(Vec3::ZERO), black_box(size));
            let mut mesh_buffers = MeshBuffers::new();
            mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, cube);
            black_box(mesh_buffers);
        });
    });
}

fn bench_bulk_cubes_small(c: &mut Criterion) {
    let resolution = 32;
    let size = Vec3::new(2.0, 4.0, 8.0);
    c.bench_function("bulk_cubes_small_10x", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let cube = CuboidSampler::new(Vec3::ZERO, black_box(size));
                let mut mesh_buffers = MeshBuffers::new();
                mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, cube);
                black_box(mesh_buffers);
            }
        });
    });
}

fn bench_bulk_cubes_large(c: &mut Criterion) {
    let resolution = 64;
    let size = Vec3::new(10.0, 15.0, 20.0);
    c.bench_function("bulk_cubes_large_10x", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let cube = CuboidSampler::new(Vec3::ZERO, black_box(size));
                let mut mesh_buffers = MeshBuffers::new();
                mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, cube);
                black_box(mesh_buffers);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_single_sphere_small,
    bench_single_sphere_large,
    bench_bulk_spheres_small,
    bench_bulk_spheres_large,
    bench_single_cube_small,
    bench_single_cube_large,
    bench_bulk_cubes_small,
    bench_bulk_cubes_large,
);

criterion_main!(benches);
