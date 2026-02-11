//! Rust ports of `strcmp2.c`, `strcpy1.c`, and `strcpy2.c` from CalculiX 2.23
//!
//! String comparison and copying utilities with special null-handling semantics.
//! These functions maintain compatibility with the legacy C implementations.

/// Compares up to `length` characters of two strings.
///
/// This is a port of `strcmp2.c` from CalculiX. It compares the first `length` characters
/// of `s1` and `s2`, stopping early if either string terminates.
///
/// **Behavior**: If either string ends before `length` characters, both are treated
/// as equal at that point (returns 0).
///
/// # Arguments
///
/// * `s1` - First string to compare
/// * `s2` - Second string to compare
/// * `length` - Maximum number of characters to compare
///
/// # Returns
///
/// * `0` if strings are equal up to `length` or either terminates early
/// * Positive if `s1 > s2` at first difference
/// * Negative if `s1 < s2` at first difference
///
/// # Examples
///
/// ```
/// use ccx_solver::ported::strcmp2;
///
/// assert_eq!(strcmp2("hello", "hello", 5), 0);
/// assert_eq!(strcmp2("hello", "help", 3), 0);  // First 3 chars match
/// assert!(strcmp2("hello", "help", 4) != 0);   // 4th char differs
/// ```
pub fn strcmp2(s1: &str, s2: &str, length: usize) -> i32 {
    let bytes1 = s1.as_bytes();
    let bytes2 = s2.as_bytes();

    let mut i = 0;
    let (mut a, mut b);

    // Mimic C do-while loop
    loop {
        // Get next byte from each string (0 if past end)
        a = bytes1.get(i).copied().unwrap_or(0);
        b = bytes2.get(i).copied().unwrap_or(0);

        // If either string ends, treat both as '\0' (equal)
        if b == 0 {
            a = 0;
            b = 0;
            break;
        }
        if a == 0 {
            a = 0;
            b = 0;
            break;
        }

        i += 1;

        // Continue while characters match AND we haven't reached length
        if a != b || i >= length {
            break;
        }
    }

    (a as i32) - (b as i32)
}

/// Copies up to `length` characters from `s2` to `s1`, padding with spaces after null.
///
/// This is a port of `strcpy1.c` from CalculiX. It copies characters from `src` until
/// a null terminator is encountered, then fills the rest with spaces up to `length`.
///
/// **Note**: This function returns a String instead of modifying in-place (Rust idiom).
///
/// # Arguments
///
/// * `src` - Source string to copy from
/// * `length` - Total length of output (will be padded with spaces)
///
/// # Returns
///
/// A `String` of exactly `length` characters: `src` content + spaces
///
/// # Examples
///
/// ```
/// use ccx_solver::ported::strcpy1;
///
/// assert_eq!(strcpy1("hello", 10), "hello     ");
/// assert_eq!(strcpy1("test", 4), "test");
/// assert_eq!(strcpy1("", 5), "     ");
/// ```
pub fn strcpy1(src: &str, length: usize) -> String {
    let bytes = src.as_bytes();
    let mut result = String::with_capacity(length);

    let mut blank = false;
    for i in 0..length {
        if !blank && i < bytes.len() {
            let b = bytes[i];
            if b == 0 {
                blank = true;
            } else {
                result.push(b as char);
                continue;
            }
        }
        result.push(' ');
    }

    result
}

/// Copies up to `length` characters from `s2` to `s1`, stopping at null terminator.
///
/// This is a port of `strcpy2.c` from CalculiX. Unlike `strcpy1`, this stops
/// immediately after copying the null terminator (if encountered).
///
/// **Note**: This function returns a String instead of modifying in-place (Rust idiom).
///
/// # Arguments
///
/// * `src` - Source string to copy from
/// * `length` - Maximum number of characters to copy
///
/// # Returns
///
/// A `String` containing up to `length` characters from `src`, stopping at null if present
///
/// # Examples
///
/// ```
/// use ccx_solver::ported::strcpy2;
///
/// assert_eq!(strcpy2("hello world", 5), "hello");
/// assert_eq!(strcpy2("test", 10), "test");
/// assert_eq!(strcpy2("", 5), "");
/// ```
pub fn strcpy2(src: &str, length: usize) -> String {
    let bytes = src.as_bytes();
    let mut result = String::with_capacity(length.min(src.len()));

    for i in 0..length.min(bytes.len()) {
        let b = bytes[i];
        if b == 0 {
            break;
        }
        result.push(b as char);
    }

    result
}

