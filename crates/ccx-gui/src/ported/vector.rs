//! Rust ports of vector helpers from `cgx_2.23/src`.

pub type Vec3 = [f64; 3];

pub fn v_add(a: Vec3, b: Vec3) -> Vec3 {
    [b[0] + a[0], b[1] + a[1], b[2] + a[2]]
}

pub fn v_prod(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn v_sprod(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn v_result(a: Vec3, b: Vec3) -> Vec3 {
    [b[0] - a[0], b[1] - a[1], b[2] - a[2]]
}

pub fn v_norm(a: Vec3) -> (f64, Vec3) {
    let m = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    if m == 0.0 {
        (m, [0.0, 0.0, 0.0])
    } else {
        (m, [a[0] / m, a[1] / m, a[2] / m])
    }
}

pub fn v_angle(v0: Vec3, v1: Vec3) -> f64 {
    let (_, n0) = v_norm(v0);
    let (_, n1) = v_norm(v1);
    v_sprod(n0, n1).acos()
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::{v_add, v_angle, v_norm, v_prod, v_result, v_sprod};

    #[test]
    fn vector_ops_match_legacy_formulas() {
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        assert_eq!(v_add(a, b), [5.0, 7.0, 9.0]);
        assert_eq!(v_result(a, b), [3.0, 3.0, 3.0]);
        assert_eq!(v_sprod(a, b), 32.0);
        assert_eq!(v_prod(a, b), [-3.0, 6.0, -3.0]);
    }

    #[test]
    fn v_norm_handles_zero_vector() {
        let (m, n) = v_norm([0.0, 0.0, 0.0]);
        assert_eq!(m, 0.0);
        assert_eq!(n, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn v_angle_returns_expected_radians() {
        let angle = v_angle([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((angle - PI * 0.5).abs() < 1e-12);
    }
}
