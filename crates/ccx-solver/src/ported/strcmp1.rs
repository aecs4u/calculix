//! Rust port of `strcmp1.c`.
//!
//! String comparison utility designed for comparing variable-length field (s1)
//! against a fixed string (s2).

use std::cmp::Ordering;

/// Compares two strings with special handling for null terminators.
///
/// This is a direct port of the C function `strcmp1` from the legacy CalculiX codebase.
/// It's designed to compare a variable-length field (s1) against a fixed string (s2).
/// When either string ends, the comparison terminates.
///
/// # Arguments
///
/// * `s1` - Variable-length field to compare
/// * `s2` - Fixed reference string to compare against
///
/// # Returns
///
/// - `Ordering::Equal` if the strings match
/// - `Ordering::Less` if s1 < s2
/// - `Ordering::Greater` if s1 > s2
///
/// # Examples
///
/// ```
/// use std::cmp::Ordering;
/// use ccx_solver::ported::strcmp1;
///
/// assert_eq!(strcmp1("hello", "hello"), Ordering::Equal);
/// assert_eq!(strcmp1("abc", "xyz"), Ordering::Less);
/// assert_eq!(strcmp1("xyz", "abc"), Ordering::Greater);
/// ```
pub fn strcmp1(s1: &str, s2: &str) -> Ordering {
    let bytes1 = s1.as_bytes();
    let bytes2 = s2.as_bytes();

    let mut i = 0;
    loop {
        let a = bytes1.get(i).copied();
        let b = bytes2.get(i).copied();

        match (a, b) {
            (None, _) | (_, None) => return Ordering::Equal,
            (Some(a_byte), Some(b_byte)) => {
                if a_byte != b_byte {
                    return a_byte.cmp(&b_byte);
                }
            }
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::strcmp1;
    use std::cmp::Ordering;

    #[test]
    fn identical_strings_are_equal() {
        assert_eq!(strcmp1("hello", "hello"), Ordering::Equal);
        assert_eq!(strcmp1("test", "test"), Ordering::Equal);
        assert_eq!(strcmp1("", ""), Ordering::Equal);
    }

    #[test]
    fn different_strings_ordered_correctly() {
        assert_eq!(strcmp1("abc", "xyz"), Ordering::Less);
        assert_eq!(strcmp1("xyz", "abc"), Ordering::Greater);
        assert_eq!(strcmp1("a", "b"), Ordering::Less);
        assert_eq!(strcmp1("b", "a"), Ordering::Greater);
    }

    #[test]
    fn prefix_matching_is_equal() {
        // s2 is fixed string - if s1 matches the prefix, it's equal
        assert_eq!(strcmp1("hello", "hel"), Ordering::Equal);
        assert_eq!(strcmp1("test", "testextra"), Ordering::Equal);
    }

    #[test]
    fn empty_string_comparisons() {
        assert_eq!(strcmp1("", "hello"), Ordering::Equal);
        assert_eq!(strcmp1("hello", ""), Ordering::Equal);
    }

    #[test]
    fn case_sensitive_comparison() {
        assert_eq!(strcmp1("Hello", "hello"), Ordering::Less); // 'H' < 'h' in ASCII
        assert_eq!(strcmp1("hello", "Hello"), Ordering::Greater);
    }
}
