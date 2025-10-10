use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::Vec3;
use isomesh::mdc::mdc::{MeshBuffers, mdc_mesh_generation};
use isomesh::mdc::sampler::{CuboidSampler, SphereSampler};

fn bench_single_sphere_small(c: &mut Criterion) {
    c.bench_function("single_sphere_small", |b| {
        b.iter(|| {
            let resolution = 32;
            let sphere = SphereSampler::new(black_box(Vec3::new(0.0, 0.0, 0.0)), black_box(10.0));
            let mut mesh_buffers = MeshBuffers::new();
            mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, &sphere);
        });
    });
}

fn bench_single_sphere_large(c: &mut Criterion) {
    c.bench_function("single_sphere_large", |b| {
        b.iter(|| {
            let resolution = 64;
            let sphere = SphereSampler::new(black_box(Vec3::new(0.0, 0.0, 0.0)), black_box(30.0));
            let mut mesh_buffers = MeshBuffers::new();
            mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, &sphere);
        });
    });
}

fn bench_bulk_spheres_small(c: &mut Criterion) {
    c.bench_function("bulk_spheres_small_10x", |b| {
        b.iter(|| {
            let resolution = 32;
            let radius = 10.0;
            for i in 0..100 {
                let offset = (i as f32) * 10.0;
                let sphere =
                    SphereSampler::new(black_box(Vec3::new(offset, 0.0, 0.0)), black_box(radius));
                let mut mesh_buffers = MeshBuffers::new();
                mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, &sphere);
            }
        });
    });
}

fn bench_bulk_spheres_large(c: &mut Criterion) {
    c.bench_function("bulk_spheres_large_10x", |b| {
        b.iter(|| {
            let resolution = 64;
            let radius = 30.0;
            for i in 0..100 {
                let offset = (i as f32) * 15.0;
                let sphere =
                    SphereSampler::new(black_box(Vec3::new(offset, 0.0, 0.0)), black_box(radius));
                let mut mesh_buffers = MeshBuffers::new();
                mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, &sphere);
                black_box(mesh_buffers);
            }
        });
    });
}

fn bench_single_cube_small(c: &mut Criterion) {
    c.bench_function("single_cube_small", |b| {
        b.iter(|| {
            let resolution = 32;
            let size = Vec3::new(2.0, 4.0, 8.0);
            let cube = CuboidSampler::new(black_box(Vec3::new(0.0, 0.0, 0.0)), black_box(size));
            let mut mesh_buffers = MeshBuffers::new();
            mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, &cube);
            black_box(mesh_buffers);
        });
    });
}

fn bench_single_cube_large(c: &mut Criterion) {
    c.bench_function("single_cube_large", |b| {
        b.iter(|| {
            let resolution = 64;
            let size = Vec3::new(10.0, 15.0, 20.0);
            let cube = CuboidSampler::new(black_box(Vec3::new(0.0, 0.0, 0.0)), black_box(size));
            let mut mesh_buffers = MeshBuffers::new();
            mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, &cube);
            black_box(mesh_buffers);
        });
    });
}

fn bench_bulk_cubes_small(c: &mut Criterion) {
    c.bench_function("bulk_cubes_small_10x", |b| {
        b.iter(|| {
            let resolution = 32;
            let size = Vec3::new(2.0, 4.0, 8.0);
            for i in 0..10 {
                let offset = Vec3::new((i as f32) * 50.0, 0.0, 0.0);
                let cube = CuboidSampler::new(black_box(offset), black_box(size));
                let mut mesh_buffers = MeshBuffers::new();
                mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, &cube);
                black_box(mesh_buffers);
            }
        });
    });
}

fn bench_bulk_cubes_large(c: &mut Criterion) {
    c.bench_function("bulk_cubes_large_10x", |b| {
        b.iter(|| {
            let resolution = 64;
            let size = Vec3::new(10.0, 15.0, 20.0);
            for i in 0..10 {
                let offset = Vec3::new((i as f32) * 75.0, 0.0, 0.0);
                let cube = CuboidSampler::new(black_box(offset), black_box(size));
                let mut mesh_buffers = MeshBuffers::new();
                mdc_mesh_generation(0.5, &mut mesh_buffers, false, resolution, true, &cube);
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
