//! Rust ports of scalar helpers from `cgx_2.23/src`.

use std::f64::consts::PI;

pub fn check_if_number(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    let len = value.len();
    for (i, byte) in value.bytes().enumerate() {
        let is_digit = byte.is_ascii_digit();
        if is_digit {
            continue;
        }
        let c = byte as char;
        let allowed_basic = c == '.' || (c == '-' && i < len - 1) || c == '+';
        let allowed_exp = i > 0 && matches!(c, 'E' | 'D' | 'e' | 'd');
        if !(allowed_basic || allowed_exp) {
            return false;
        }
    }
    true
}

pub fn p_angle(x: f64, y: f64) -> f64 {
    if x > 0.0 && y >= 0.0 {
        (y / x).atan()
    } else if y > 0.0 && x <= 0.0 {
        (-x / y).atan() + PI * 0.5
    } else if x < 0.0 && y <= 0.0 {
        (-y / -x).atan() + PI
    } else if y < 0.0 && x >= 0.0 {
        (y / x).atan() + 2.0 * PI
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::{check_if_number, p_angle};

    #[test]
    fn check_if_number_accepts_legacy_patterns() {
        assert!(check_if_number("1.2"));
        assert!(check_if_number("-12"));
        assert!(check_if_number("+12"));
        assert!(check_if_number("1E-3"));
        assert!(check_if_number("1D+3"));
        assert!(!check_if_number(""));
        assert!(!check_if_number("1x2"));
    }

    #[test]
    fn p_angle_matches_quadrant_convention() {
        assert!((p_angle(1.0, 0.0) - 0.0).abs() < 1e-12);
        assert!((p_angle(0.0, 1.0) - PI * 0.5).abs() < 1e-12);
        assert!((p_angle(-1.0, 0.0) - PI).abs() < 1e-12);
        assert!((p_angle(0.0, -1.0) - PI * 1.5).abs() < 1e-12);
    }
}
