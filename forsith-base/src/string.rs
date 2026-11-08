pub trait StrExt {
    fn find_singular_char(&self, c: char) -> Option<usize>;
}

impl StrExt for str {
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

pub trait StringExt {
    fn replace_end(&mut self, n: usize, replacement: &str);
}

impl StringExt for String {
    /// Replaces the last `n` characters of the string with `replacement`.
    fn replace_end(&mut self, n: usize, replacement: &str) {
        let len = self.len();
        self.truncate(len.checked_sub(n).expect("String length is less than n"));
        self.push_str(replacement);
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
