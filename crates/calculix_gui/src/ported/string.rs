//! Rust ports of `compare.c`, `compareStrings.c`, and `strfind.c`.

pub fn compare_prefix(str1: &str, str2: &str, length: usize) -> usize {
    let lhs = str1.as_bytes();
    let rhs = str2.as_bytes();
    let mut i = 0usize;
    while i < length && i < lhs.len() && i < rhs.len() && lhs[i] == rhs[i] {
        i += 1;
    }
    i
}

pub fn compare_strings(str1: &str, str2: &str) -> i32 {
    let length = str1.len();
    if str2.len() != length {
        return -1;
    }
    if compare_prefix(str1, str2, length) == length {
        length as i32
    } else {
        0
    }
}

pub fn strfind(as1: &str, as2: &str) -> i32 {
    if as2.is_empty() {
        return -1;
    }
    let length_as2 = as2.len();
    let needle_first = as2.as_bytes()[0];
    let hay = as1.as_bytes();

    for i in 0..hay.len() {
        if hay[i] == needle_first && compare_prefix(&as1[i..], as2, length_as2) == length_as2 {
            return i as i32;
        }
    }
    -1
}

#[cfg(test)]
mod tests {
    use super::{compare_prefix, compare_strings, strfind};

    #[test]
    fn compare_prefix_matches_legacy_behavior() {
        assert_eq!(compare_prefix("abcde", "abZ", 5), 2);
        assert_eq!(compare_prefix("abc", "abc", 3), 3);
        assert_eq!(compare_prefix("abc", "abc", 2), 2);
    }

    #[test]
    fn compare_strings_returns_length_or_status() {
        assert_eq!(compare_strings("abc", "abc"), 3);
        assert_eq!(compare_strings("abc", "abx"), 0);
        assert_eq!(compare_strings("abc", "ab"), -1);
    }

    #[test]
    fn strfind_returns_first_match_or_minus_one() {
        assert_eq!(strfind("abc abc", "abc"), 0);
        assert_eq!(strfind("abc abc", "bc"), 1);
        assert_eq!(strfind("abc abc", "zz"), -1);
        assert_eq!(strfind("abc", ""), -1);
    }
}
