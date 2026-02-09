//! Rust port of `cident.f`.

use std::cmp::Ordering;

pub fn cident<S: AsRef<str>>(ordered: &[S], probe: &str) -> usize {
    let n = ordered.len();
    let mut id = 0usize;
    if n == 0 {
        return id;
    }
    let mut n2 = n + 1;

    loop {
        let m = (n2 + id) / 2;
        if fortran_cmp(probe, ordered[m - 1].as_ref()) != Ordering::Less {
            id = m;
        } else {
            n2 = m;
        }
        if n2 - id == 1 {
            return id;
        }
    }
}

fn fortran_cmp(lhs: &str, rhs: &str) -> Ordering {
    let lhs_bytes = lhs.as_bytes();
    let rhs_bytes = rhs.as_bytes();
    let width = lhs_bytes.len().max(rhs_bytes.len());

    for i in 0..width {
        let l = *lhs_bytes.get(i).unwrap_or(&b' ');
        let r = *rhs_bytes.get(i).unwrap_or(&b' ');
        match l.cmp(&r) {
            Ordering::Equal => {}
            other => return other,
        }
    }
    Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::cident;

    #[test]
    fn returns_fortran_style_insertion_index() {
        let ordered = vec!["A", "C", "E"];
        assert_eq!(cident(&ordered, ""), 0);
        assert_eq!(cident(&ordered, "A"), 1);
        assert_eq!(cident(&ordered, "B"), 1);
        assert_eq!(cident(&ordered, "C"), 2);
        assert_eq!(cident(&ordered, "Z"), 3);
    }

    #[test]
    fn treats_trailing_spaces_like_fortran_character_fields() {
        let ordered = vec!["AB", "AB  ", "AC"];
        assert_eq!(cident(&ordered, "AB"), 2);
        assert_eq!(cident(&ordered, "AB "), 2);
        assert_eq!(cident(&ordered, "AB   "), 2);
    }

    #[test]
    fn handles_empty_input_list() {
        let ordered: Vec<&str> = Vec::new();
        assert_eq!(cident(&ordered, "ANY"), 0);
    }

    #[test]
    fn preserves_insertion_point_between_neighbors() {
        let ordered = vec!["A", "D", "G"];
        assert_eq!(cident(&ordered, "C"), 1);
        assert_eq!(cident(&ordered, "F"), 2);
    }
}
