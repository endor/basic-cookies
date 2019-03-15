pub(crate) struct IndexedString<'a> {
    string: &'a str,
    char_indexes: Vec<(usize, char)>,
}

impl<'a> IndexedString<'a> {
    pub(crate) fn from_str(src: &'a str) -> IndexedString<'a> {
        IndexedString {
            string: src,
            char_indexes: src.char_indices().collect(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.char_indexes.len()
    }

    fn idx_of_char_in_str(&self, idx: usize) -> usize {
        if idx == self.char_indexes.len() {
            if idx == 0 {
                0
            } else {
                let (idx_in_str, last_char) = self.char_indexes[idx - 1];
                idx_in_str + last_char.len_utf8()
            }
        } else {
            let (idx_in_str, _) = self.char_indexes[idx];
            idx_in_str
        }
    }

    pub(crate) fn char_at_idx(&self, idx: usize) -> char {
        self.char_indexes[idx].1
    }

    pub(crate) fn substring(&self, from: usize, to: usize) -> &'a str {
        &self.string[self.idx_of_char_in_str(from)..self.idx_of_char_in_str(to)]
    }

    pub(crate) fn get_char_index_range_from<'b>(&'b self, from: usize) -> &'b [(usize, char)] {
        &self.char_indexes[from..]
    }
}

#[cfg(test)]
mod tests {
    use super::IndexedString;

    #[test]
    fn len_empty() {
        let indexed_str = IndexedString::from_str("");
        let expected = 0;
        let actual = indexed_str.len();

        assert_eq!(expected, actual);
    }

    #[test]
    fn len_not_empty() {
        let indexed_str = IndexedString::from_str("abcde");
        let expected = 5;
        let actual = indexed_str.len();

        assert_eq!(expected, actual);
    }

    #[test]
    fn idx_of_char_in_str_pos0() {
        let indexed_str = IndexedString::from_str("abcde");
        let expected = 0;
        let actual = indexed_str.idx_of_char_in_str(0);

        assert_eq!(expected, actual);
    }

    #[test]
    fn idx_of_char_in_str_pos1() {
        let indexed_str = IndexedString::from_str("abcde");
        let expected = 'a'.len_utf8();
        let actual = indexed_str.idx_of_char_in_str(1);

        assert_eq!(expected, actual);
    }

    #[test]
    fn idx_of_char_in_str_pos1_wide() {
        let indexed_str = IndexedString::from_str("東京都");
        let expected = '東'.len_utf8();
        let actual = indexed_str.idx_of_char_in_str(1);

        assert_eq!(expected, actual);
    }

    #[test]
    fn idx_of_char_in_str_one_after_last() {
        let indexed_str = IndexedString::from_str("abcde");
        let expected = 5;
        let actual = indexed_str.idx_of_char_in_str(5);

        assert_eq!(expected, actual);
    }

    #[test]
    fn char_at_idx_pos0() {
        let indexed_str = IndexedString::from_str("abcde");
        let expected = 'a';
        let actual = indexed_str.char_at_idx(0);

        assert_eq!(expected, actual);
    }

    #[test]
    fn char_at_idx_pos1() {
        let indexed_str = IndexedString::from_str("abcde");
        let expected = 'b';
        let actual = indexed_str.char_at_idx(1);

        assert_eq!(expected, actual);
    }

    #[test]
    fn char_at_idx_pos1_wide() {
        let indexed_str = IndexedString::from_str("東京都");
        let expected = '京';
        let actual = indexed_str.char_at_idx(1);

        assert_eq!(expected, actual);
    }

    #[test]
    fn substring_entire_string() {
        let indexed_str = IndexedString::from_str("abcde");
        let expected = "abcde";
        let actual = indexed_str.substring(0, 5);

        assert_eq!(expected, actual);
    }

    #[test]
    fn substring_first_2() {
        let indexed_str = IndexedString::from_str("abcde");
        let expected = "ab";
        let actual = indexed_str.substring(0, 2);

        assert_eq!(expected, actual);
    }

    #[test]
    fn substring_last_2() {
        let indexed_str = IndexedString::from_str("abcde");
        let expected = "de";
        let actual = indexed_str.substring(3, 5);

        assert_eq!(expected, actual);
    }

    #[test]
    fn substring_empty() {
        let indexed_str = IndexedString::from_str("");
        let expected = "";
        let actual = indexed_str.substring(0, 0);

        assert_eq!(expected, actual);
    }

    #[test]
    fn get_char_index_range_from_empty() {
        let indexed_str = IndexedString::from_str("");
        let result = indexed_str.get_char_index_range_from(0);

        assert_eq!(0, result.len());
    }

    #[test]
    fn get_char_index_range_from_one_after_last() {
        let indexed_str = IndexedString::from_str("abcde");
        let result = indexed_str.get_char_index_range_from(5);

        assert_eq!(0, result.len());
    }

    #[test]
    fn get_char_index_range_from_last_2() {
        let indexed_str = IndexedString::from_str("abcde");
        let result = indexed_str.get_char_index_range_from(3);

        assert_eq!(2, result.len());
        assert_eq!(3, result[0].0);
        assert_eq!('d', result[0].1);
        assert_eq!(4, result[1].0);
        assert_eq!('e', result[1].1);
    }

    #[test]
    fn get_char_index_range_from_beginning() {
        let indexed_str = IndexedString::from_str("abcde");
        let result = indexed_str.get_char_index_range_from(0);

        assert_eq!(5, result.len());
        assert_eq!(0, result[0].0);
        assert_eq!('a', result[0].1);
        assert_eq!(1, result[1].0);
        assert_eq!('b', result[1].1);
        assert_eq!(2, result[2].0);
        assert_eq!('c', result[2].1);
        assert_eq!(3, result[3].0);
        assert_eq!('d', result[3].1);
        assert_eq!(4, result[4].0);
        assert_eq!('e', result[4].1);
    }
}
