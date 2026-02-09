//! Rust port of `compare.c`.
//!
//! String comparison utility that returns how many characters match from the beginning.

/// Compares two strings and returns the number of matching characters from the start.
///
/// This is a direct port of the C function `compare` from the legacy CalculiX codebase.
/// It compares two strings character by character until a difference is found or the
/// specified length is reached.
///
/// # Arguments
///
/// * `str1` - First string to compare
/// * `str2` - Second string to compare
/// * `length` - Maximum number of characters to compare
///
/// # Returns
///
/// The number of characters that matched from the beginning, up to `length`.
///
/// # Examples
///
/// ```
/// use ccx_solver::ported::compare;
///
/// assert_eq!(compare("hello", "hello", 5), 5);
/// assert_eq!(compare("hello", "help", 4), 3);
/// assert_eq!(compare("abc", "xyz", 3), 0);
/// ```
pub fn compare(str1: &str, str2: &str, length: usize) -> usize {
    let bytes1 = str1.as_bytes();
    let bytes2 = str2.as_bytes();

    let mut i = 0;
    while i < length && i < bytes1.len() && i < bytes2.len() && bytes1[i] == bytes2[i] {
        i += 1;
    }

    i
}

#[cfg(test)]
mod tests {
    use super::compare;

    #[test]
    fn identical_strings_match_fully() {
        assert_eq!(compare("hello", "hello", 5), 5);
        assert_eq!(compare("test", "test", 10), 4); // stops at actual length
    }

    #[test]
    fn different_strings_return_mismatch_position() {
        assert_eq!(compare("hello", "help", 5), 3);
        assert_eq!(compare("abc", "xyz", 3), 0);
        assert_eq!(compare("test123", "test456", 7), 4);
    }

    #[test]
    fn respects_length_parameter() {
        assert_eq!(compare("hello", "help", 2), 2);
        assert_eq!(compare("abc", "xyz", 0), 0);
    }

    #[test]
    fn handles_empty_strings() {
        assert_eq!(compare("", "", 5), 0);
        assert_eq!(compare("hello", "", 5), 0);
        assert_eq!(compare("", "hello", 5), 0);
    }

    #[test]
    fn handles_different_length_strings() {
        assert_eq!(compare("hi", "hello", 5), 1);
        assert_eq!(compare("hello", "hi", 5), 1);
    }
}