/// Extracts a substring from position `a` to `b` (1-based, inclusive).
///
/// This is a port of `stos()` from CalculiX. It extracts characters from
/// position `a-1` to position `b-1` (using 1-based indexing like Fortran/C convention).
///
/// **Safety**: Bounds-checked. Will not panic if indices are out of range.
///
/// # Arguments
///
/// * `string` - Source string
/// * `a` - Start position (1-based, inclusive)
/// * `b` - End position (1-based, inclusive)
///
/// # Returns
///
/// Extracted substring, or empty string if indices are invalid
///
/// # Examples
///
/// ```
/// use ccx_solver::ported::stos;
///
/// assert_eq!(stos("Hello World", 1, 5), "Hello");
/// assert_eq!(stos("Testing", 2, 4), "est");
/// assert_eq!(stos("Short", 1, 10), "Short");  // Truncates at end
/// ```
pub fn stos(string: &str, a: usize, b: usize) -> String {
    if a == 0 || b == 0 || a > b {
        return String::new();
    }

    let bytes = string.as_bytes();
    let start = (a - 1).min(bytes.len());
    let end = b.min(bytes.len());

    if start >= end {
        return String::new();
    }

    // Convert bytes to string, handling UTF-8
    match std::str::from_utf8(&bytes[start..end]) {
        Ok(s) => s.to_string(),
        Err(_) => String::new(),
    }
}

/// Writes a substring into a buffer at positions `a` to `b` (1-based).
///
/// This is a port of `stos_inv()` from CalculiX. It writes characters from
/// `source` into specified positions of a target buffer.
///
/// **Note**: Returns a new String rather than modifying in-place (Rust idiom).
///
/// # Arguments
///
/// * `source` - String to copy from
/// * `a` - Start position in target (1-based)
/// * `b` - End position in target (1-based)
/// * `target_len` - Total length of target buffer
///
/// # Returns
///
/// A string of length `target_len` with `source` written at positions [a, b)
///
/// # Examples
///
/// ```
/// use ccx_solver::ported::stos_inv;
///
/// assert_eq!(stos_inv("ABC", 3, 5, 10), "  ABC     ");
/// assert_eq!(stos_inv("Test", 1, 4, 6), "Test  ");
/// ```
pub fn stos_inv(source: &str, a: usize, b: usize, target_len: usize) -> String {
    let mut result = vec![b' '; target_len];  // Fill with spaces

    if a == 0 || b == 0 || a > b || a > target_len {
        return String::from_utf8_lossy(&result).to_string();
    }

    let src_bytes = source.as_bytes();
    let start = a - 1;  // Convert to 0-based
    let end = b.min(target_len);

    let copy_len = (end - start).min(src_bytes.len());

    for i in 0..copy_len {
        if start + i >= result.len() {
            break;
        }
        result[start + i] = src_bytes[i];
    }

    String::from_utf8_lossy(&result).to_string()
}

