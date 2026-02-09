//! Rust port of `nident.f` and `nident2.f`.
//!
//! Binary search functions for finding insertion positions in ordered integer arrays.

/// Finds the position id of px in an ordered array of integers.
///
/// This is a direct port of the Fortran subroutine `nident` from the legacy
/// CalculiX codebase. It performs a binary search to find the position where:
/// - `x[id] <= px` and `x[id+1] > px`
///
/// # Arguments
///
/// * `x` - Ordered slice of integers (ascending order)
/// * `px` - Value to search for
///
/// # Returns
///
/// The index `id` such that `x[id] <= px` and `x[id+1] > px`.
/// Returns 0 if px is less than all elements.
///
/// # Examples
///
/// ```
/// use ccx_solver::ported::nident;
///
/// let x = vec![1, 3, 5, 7, 9];
/// assert_eq!(nident(&x, 0), 0);  // Before all elements
/// assert_eq!(nident(&x, 1), 1);  // Equals x[0]
/// assert_eq!(nident(&x, 4), 2);  // Between x[1] and x[2]
/// assert_eq!(nident(&x, 10), 5); // After all elements
/// ```
pub fn nident(x: &[i32], px: i32) -> usize {
    let n = x.len();
    if n == 0 {
        return 0;
    }

    let mut id = 0;
    let mut n2 = n + 1;

    loop {
        let m = (n2 + id) / 2;
        if px >= x[m - 1] {
            id = m;
        } else {
            n2 = m;
        }
        if n2 - id == 1 {
            return id;
        }
    }
}

/// Finds the position id of px in an ordered 2D array of integers (searching first column).
///
/// This is a direct port of the Fortran subroutine `nident2` from the legacy
/// CalculiX codebase. The array x has shape (2, n) and the search is performed
/// on the first row: `x[0][id] <= px` and `x[0][id+1] > px`
///
/// # Arguments
///
/// * `x` - Slice of tuples (key, value) where keys are ordered ascending
/// * `px` - Key value to search for
///
/// # Returns
///
/// The index `id` such that `x[id].0 <= px` and `x[id+1].0 > px`.
/// Returns 0 if px is less than all elements.
///
/// # Examples
///
/// ```
/// use ccx_solver::ported::nident2;
///
/// let x = vec![(1, 100), (3, 200), (5, 300), (7, 400)];
/// assert_eq!(nident2(&x, 0), 0);  // Before all elements
/// assert_eq!(nident2(&x, 3), 2);  // Equals x[1].0
/// assert_eq!(nident2(&x, 4), 2);  // Between x[1].0 and x[2].0
/// assert_eq!(nident2(&x, 10), 4); // After all elements
/// ```
pub fn nident2(x: &[(i32, i32)], px: i32) -> usize {
    let n = x.len();
    if n == 0 {
        return 0;
    }

    let mut id = 0;
    let mut n2 = n + 1;

    loop {
        let m = (n2 + id) / 2;
        if px >= x[m - 1].0 {
            id = m;
        } else {
            n2 = m;
        }
        if n2 - id == 1 {
            return id;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{nident, nident2};

    #[test]
    fn nident_empty_array() {
        let x: Vec<i32> = vec![];
        assert_eq!(nident(&x, 5), 0);
    }

    #[test]
    fn nident_single_element() {
        let x = vec![5];
        assert_eq!(nident(&x, 3), 0); // Before
        assert_eq!(nident(&x, 5), 1); // Equal
        assert_eq!(nident(&x, 7), 1); // After
    }

    #[test]
    fn nident_finds_correct_positions() {
        let x = vec![1, 3, 5, 7, 9];
        assert_eq!(nident(&x, 0), 0); // Before first
        assert_eq!(nident(&x, 1), 1); // Equal to first
        assert_eq!(nident(&x, 2), 1); // Between first and second
        assert_eq!(nident(&x, 3), 2); // Equal to second
        assert_eq!(nident(&x, 4), 2); // Between second and third
        assert_eq!(nident(&x, 5), 3); // Equal to third
        assert_eq!(nident(&x, 9), 5); // Equal to last
        assert_eq!(nident(&x, 10), 5); // After last
    }

    #[test]
    fn nident_duplicates() {
        let x = vec![1, 3, 3, 3, 5, 7];
        assert_eq!(nident(&x, 3), 4); // Should return last occurrence + 1
    }

    #[test]
    fn nident2_empty_array() {
        let x: Vec<(i32, i32)> = vec![];
        assert_eq!(nident2(&x, 5), 0);
    }

    #[test]
    fn nident2_single_element() {
        let x = vec![(5, 100)];
        assert_eq!(nident2(&x, 3), 0); // Before
        assert_eq!(nident2(&x, 5), 1); // Equal
        assert_eq!(nident2(&x, 7), 1); // After
    }

    #[test]
    fn nident2_finds_correct_positions() {
        let x = vec![(1, 100), (3, 200), (5, 300), (7, 400), (9, 500)];
        assert_eq!(nident2(&x, 0), 0); // Before first
        assert_eq!(nident2(&x, 1), 1); // Equal to first key
        assert_eq!(nident2(&x, 2), 1); // Between first and second keys
        assert_eq!(nident2(&x, 3), 2); // Equal to second key
        assert_eq!(nident2(&x, 5), 3); // Equal to third key
        assert_eq!(nident2(&x, 10), 5); // After last key
    }

    #[test]
    fn nident2_ignores_second_value() {
        let x = vec![(1, 999), (3, 111), (5, 777)];
        // Search only uses first value in tuple
        assert_eq!(nident2(&x, 3), 2);
        assert_eq!(nident2(&x, 4), 2);
    }

    #[test]
    fn nident2_duplicates() {
        let x = vec![(1, 10), (3, 20), (3, 30), (3, 40), (5, 50)];
        assert_eq!(nident2(&x, 3), 4); // Should return last occurrence + 1
    }
}
