use crate::IndexedString;
use std::num::NonZeroUsize;

pub(crate) struct StringScanner<'a> {
    cursor: usize,
    indexed_string: IndexedString<'a>,
}

impl<'a> StringScanner<'a> {
    pub(crate) fn from_str(src: &'a str) -> StringScanner<'a> {
        StringScanner {
            cursor: 0,
            indexed_string: IndexedString::from_str(src),
        }
    }

    pub(crate) fn get_cursor(&self) -> usize {
        self.cursor
    }

    fn get_char_index_range_from_cursor<'b>(&'b self) -> &'b [(usize, char)] {
        self.indexed_string.get_char_index_range_from(self.cursor)
    }

    pub(crate) fn is_at_end_of_string(&'a self) -> bool {
        self.cursor >= self.indexed_string.len()
    }

    pub(crate) fn substring(&self, from: usize, to: usize) -> &'a str {
        self.indexed_string.substring(from, to)
    }

    pub(crate) fn scan_char_once(&mut self, char_to_scan: char) -> ScanCharResult {
        if self.cursor < self.indexed_string.len() {
            if self.indexed_string.char_at_idx(self.cursor) == char_to_scan {
                self.cursor += 1;
                ScanCharResult::CharFound(unsafe { NonZeroUsize::new_unchecked(1) })
            } else {
                ScanCharResult::CharNotFound
            }
        } else {
            ScanCharResult::CharNotFound
        }
    }

    pub(crate) fn scan_until_char(&mut self, char_to_find: char) -> ScanUntilCharResult {
        let mut chars_scanned: usize = 0;
        let mut char_found = false;
        for (_, c) in self.get_char_index_range_from_cursor() {
            if *c == char_to_find {
                char_found = true;
                break;
            } else {
                chars_scanned += 1;
            }
        }

        self.cursor += chars_scanned;
        if char_found {
            ScanUntilCharResult::CharFound
        } else {
            ScanUntilCharResult::EndOfStringReached
        }
    }

    pub(crate) fn scan_until_char_or_whitespace(
        &mut self,
        char_to_find: char,
    ) -> ScanUntilCharResult {
        let mut chars_scanned: usize = 0;
        let mut char_found = false;
        for (_, c) in self.get_char_index_range_from_cursor() {
            let pc = *c;
            if pc == char_to_find || pc == '\x09' || pc == '\x20' {
                char_found = true;
                break;
            } else {
                chars_scanned += 1;
            }
        }

        self.cursor += chars_scanned;
        if char_found {
            ScanUntilCharResult::CharFound
        } else {
            ScanUntilCharResult::EndOfStringReached
        }
    }

    pub(crate) fn scan_whitespace_repeating(&mut self) -> ScanCharResult {
        let mut chars_scanned: usize = 0;

        for (_, c) in self.get_char_index_range_from_cursor() {
            match *c {
                '\x09' | '\x20' => (),
                _ => break,
            }

            chars_scanned += 1;
        }

        if chars_scanned > 0 {
            self.cursor += chars_scanned;
            ScanCharResult::CharFound(unsafe { NonZeroUsize::new_unchecked(chars_scanned) })
        } else {
            ScanCharResult::CharNotFound
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) enum ScanCharResult {
    CharNotFound,
    CharFound(NonZeroUsize),
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) enum ScanUntilCharResult {
    CharFound,
    EndOfStringReached,
}

#[cfg(test)]
mod tests {
    use super::{ScanCharResult, ScanUntilCharResult, StringScanner};
    use std::num::NonZeroUsize;

    #[test]
    fn get_cursor() {
        assert_eq!(0, StringScanner::from_str("").get_cursor())
    }

    #[test]
    fn is_at_end_of_string_empty() {
        let scanner = StringScanner::from_str("");
        assert_eq!(true, scanner.is_at_end_of_string());
    }

    #[test]
    fn is_at_end_of_string_false() {
        let scanner = StringScanner::from_str("abc");
        assert_eq!(false, scanner.is_at_end_of_string());
    }

    #[test]
    fn is_at_end_of_string_true() {
        let mut scanner = StringScanner::from_str("abc");
        scanner.scan_char_once('a');
        scanner.scan_char_once('b');
        scanner.scan_char_once('c');

        assert_eq!(true, scanner.is_at_end_of_string());
    }

    #[test]
    fn scan_char_once_single_occurence() {
        let mut scanner = StringScanner::from_str("abcde");
        let result = scanner.scan_char_once('a');
        assert_eq!(
            ScanCharResult::CharFound(unsafe { NonZeroUsize::new_unchecked(1) }),
            result
        );
        assert_eq!(1, scanner.get_cursor());
    }

    #[test]
    fn scan_char_once_multiple_occurences() {
        let mut scanner = StringScanner::from_str("aaaabcde");
        let result = scanner.scan_char_once('a');
        assert_eq!(
            ScanCharResult::CharFound(unsafe { NonZeroUsize::new_unchecked(1) }),
            result
        );
        assert_eq!(1, scanner.get_cursor());
    }

    #[test]
    fn scan_char_once_no_match() {
        let mut scanner = StringScanner::from_str("bcde");
        let result = scanner.scan_char_once('a');
        assert_eq!(ScanCharResult::CharNotFound, result);
        assert_eq!(0, scanner.get_cursor());
    }

    #[test]
    fn scan_char_once_empty_str() {
        let mut scanner = StringScanner::from_str("");
        let result = scanner.scan_char_once('a');
        assert_eq!(ScanCharResult::CharNotFound, result);
        assert_eq!(0, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_immediate() {
        let mut scanner = StringScanner::from_str("abcde");
        let result = scanner.scan_until_char('a');
        assert_eq!(ScanUntilCharResult::CharFound, result);
        assert_eq!(0, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_empty() {
        let mut scanner = StringScanner::from_str("");
        let result = scanner.scan_until_char('a');
        assert_eq!(ScanUntilCharResult::EndOfStringReached, result);
        assert_eq!(0, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_mid_string() {
        let mut scanner = StringScanner::from_str("abcde");
        let result = scanner.scan_until_char('c');
        assert_eq!(ScanUntilCharResult::CharFound, result);
        assert_eq!(2, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_mid_string_multiple_matches() {
        let mut scanner = StringScanner::from_str("abccde");
        let result = scanner.scan_until_char('c');
        assert_eq!(ScanUntilCharResult::CharFound, result);
        assert_eq!(2, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_no_match() {
        let mut scanner = StringScanner::from_str("abcde");
        let result = scanner.scan_until_char('x');
        assert_eq!(ScanUntilCharResult::EndOfStringReached, result);
        assert_eq!(5, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_or_whitespace_immediate() {
        let mut scanner = StringScanner::from_str("abcde");
        let result = scanner.scan_until_char_or_whitespace('a');
        assert_eq!(ScanUntilCharResult::CharFound, result);
        assert_eq!(0, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_or_whitespace_empty() {
        let mut scanner = StringScanner::from_str("");
        let result = scanner.scan_until_char_or_whitespace('a');
        assert_eq!(ScanUntilCharResult::EndOfStringReached, result);
        assert_eq!(0, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_or_whitespace_mid_string() {
        let mut scanner = StringScanner::from_str("abcde");
        let result = scanner.scan_until_char_or_whitespace('c');
        assert_eq!(ScanUntilCharResult::CharFound, result);
        assert_eq!(2, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_or_whitespace_mid_string_x09() {
        let mut scanner = StringScanner::from_str("ab\x09de");
        let result = scanner.scan_until_char_or_whitespace('c');
        assert_eq!(ScanUntilCharResult::CharFound, result);
        assert_eq!(2, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_or_whitespace_mid_string_x20() {
        let mut scanner = StringScanner::from_str("ab de");
        let result = scanner.scan_until_char_or_whitespace('c');
        assert_eq!(ScanUntilCharResult::CharFound, result);
        assert_eq!(2, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_or_whitespace_mid_string_multiple_matches() {
        let mut scanner = StringScanner::from_str("abccde");
        let result = scanner.scan_until_char_or_whitespace('c');
        assert_eq!(ScanUntilCharResult::CharFound, result);
        assert_eq!(2, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_or_whitespace_mid_string_multiple_matches_x09() {
        let mut scanner = StringScanner::from_str("ab\x09\x09de");
        let result = scanner.scan_until_char_or_whitespace('c');
        assert_eq!(ScanUntilCharResult::CharFound, result);
        assert_eq!(2, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_or_whitespace_mid_string_multiple_matches_x20() {
        let mut scanner = StringScanner::from_str("ab \x09de");
        let result = scanner.scan_until_char_or_whitespace('c');
        assert_eq!(ScanUntilCharResult::CharFound, result);
        assert_eq!(2, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_or_whitespace_mid_string_multiple_matches_mixed() {
        let mut scanner = StringScanner::from_str("ab\x20cde");
        let result = scanner.scan_until_char_or_whitespace('c');
        assert_eq!(ScanUntilCharResult::CharFound, result);
        assert_eq!(2, scanner.get_cursor());
    }

    #[test]
    fn scan_until_char_or_whitespace_no_match() {
        let mut scanner = StringScanner::from_str("abcde");
        let result = scanner.scan_until_char_or_whitespace('x');
        assert_eq!(ScanUntilCharResult::EndOfStringReached, result);
        assert_eq!(5, scanner.get_cursor());
    }

    #[test]
    fn scan_whitespace_repeating_empty() {
        let mut scanner = StringScanner::from_str("");
        let result = scanner.scan_whitespace_repeating();
        assert_eq!(ScanCharResult::CharNotFound, result);
        assert_eq!(0, scanner.get_cursor());
    }

    #[test]
    fn scan_whitespace_repeating_no_whitespace() {
        let mut scanner = StringScanner::from_str("abcde");
        let result = scanner.scan_whitespace_repeating();
        assert_eq!(ScanCharResult::CharNotFound, result);
        assert_eq!(0, scanner.get_cursor());
    }

    #[test]
    fn scan_whitespace_repeating_single_x09() {
        let mut scanner = StringScanner::from_str("\x09");
        let result = scanner.scan_whitespace_repeating();
        assert_eq!(
            ScanCharResult::CharFound(unsafe { NonZeroUsize::new_unchecked(1) }),
            result
        );
        assert_eq!(1, scanner.get_cursor());
    }

    #[test]
    fn scan_whitespace_repeating_single_x20() {
        let mut scanner = StringScanner::from_str("\x20");
        let result = scanner.scan_whitespace_repeating();
        assert_eq!(
            ScanCharResult::CharFound(unsafe { NonZeroUsize::new_unchecked(1) }),
            result
        );
        assert_eq!(1, scanner.get_cursor());
    }

    #[test]
    fn scan_whitespace_repeating_mixed_entire_str() {
        let mut scanner = StringScanner::from_str(" \x09\x20\x09 ");
        let result = scanner.scan_whitespace_repeating();
        assert_eq!(
            ScanCharResult::CharFound(unsafe { NonZeroUsize::new_unchecked(5) }),
            result
        );
        assert_eq!(5, scanner.get_cursor());
    }

    #[test]
    fn scan_whitespace_repeating_mixed_prefix_only() {
        let mut scanner = StringScanner::from_str(" \x09\x20\x09 abcde");
        let result = scanner.scan_whitespace_repeating();
        assert_eq!(
            ScanCharResult::CharFound(unsafe { NonZeroUsize::new_unchecked(5) }),
            result
        );
        assert_eq!(5, scanner.get_cursor());
    }
}
