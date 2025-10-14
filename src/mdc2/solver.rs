use glam::{Vec2, Vec3, Vec4};

#[derive(Clone)]
pub struct LevenQefSolver {
    svd_num_sweeps: i32,
    psuedo_inverse_threshold: f32,
}

impl LevenQefSolver {
    pub fn new() -> Self {
        LevenQefSolver {
            svd_num_sweeps: 10,
            psuedo_inverse_threshold: 0.1_f32,
        }
    }

    fn rsqrt(&self, a: f32) -> f32 {
        a.powf(-0.5_f32)
    }

    fn svd_mul_matrix_vec(&self, mat3x3_a: [[f32; 3]; 3], b: Vec4) -> Vec4 {
        let mut result = Vec4::ZERO;
        result.x = b.dot(Vec4::new(
            mat3x3_a[0][0],
            mat3x3_a[0][1],
            mat3x3_a[0][2],
            0.0,
        ));
        result.y = b.dot(Vec4::new(
            mat3x3_a[1][0],
            mat3x3_a[1][1],
            mat3x3_a[1][2],
            0.0,
        ));
        result.z = b.dot(Vec4::new(
            mat3x3_a[2][0],
            mat3x3_a[2][1],
            mat3x3_a[2][2],
            0.0,
        ));
        result.w = 0.0;
        result
    }

    fn givens_coeffs_sym(&self, a_pp: f32, a_pq: f32, a_qq: f32, cs: &mut Vec2) {
        if a_pq == 0.0_f32 {
            cs.x = 1.0_f32;
            cs.y = 0.0_f32;
            return;
        }
        let tau = (a_qq - a_pp) / (2.0_f32 * a_pq);
        let stt = (1.0_f32 + tau * tau).sqrt();
        let tan = 1.0_f32 / if tau >= 0.0_f32 { tau + stt } else { tau - stt };
        cs.x = self.rsqrt(1.0_f32 + tan * tan);
        cs.y = tan * (cs.x);
    }

    fn svd_rotate_xy(&self, xy: &mut Vec2, c: f32, s: f32) {
        let u = xy.x;
        let v = xy.y;
        xy.x = c * u - s * v;
        xy.y = s * u + c * v;
    }

    fn svd_rotateq_xy(&self, xya: &mut Vec3, c: f32, s: f32) {
        let cc = c * c;
        let ss = s * s;
        let mx = 2.0_f32 * c * s * (xya.z);
        let u = xya.x;
        let v = xya.y;
        xya.x = cc * u - mx + ss * v;
        xya.y = ss * u + mx + cc * v;
    }

    fn svd_rotate(
        &self,
        mat3x3_vtav: &mut [[f32; 3]; 3],
        mat3x3_v: &mut [[f32; 3]; 3],
        a: usize,
        b: usize,
    ) {
        if mat3x3_vtav[a][b] == 0.0_f32 {
            return;
        }

        let mut cs = Vec2::ZERO;
        self.givens_coeffs_sym(
            mat3x3_vtav[a][a],
            mat3x3_vtav[a][b],
            mat3x3_vtav[b][b],
            &mut cs,
        );

        let mut xyz = Vec3::new(mat3x3_vtav[a][a], mat3x3_vtav[b][b], mat3x3_vtav[a][b]);
        self.svd_rotateq_xy(&mut xyz, cs.x, cs.y);
        mat3x3_vtav[a][a] = xyz.x;
        mat3x3_vtav[b][b] = xyz.y;
        mat3x3_vtav[a][b] = xyz.z;

        let mut xy = Vec2::new(
            mat3x3_vtav[0usize][3usize - b],
            mat3x3_vtav[1usize - a][2usize],
        );
        self.svd_rotate_xy(&mut xy, cs.x, cs.y);
        mat3x3_vtav[0usize][3usize - b] = xy.x;
        mat3x3_vtav[1usize - a][2usize] = xy.y;

        mat3x3_vtav[a][b] = 0.0_f32;

        let mut xy = Vec2::new(mat3x3_v[0][a], mat3x3_v[0][b]);
        self.svd_rotate_xy(&mut xy, cs.x, cs.y);
        mat3x3_v[0][a] = xy.x;
        mat3x3_v[0][b] = xy.y;

        let mut xy = Vec2::new(mat3x3_v[1][a], mat3x3_v[1][b]);
        self.svd_rotate_xy(&mut xy, cs.x, cs.y);
        mat3x3_v[1][a] = xy.x;
        mat3x3_v[1][b] = xy.y;

        let mut xy = Vec2::new(mat3x3_v[2][a], mat3x3_v[2][b]);
        self.svd_rotate_xy(&mut xy, cs.x, cs.y);
        mat3x3_v[2][a] = xy.x;
        mat3x3_v[2][b] = xy.y;
    }

