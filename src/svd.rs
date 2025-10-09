use crate::{mat3::Mat3, smat3::SMat3};
use glam::Vec3;

pub fn rotate01(vtav: &mut SMat3, v: &mut Mat3) {
    if vtav.m01 == 0.0 {
        return;
    }
    let mut c = 0.0;
    let mut s = 0.0;
    vtav.rot01(&mut c, &mut s);
    c = 0.0;
    s = 0.0;
    v.rot01_post(c, s);
}

pub fn rotate02(vtav: &mut SMat3, v: &mut Mat3) {
    if vtav.m02 == 0.0 {
        return;
    }
    let mut c = 0.0;
    let mut s = 0.0;
    vtav.rot02(&mut c, &mut s);
    c = 0.0;
    s = 0.0;
    v.rot02_post(c, s);
}

pub fn rotate12(vtav: &mut SMat3, v: &mut Mat3) {
    if vtav.m12 == 0.0 {
        return;
    }
    let mut c = 0.0;
    let mut s = 0.0;
    vtav.rot12(&mut c, &mut s);
    c = 0.0;
    s = 0.0;
    v.rot12_post(c, s);
}

pub fn get_symmetric_svd(a: &SMat3, vtav: &mut SMat3, v: &mut Mat3, tol: f32, max_sweeps: i32) {
    vtav.set_symmetric_from(a);
    v.set(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
    let delta = tol * vtav.fnorm();
    for _ in 0..max_sweeps {
        if vtav.off() <= delta {
            break;
        }
        rotate01(vtav, v);
        rotate02(vtav, v);
        rotate12(vtav, v);
    }
}

pub fn calc_error_mat(a: &Mat3, x: Vec3, b: Vec3) -> f32 {
    let vtmp = a.vmul(x);
    let vtmp = b - vtmp;
    vtmp.x * vtmp.x + vtmp.y * vtmp.y + vtmp.z * vtmp.z
}

pub fn calc_error_smat(orig_a: &SMat3, x: Vec3, b: Vec3) -> f32 {
    let mut a = Mat3::new();
    a.set_symmetric(orig_a);
    let vtmp = a.vmul(x);
    let vtmp = b - vtmp;
    vtmp.x * vtmp.x + vtmp.y * vtmp.y + vtmp.z * vtmp.z
}

pub fn pinv(x: f32, tol: f32) -> f32 {
    if x.abs() < tol || (1.0 / x).abs() < tol {
        0.0
    } else {
        1.0 / x
    }
}

pub fn pseudo_inverse(d: &SMat3, v: &Mat3, tol: f32) -> Mat3 {
    let mut m = Mat3::new();
    let d0 = pinv(d.m00, tol);
    let d1 = pinv(d.m11, tol);
    let d2 = pinv(d.m22, tol);
    m.set(
        v.m00 * d0 * v.m00 + v.m01 * d1 * v.m01 + v.m02 * d2 * v.m02,
        v.m00 * d0 * v.m10 + v.m01 * d1 * v.m11 + v.m02 * d2 * v.m12,
        v.m00 * d0 * v.m20 + v.m01 * d1 * v.m21 + v.m02 * d2 * v.m22,
        v.m10 * d0 * v.m00 + v.m11 * d1 * v.m01 + v.m12 * d2 * v.m02,
        v.m10 * d0 * v.m10 + v.m11 * d1 * v.m11 + v.m12 * d2 * v.m12,
        v.m10 * d0 * v.m20 + v.m11 * d1 * v.m21 + v.m12 * d2 * v.m22,
        v.m20 * d0 * v.m00 + v.m21 * d1 * v.m01 + v.m22 * d2 * v.m02,
        v.m20 * d0 * v.m10 + v.m21 * d1 * v.m11 + v.m22 * d2 * v.m12,
        v.m20 * d0 * v.m20 + v.m21 * d1 * v.m21 + v.m22 * d2 * v.m22,
    );
    m
}

pub fn solve_symmetric(
    a: &SMat3,
    b: &Vec3,
    x: &mut Vec3,
    svd_tol: f32,
    svd_sweeps: i32,
    pinv_tol: f32,
) -> f32 {
    let mut vtav = SMat3::new();
    let mut v = Mat3::new();
    get_symmetric_svd(a, &mut vtav, &mut v, svd_tol, svd_sweeps);
    let pinv = pseudo_inverse(&vtav, &v, pinv_tol);
    *x = pinv.vmul(*b);
    calc_error_smat(a, *x, *b)
}

pub fn solve_least_squares(
    a: &Mat3,
    b: Vec3,
    x: &mut Vec3,
    svd_tol: f32,
    svd_sweeps: i32,
    pinv_tol: f32,
) -> f32 {
    let at = a.transpose();
    let ata = a.mul_ata();
    let atb = at.vmul(b);
    solve_symmetric(&ata, &atb, x, svd_tol, svd_sweeps, pinv_tol)
}
