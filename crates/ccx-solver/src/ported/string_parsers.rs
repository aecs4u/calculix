//! Rust ports of `stoi.c` and `stof.c`.
//!
//! String parsing utilities for extracting integers and floats from substrings.
//! These are commonly used for parsing fixed-width format input files.

/// Extracts an integer from a substring of positions [a, b).
///
/// This is a direct port of the C function `stoi` from the legacy CalculiX codebase.
/// It extracts characters from position `a-1` to position `b-1` (using 1-based indexing
/// like Fortran) and parses them as an integer.
///
/// # Arguments
///
/// * `string` - Source string
/// * `a` - Start position (1-based, inclusive)
/// * `b` - End position (1-based, inclusive)
///
/// # Returns
///
/// The parsed integer value, or 0 if parsing fails.
///
/// # Examples
///
/// ```
/// use ccx_solver::ported::stoi;
///
/// let s = "  123  456  789";
/// assert_eq!(stoi(s, 1, 5), 123);
/// assert_eq!(stoi(s, 7, 10), 456);
/// assert_eq!(stoi(s, 12, 15), 789);
/// ```
pub fn stoi(string: &str, a: usize, b: usize) -> i32 {
    if a == 0 || b == 0 || a > b {
        return 0;
    }

    let bytes = string.as_bytes();
    let start = a.saturating_sub(1);
    let end = b.min(bytes.len());

    if start >= end {
        return 0;
    }

    let substring = match std::str::from_utf8(&bytes[start..end]) {
        Ok(s) => s.trim(),
        Err(_) => return 0,
    };

    substring.parse::<i32>().unwrap_or(0)
}

/// Extracts a double from a substring of positions [a, b).
///
/// This is a direct port of the C function `stof` from the legacy CalculiX codebase.
/// It extracts characters from position `a-1` to position `b-1` (using 1-based indexing
/// like Fortran) and parses them as a double precision float.
///
/// # Arguments
///
/// * `string` - Source string
/// * `a` - Start position (1-based, inclusive)
/// * `b` - End position (1-based, inclusive)
///
/// # Returns
///
/// The parsed f64 value, or 0.0 if parsing fails.
///
/// # Examples
///
/// ```
/// use ccx_solver::ported::stof;
///
/// let s = "  1.5  -2.3e2  2.75";
/// assert_eq!(stof(s, 1, 5), 1.5);
/// assert_eq!(stof(s, 7, 13), -230.0);
/// assert!((stof(s, 16, 19) - 2.75).abs() < 1e-10);
/// ```
pub fn stof(string: &str, a: usize, b: usize) -> f64 {
    if a == 0 || b == 0 || a > b {
        return 0.0;
    }

    let bytes = string.as_bytes();
    let start = a.saturating_sub(1);
    let end = b.min(bytes.len());

    if start >= end {
        return 0.0;
    }

    let substring = match std::str::from_utf8(&bytes[start..end]) {
        Ok(s) => s.trim(),
        Err(_) => return 0.0,
    };

    substring.parse::<f64>().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::{stof, stoi};

    #[test]
    fn stoi_extracts_integers() {
        let s = "  123  456  789";
        assert_eq!(stoi(s, 1, 5), 123);
        assert_eq!(stoi(s, 7, 10), 456);
        assert_eq!(stoi(s, 12, 15), 789);
    }

    #[test]
    fn stoi_handles_negative_numbers() {
        let s = " -123  -456";
        assert_eq!(stoi(s, 1, 5), -123);
        assert_eq!(stoi(s, 7, 11), -456);
    }

    #[test]
    fn stoi_handles_edge_cases() {
        let s = "12345";
        assert_eq!(stoi(s, 1, 5), 12345);
        assert_eq!(stoi(s, 3, 5), 345);
        assert_eq!(stoi(s, 1, 1), 1);
        assert_eq!(stoi(s, 0, 0), 0); // Invalid range
        assert_eq!(stoi(s, 5, 3), 0); // Reversed range
    }

    #[test]
    fn stoi_handles_whitespace() {
        let s = "   42   ";
        assert_eq!(stoi(s, 1, 8), 42);
    }

    #[test]
    fn stoi_handles_invalid_input() {
        let s = "  abc  ";
        assert_eq!(stoi(s, 1, 7), 0);
    }

    #[test]
    fn stoi_handles_overflow() {
        let s = "  123  456  789";
        assert_eq!(stoi(s, 13, 100), 789); // Beyond string length, extracts "789"
    }

    #[test]
    fn stof_extracts_floats() {
        let s = "  1.5  -2.3  3.75";
        assert_eq!(stof(s, 1, 5), 1.5);
        assert_eq!(stof(s, 7, 11), -2.3);
        assert!((stof(s, 14, 17) - 3.75).abs() < 1e-10);
    }

    #[test]
    fn stof_handles_scientific_notation() {
        let s = "  1.5e2  -2.3e-2  2.75E+1";
        assert_eq!(stof(s, 1, 7), 150.0);
        assert_eq!(stof(s, 9, 16), -0.023);
        assert!((stof(s, 18, 25) - 27.5).abs() < 1e-10);
    }

    #[test]
    fn stof_handles_integers() {
        let s = "  123  456";
        assert_eq!(stof(s, 1, 5), 123.0);
        assert_eq!(stof(s, 7, 10), 456.0);
    }

    #[test]
    fn stof_handles_edge_cases() {
        let s = "1.23456";
        assert!((stof(s, 1, 7) - 1.23456).abs() < 1e-10);
        assert_eq!(stof(s, 0, 0), 0.0); // Invalid range
        assert_eq!(stof(s, 5, 3), 0.0); // Reversed range
    }

    #[test]
    fn stof_handles_whitespace() {
        let s = "   42.0   ";
        assert_eq!(stof(s, 1, 10), 42.0);
    }

    #[test]
    fn stof_handles_invalid_input() {
        let s = "  xyz  ";
        assert_eq!(stof(s, 1, 7), 0.0);
    }

    #[test]
    fn stof_handles_overflow() {
        let s = "  1.5  2.3";
        assert_eq!(stof(s, 7, 100), 2.3); // Beyond string length
    }

    #[test]
    fn fortran_style_fixed_format() {
        // Simulating a Fortran fixed-format line
        // Simple case with clear field boundaries
        let line = "    42  1.5 2.3 3.7";

        // Extract integer from columns 1-8
        assert_eq!(stoi(line, 1, 8), 42);

        // Extract first float from columns 9-12
        assert_eq!(stof(line, 9, 12), 1.5);

        // Extract second float from columns 13-16
        assert_eq!(stof(line, 13, 16), 2.3);

        // Extract third float from columns 17-20
        assert!((stof(line, 17, 20) - 3.7).abs() < 1e-10);
    }
}