    fn svd_solve_sym(
        &self,
        mat3x3_tri_a: &[f32; 6],
        sigma: &mut Vec4,
        mat3x3_v: &mut [[f32; 3]; 3],
    ) {
        let mut mat3x3_vtav = [[0.0_f32; 3]; 3];
        mat3x3_vtav[0][0] = mat3x3_tri_a[0];
        mat3x3_vtav[0][1] = mat3x3_tri_a[1];
        mat3x3_vtav[0][2] = mat3x3_tri_a[2];
        mat3x3_vtav[1][0] = 0.0_f32;
        mat3x3_vtav[1][1] = mat3x3_tri_a[3];
        mat3x3_vtav[1][2] = mat3x3_tri_a[4];
        mat3x3_vtav[2][0] = 0.0_f32;
        mat3x3_vtav[2][1] = 0.0_f32;
        mat3x3_vtav[2][2] = mat3x3_tri_a[5];
        for _ in 0..self.svd_num_sweeps {
            self.svd_rotate(&mut mat3x3_vtav, mat3x3_v, 0, 1);
            self.svd_rotate(&mut mat3x3_vtav, mat3x3_v, 0, 2);
            self.svd_rotate(&mut mat3x3_vtav, mat3x3_v, 1, 2);
        }
        sigma.x = mat3x3_vtav[0][0];
        sigma.y = mat3x3_vtav[1][1];
        sigma.z = mat3x3_vtav[2][2];
        sigma.w = 0.0_f32;
    }

    fn svd_invdet(&self, x: f32, tol: f32) -> f32 {
        if x.abs() < tol || (1.0_f32 / x).abs() < tol {
            0.0_f32
        } else {
            1.0_f32 / x
        }
    }

