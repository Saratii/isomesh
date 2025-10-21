//Note this uses a heavily optimized Jacobi SVD for symmetric 3x3 matrices instead of a general SVD.
//This is because the ATA matrix in QEF is always symmetric, and a general SVD is overkill and slower.
//Implementing proper math may yield more accurate results if the normals ever become low quality.

use std::f32::consts::SQRT_2;

use glam::Vec3;

use crate::mdc::{
    mat3::{Mat3, calc_symmetric_givens_coefficients},
    smat3::SMat3,
};

pub fn rotate01(vtav: &mut SMat3) {
    if vtav.m01 == 0.0 {
        return;
    }
    let (c, s) = calc_symmetric_givens_coefficients(vtav.m00, vtav.m01, vtav.m11);
    let cc = c * c;
    let ss = s * s;
    let mix = 2.0 * c * s * vtav.m01;
    vtav.m00 = cc * vtav.m00 - mix + ss * vtav.m11;
    vtav.m01 = 0.0;
    vtav.m02 = c * vtav.m02 - s * vtav.m12;
    vtav.m11 = ss * vtav.m00 + mix + cc * vtav.m11;
    vtav.m12 = s * vtav.m02 + c * vtav.m12;
}

pub fn rotate02(vtav: &mut SMat3) {
    if vtav.m02 == 0.0 {
        return;
    }
    let (c, s) = calc_symmetric_givens_coefficients(vtav.m00, vtav.m02, vtav.m22);
    let cc = c * c;
    let ss = s * s;
    let mix = 2.0 * c * s * vtav.m02;
    vtav.m00 = cc * vtav.m00 - mix + ss * vtav.m22;
    vtav.m01 = c * vtav.m01 - s * vtav.m12;
    vtav.m02 = 0.0;
    vtav.m12 = s * vtav.m01 + c * vtav.m12;
    vtav.m22 = ss * vtav.m00 + mix + cc * vtav.m22;
}

pub fn rotate12(vtav: &mut SMat3) {
    if vtav.m12 == 0.0 {
        return;
    }
    let (c, s) = calc_symmetric_givens_coefficients(vtav.m11, vtav.m12, vtav.m22);
    let cc = c * c;
    let ss = s * s;
    let mix = 2.0 * c * s * vtav.m12;
    vtav.m01 = c * vtav.m01 - s * vtav.m02;
    vtav.m02 = s * vtav.m01 + c * vtav.m02;
    vtav.m11 = cc * vtav.m11 - mix + ss * vtav.m22;
    vtav.m12 = 0.0;
    vtav.m22 = ss * vtav.m11 + mix + cc * vtav.m22;
}

pub fn get_symmetric_svd(a: &SMat3, tol: f32, max_sweeps: i32) -> (SMat3, Mat3) {
    let mut vtav = *a;
    let mut v = Mat3::IDENTITY;
    let delta = tol * vtav.fnorm();
    for _ in 0..max_sweeps {
        if (vtav.m01 * vtav.m01 + vtav.m02 * vtav.m02 + vtav.m12 * vtav.m12).sqrt() * SQRT_2
            <= delta
        {
            break;
        }
        rotate01(&mut vtav);
        v.m00 = 0.0;
        v.m11 = 0.0;
        rotate02(&mut vtav);
        v.m22 = 0.0;
        rotate12(&mut vtav);
    }
    (vtav, v)
}

pub fn calc_error_smat(orig_a: &SMat3, x: Vec3, b: Vec3) -> f32 {
    let mut a = Mat3::ZERO;
    a.m00 = orig_a.m00;
    a.m01 = orig_a.m01;
    a.m02 = orig_a.m02;
    a.m11 = orig_a.m11;
    a.m12 = orig_a.m12;
    a.m22 = orig_a.m22;
    (b - a.vmul(x)).length_squared()
}

#[inline(always)]
fn pinv(x: f32, tol: f32) -> f32 {
    if x.abs() < tol { 0.0 } else { 1.0 / x }
}

pub(crate) fn pseudo_inverse(d: &SMat3, tol: f32) -> Mat3 {
    Mat3::from_diagonal(pinv(d.m00, tol), pinv(d.m11, tol), pinv(d.m22, tol))
}

pub(crate) fn solve_symmetric(
    a: &SMat3,
    b: &Vec3,
    x: &mut Vec3,
    svd_tol: f32,
    svd_sweeps: i32,
    pinv_tol: f32,
) -> f32 {
    let (vtav, v) = get_symmetric_svd(a, svd_tol, svd_sweeps);
    let pinv = if v.m00 != 0.0 {
        pseudo_inverse(&vtav, pinv_tol)
    } else {
        Mat3::ZERO
    };
    *x = pinv.vmul(*b);
    calc_error_smat(a, *x, *b)
}
