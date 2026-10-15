pub trait StringExt {
    fn find_singular_char(&self, c: char) -> Option<usize>;
}

impl StringExt for str {
    /// Returns the byte index of the first occurrence of a character `c` in the string,
    /// where `c` is not preceded or followed by the same character.
    fn find_singular_char(&self, c: char) -> Option<usize> {
        let mut prev = None;
        let mut chars = self.char_indices().peekable();
        while let Some((i, ch)) = chars.next() {
            if ch == c && prev != Some(c) && chars.peek().is_none_or(|(_, ch)| *ch != c) {
                return Some(i);
            }
            prev = Some(ch);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_singular_character() {
        assert_eq!("aabbcc".find_singular_char('a'), None);
        assert_eq!("aabbcc".find_singular_char('b'), None);
        assert_eq!("aabbcc".find_singular_char('c'), None);
        assert_eq!("aabbcc".find_singular_char('d'), None);
    }
}