    fn svd_pseudoinverse(&self, sigma: &Vec4, mat3x3_v: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
        let d0 = self.svd_invdet(sigma.x, self.psuedo_inverse_threshold);
        let d1 = self.svd_invdet(sigma.y, self.psuedo_inverse_threshold);
        let d2 = self.svd_invdet(sigma.z, self.psuedo_inverse_threshold);
        let mut mat3x3_o = [[0.0_f32; 3]; 3];
        mat3x3_o[0][0] = mat3x3_v[0][0] * d0 * mat3x3_v[0][0]
            + mat3x3_v[0][1] * d1 * mat3x3_v[0][1]
            + mat3x3_v[0][2] * d2 * mat3x3_v[0][2];
        mat3x3_o[0][1] = mat3x3_v[0][0] * d0 * mat3x3_v[1][0]
            + mat3x3_v[0][1] * d1 * mat3x3_v[1][1]
            + mat3x3_v[0][2] * d2 * mat3x3_v[1][2];
        mat3x3_o[0][2] = mat3x3_v[0][0] * d0 * mat3x3_v[2][0]
            + mat3x3_v[0][1] * d1 * mat3x3_v[2][1]
            + mat3x3_v[0][2] * d2 * mat3x3_v[2][2];
        mat3x3_o[1][0] = mat3x3_v[1][0] * d0 * mat3x3_v[0][0]
            + mat3x3_v[1][1] * d1 * mat3x3_v[0][1]
            + mat3x3_v[1][2] * d2 * mat3x3_v[0][2];
        mat3x3_o[1][1] = mat3x3_v[1][0] * d0 * mat3x3_v[1][0]
            + mat3x3_v[1][1] * d1 * mat3x3_v[1][1]
            + mat3x3_v[1][2] * d2 * mat3x3_v[1][2];
        mat3x3_o[1][2] = mat3x3_v[1][0] * d0 * mat3x3_v[2][0]
            + mat3x3_v[1][1] * d1 * mat3x3_v[2][1]
            + mat3x3_v[1][2] * d2 * mat3x3_v[2][2];
        mat3x3_o[2][0] = mat3x3_v[2][0] * d0 * mat3x3_v[0][0]
            + mat3x3_v[2][1] * d1 * mat3x3_v[0][1]
            + mat3x3_v[2][2] * d2 * mat3x3_v[0][2];
        mat3x3_o[2][1] = mat3x3_v[2][0] * d0 * mat3x3_v[1][0]
            + mat3x3_v[2][1] * d1 * mat3x3_v[1][1]
            + mat3x3_v[2][2] * d2 * mat3x3_v[1][2];
        mat3x3_o[2][2] = mat3x3_v[2][0] * d0 * mat3x3_v[2][0]
            + mat3x3_v[2][1] * d1 * mat3x3_v[2][1]
            + mat3x3_v[2][2] * d2 * mat3x3_v[2][2];
        mat3x3_o
    }

    fn svd_solve_ata_atb(&self, mat3x3_tri_ata: &[f32; 6], atb: Vec4) -> Vec4 {
        let mut mat3x3_v = [[0.0_f32; 3]; 3];
        mat3x3_v[0][0] = 1.0_f32;
        mat3x3_v[0][1] = 0.0_f32;
        mat3x3_v[0][2] = 0.0_f32;
        mat3x3_v[1][0] = 0.0_f32;
        mat3x3_v[1][1] = 1.0_f32;
        mat3x3_v[1][2] = 0.0_f32;
        mat3x3_v[2][0] = 0.0_f32;
        mat3x3_v[2][1] = 0.0_f32;
        mat3x3_v[2][2] = 1.0_f32;
        let mut sigma = Vec4::ZERO;
        self.svd_solve_sym(mat3x3_tri_ata, &mut sigma, &mut mat3x3_v);
        let mat3x3_vinv = self.svd_pseudoinverse(&sigma, &mat3x3_v);
        self.svd_mul_matrix_vec(mat3x3_vinv, atb)
    }

    fn svd_vmul_sym(&self, mat3x3_tri_a: &[f32; 6], v: Vec4) -> Vec4 {
        let a_row_x = Vec4::new(mat3x3_tri_a[0], mat3x3_tri_a[1], mat3x3_tri_a[2], 0.0);
        let mut result = Vec4::ZERO;
        result.x = v.dot(a_row_x);
        result.y = mat3x3_tri_a[1] * v.x + mat3x3_tri_a[3] * v.y + mat3x3_tri_a[4] * v.z;
        result.z = mat3x3_tri_a[2] * v.x + mat3x3_tri_a[4] * v.y + mat3x3_tri_a[5] * v.z;
        result
    }

    pub fn solve(&self, mat3x3_tri_ata: &[f32; 6], atb: Vec4, masspoint: Vec4) -> Vec4 {
        let mut masspoint = masspoint;
        masspoint = masspoint / masspoint.w.max(1.0_f32);
        let mut a_mp = self.svd_vmul_sym(mat3x3_tri_ata, masspoint);
        a_mp = atb - a_mp;
        let solved_pos = self.svd_solve_ata_atb(mat3x3_tri_ata, a_mp);
        let solved_pos = solved_pos + masspoint;
        solved_pos
    }
}