/// Splits a string by a delimiter character, respecting quoted sections.
///
/// This is a port of `strsplt()` from CalculiX. It splits the input string
/// at each occurrence of `delimiter`, but skips delimiters inside quoted sections.
///
/// **Features**:
/// - Splits at `delimiter` character
/// - Respects double-quoted sections (doesn't split inside quotes)
/// - Strips leading/trailing spaces from tokens
/// - Skips empty tokens
///
/// # Arguments
///
/// * `input` - String to split
/// * `delimiter` - Delimiter character (e.g., ',', ' ', '\t')
///
/// # Returns
///
/// Vector of tokens (non-empty substrings)
///
/// # Examples
///
/// ```
/// use ccx_solver::ported::strsplt;
///
/// assert_eq!(strsplt("one,two,three", ','), vec!["one", "two", "three"]);
/// assert_eq!(strsplt("a, b, c", ','), vec!["a", "b", "c"]);
/// assert_eq!(strsplt("hello, \"world, test\", end", ','),
///            vec!["hello", "world, test", "end"]);
/// ```
pub fn strsplt(input: &str, delimiter: char) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_token = String::new();
    let mut inside_quotes = false;

    for ch in input.chars() {
        match ch {
            '\n' | '\0' => break,  // Stop at newline or null
            '"' => {
                inside_quotes = !inside_quotes;
                // Don't include the quotes in the output
            }
            c if c == delimiter && !inside_quotes => {
                // Hit delimiter outside quotes - save token if non-empty
                let trimmed = current_token.trim().to_string();
                if !trimmed.is_empty() {
                    result.push(trimmed);
                }
                current_token.clear();
            }
            ' ' if !inside_quotes && current_token.is_empty() => {
                // Skip leading spaces outside quotes
            }
            c => {
                current_token.push(c);
            }
        }
    }

    // Add final token if non-empty
    let trimmed = current_token.trim().to_string();
    if !trimmed.is_empty() {
        result.push(trimmed);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== strcmp2 tests ==========

    #[test]
    fn strcmp2_identical_strings() {
        assert_eq!(strcmp2("hello", "hello", 5), 0);
        assert_eq!(strcmp2("test", "test", 4), 0);
    }

    #[test]
    fn strcmp2_different_strings() {
        // 'o' > 'p' by ASCII value
        assert!(strcmp2("hello", "help", 5) < 0);
        // 'b' > 'a'
        assert!(strcmp2("abc", "abd", 3) < 0);
    }

    #[test]
    fn strcmp2_partial_match() {
        // First 3 characters match ("hel")
        assert_eq!(strcmp2("hello", "help", 3), 0);
        // First 2 characters match ("he")
        assert_eq!(strcmp2("hello", "hexagon", 2), 0);
    }

    #[test]
    fn strcmp2_early_termination() {
        // When comparing "hello" and "hi", they differ at position 1 ('e' vs 'i')
        // So this returns the difference: 'e' - 'i' = -4
        assert!(strcmp2("hello", "hi", 5) < 0);

        // But if the shorter string matches the prefix exactly:
        // Comparing "hel" with "hello" up to 5 chars
        // At position 3, "hel" ends (gets '\0'), so both treated as '\0'
        assert_eq!(strcmp2("hel", "hello", 5), 0);

        // And vice versa:
        assert_eq!(strcmp2("hello", "hel", 5), 0);
    }

    #[test]
    fn strcmp2_empty_strings() {
        assert_eq!(strcmp2("", "", 5), 0);
        assert_eq!(strcmp2("", "test", 5), 0);  // Empty s1 ends immediately
        assert_eq!(strcmp2("test", "", 5), 0);  // Empty s2 ends immediately
    }

    #[test]
    fn strcmp2_zero_length() {
        // C code does compare first character even when length=0!
        // This is because the do-while loop executes at least once
        assert!(strcmp2("hello", "world", 0) < 0);  // 'h' < 'w'
        assert_eq!(strcmp2("abc", "abc", 0), 0);     // First chars match, returns 0
    }

    // ========== strcpy1 tests ==========

    #[test]
    fn strcpy1_normal_copy_with_padding() {
        assert_eq!(strcpy1("hello", 10), "hello     ");
        assert_eq!(strcpy1("test", 8), "test    ");
    }

    #[test]
    fn strcpy1_exact_length() {
        assert_eq!(strcpy1("hello", 5), "hello");
        assert_eq!(strcpy1("test", 4), "test");
    }

    #[test]
    fn strcpy1_truncate_source() {
        assert_eq!(strcpy1("hello world", 5), "hello");
    }

    #[test]
    fn strcpy1_empty_source() {
        assert_eq!(strcpy1("", 5), "     ");
        assert_eq!(strcpy1("", 10), "          ");
    }

    #[test]
    fn strcpy1_zero_length() {
        assert_eq!(strcpy1("hello", 0), "");
    }

    #[test]
    fn strcpy1_pads_after_short_string() {
        assert_eq!(strcpy1("a", 5), "a    ");
        assert_eq!(strcpy1("ab", 5), "ab   ");
    }

    // ========== strcpy2 tests ==========

    #[test]
    fn strcpy2_normal_copy() {
        assert_eq!(strcpy2("hello", 10), "hello");
        assert_eq!(strcpy2("test", 10), "test");
    }

    #[test]
    fn strcpy2_exact_length() {
        assert_eq!(strcpy2("hello", 5), "hello");
    }

    #[test]
    fn strcpy2_truncate_at_length() {
        assert_eq!(strcpy2("hello world", 5), "hello");
        assert_eq!(strcpy2("testing", 4), "test");
    }

    #[test]
    fn strcpy2_empty_source() {
        assert_eq!(strcpy2("", 10), "");
    }

    #[test]
    fn strcpy2_zero_length() {
        assert_eq!(strcpy2("hello", 0), "");
    }

    #[test]
    fn strcpy2_stops_at_string_end() {
        // Unlike strcpy1, strcpy2 doesn't pad - just returns what's copied
        assert_eq!(strcpy2("hi", 10), "hi");
    }

    #[test]
    fn strcpy2_vs_strcpy1_difference() {
        // strcpy1 pads with spaces
        assert_eq!(strcpy1("hi", 5), "hi   ");
        // strcpy2 does not pad
        assert_eq!(strcpy2("hi", 5), "hi");
    }

    // ========== stos tests ==========

    #[test]
    fn stos_basic_extraction() {
        assert_eq!(stos("Hello World", 1, 5), "Hello");
        assert_eq!(stos("Testing", 2, 4), "est");
        assert_eq!(stos("Test", 1, 4), "Test");
    }

    #[test]
    fn stos_out_of_bounds() {
        assert_eq!(stos("Short", 1, 10), "Short");  // Truncates
        assert_eq!(stos("Test", 5, 10), "");         // Start beyond string
        assert_eq!(stos("Test", 0, 5), "");          // Invalid start (0)
        assert_eq!(stos("Test", 5, 3), "");          // a > b
    }

    #[test]
    fn stos_single_char() {
        assert_eq!(stos("Hello", 1, 1), "H");
        assert_eq!(stos("World", 3, 3), "r");
    }

    // ========== stos_inv tests ==========

    #[test]
    fn stos_inv_basic_write() {
        assert_eq!(stos_inv("ABC", 3, 5, 10), "  ABC     ");
        assert_eq!(stos_inv("Test", 1, 4, 6), "Test  ");
    }

    #[test]
    fn stos_inv_beginning() {
        assert_eq!(stos_inv("Start", 1, 5, 10), "Start     ");
    }

    #[test]
    fn stos_inv_middle() {
        assert_eq!(stos_inv("Mid", 4, 6, 10), "   Mid    ");
    }

    #[test]
    fn stos_inv_invalid_params() {
        assert_eq!(stos_inv("Test", 0, 5, 10), "          ");  // a=0
        assert_eq!(stos_inv("Test", 5, 3, 10), "          ");  // a > b
    }

    // ========== strsplt tests ==========

    #[test]
    fn strsplt_basic_split() {
        assert_eq!(strsplt("one,two,three", ','), vec!["one", "two", "three"]);
        assert_eq!(strsplt("a b c", ' '), vec!["a", "b", "c"]);
    }

    #[test]
    fn strsplt_with_spaces() {
        assert_eq!(strsplt("a, b, c", ','), vec!["a", "b", "c"]);
        assert_eq!(strsplt("  hello  ,  world  ", ','), vec!["hello", "world"]);
    }

    #[test]
    fn strsplt_quoted_sections() {
        assert_eq!(
            strsplt("hello, \"world, test\", end", ','),
            vec!["hello", "world, test", "end"]
        );
        assert_eq!(
            strsplt("a, \"b,c\", d", ','),
            vec!["a", "b,c", "d"]
        );
    }

    #[test]
    fn strsplt_empty_and_single() {
        assert_eq!(strsplt("", ','), Vec::<String>::new());
        assert_eq!(strsplt("single", ','), vec!["single"]);
    }

    #[test]
    fn strsplt_multiple_delimiters() {
        assert_eq!(strsplt("a,,b", ','), vec!["a", "b"]);  // Skips empty
        assert_eq!(strsplt(",a,b,", ','), vec!["a", "b"]);  // Leading/trailing
    }
}
