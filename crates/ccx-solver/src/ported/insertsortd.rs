//! Rust port of `insertsortd.f`.
//!
//! Simple insertion sort routine for very small arrays of doubles.
//! Based on <https://en.wikipedia.org/wiki/Insertion_sort>
//!
//! Original Author: Lukas Mayrhofer

/// In-place insertion sort for a slice of f64 values.
///
/// This is a direct port of the Fortran subroutine `insertsortd` from the legacy
/// CalculiX codebase. It implements a simple insertion sort algorithm which is
/// efficient for very small arrays (typically n < 10-20 elements).
///
/// # Arguments
///
/// * `dx` - Mutable slice of f64 values to sort in ascending order
///
/// # Examples
///
/// ```
/// use ccx_solver::ported::insertsortd;
///
/// let mut data = vec![3.0, 1.0, 4.0, 1.5, 9.0, 2.0];
/// insertsortd(&mut data);
/// assert_eq!(data, vec![1.0, 1.5, 2.0, 3.0, 4.0, 9.0]);
/// ```
pub fn insertsortd(dx: &mut [f64]) {
    let n = dx.len();
    if n <= 1 {
        return;
    }

    for i in 1..n {
        let xtmp = dx[i];
        let mut j = i;

        while j > 0 && xtmp < dx[j - 1] {
            dx[j] = dx[j - 1];
            j -= 1;
        }

        dx[j] = xtmp;
    }
}

#[cfg(test)]
mod tests {
    use super::insertsortd;

    #[test]
    fn sorts_simple_array() {
        let mut data = vec![3.0, 1.0, 4.0, 1.5, 9.0, 2.0];
        insertsortd(&mut data);
        assert_eq!(data, vec![1.0, 1.5, 2.0, 3.0, 4.0, 9.0]);
    }

    #[test]
    fn handles_already_sorted() {
        let mut data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        insertsortd(&mut data);
        assert_eq!(data, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn handles_reverse_sorted() {
        let mut data = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        insertsortd(&mut data);
        assert_eq!(data, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn handles_duplicates() {
        let mut data = vec![3.0, 1.0, 2.0, 1.0, 3.0, 2.0];
        insertsortd(&mut data);
        assert_eq!(data, vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
    }

    #[test]
    fn handles_empty_array() {
        let mut data: Vec<f64> = vec![];
        insertsortd(&mut data);
        assert_eq!(data, vec![]);
    }

    #[test]
    fn handles_single_element() {
        let mut data = vec![42.0];
        insertsortd(&mut data);
        assert_eq!(data, vec![42.0]);
    }

    #[test]
    fn handles_two_elements() {
        let mut data = vec![2.0, 1.0];
        insertsortd(&mut data);
        assert_eq!(data, vec![1.0, 2.0]);
    }

    #[test]
    fn handles_negative_numbers() {
        let mut data = vec![3.0, -1.0, 4.0, -5.0, 2.0];
        insertsortd(&mut data);
        assert_eq!(data, vec![-5.0, -1.0, 2.0, 3.0, 4.0]);
    }
}
