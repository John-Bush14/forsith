use forsith_proc::{Deref, DerefMut};
use std::io::{BufRead, Cursor, Read, Seek};

#[derive(Debug, Deref, DerefMut, Default)]
pub struct CursorVec<T>(Cursor<Vec<T>>);

impl<T: Default + Clone> CursorVec<T> {
    #[must_use]
    pub fn new(len: usize) -> Self {
        Self(Cursor::new(vec![T::default(); len]))
    }

    pub fn expand(&mut self, len: usize) {
        let cap = self.capacity();
        self.get_mut().resize(cap + len, T::default());
    }
}

impl<T> CursorVec<T> {
    #[must_use]
    pub fn into_inner(self) -> Cursor<Vec<T>> {
        self.0
    }
    pub fn read_single(&mut self) -> &T {
        &self.take_slice(1)[0]
    }
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.capacity() - self.cursor()
    }
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.get_ref().len()
    }
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // Vec len is always less than usize::MAX
    pub fn cursor(&self) -> usize {
        self.position() as usize
    }
    pub fn set_cursor(&mut self, cursor: usize) {
        self.set_position(cursor as u64);
    }
    pub fn consume(&mut self, len: usize) {
        self.set_cursor(self.cursor() + len);
    }
    pub fn unconsume(&mut self, len: usize) {
        self.set_cursor(self.cursor().saturating_sub(len));
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.capacity() == 0
    }
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.cursor() == self.capacity()
    }
    #[must_use]
    pub fn current(&self) -> Option<&T> {
        self.get_ref().get(self.cursor())
    }

    #[inline(always)]
    pub fn write_fast_single(&mut self, data: T) {
        let cursor = self.cursor();
        self.get_mut()[cursor] = data;
        self.set_cursor(cursor + 1);
    }

    pub fn take_slice(&mut self, len: usize) -> &[T] {
        let cursor = self.cursor();
        self.set_cursor(cursor + len);
        &self.get_ref()[cursor..cursor + len]
    }
    pub fn take_mut_slice(&mut self, len: usize) -> &mut [T] {
        let cursor = self.cursor();
        self.set_cursor(cursor + len);
        &mut self.get_mut()[cursor..cursor + len]
    }
}
impl CursorVec<u8> {
    pub fn fill_from(&mut self, reader: &mut impl Read, len: usize) -> std::io::Result<()> {
        let cursor = self.cursor();
        let buf = &mut self.0.get_mut()[cursor..cursor + len];
        reader.read_exact(buf)?;

        Ok(())
    }
    pub fn read_from(&mut self, reader: &mut impl Read, len: usize) -> std::io::Result<()> {
        self.fill_from(reader, len)?;
        self.consume(len);
        Ok(())
    }
}

impl<T> From<Vec<T>> for CursorVec<T>
where
    T: Default + Clone,
{
    fn from(vec: Vec<T>) -> Self {
        Self(Cursor::new(vec))
    }
}

impl Read for CursorVec<u8> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        (**self).read(buf)
    }
}

impl Seek for CursorVec<u8> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        (**self).seek(pos)
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct CursorString<'input>(std::io::Cursor<&'input str>);
impl<'input> From<&'input str> for CursorString<'input> {
    fn from(s: &'input str) -> Self {
        Self(std::io::Cursor::new(s))
    }
}

impl CursorString<'_> {
    #[must_use]
    pub fn peek(&self, len: usize) -> &str {
        let pos = self.cursor();
        let end = (pos + len).min(self.get_ref().len());
        &self.get_ref()[pos..end]
    }

    pub fn take(&mut self, len: usize) -> &str {
        let pos = self.cursor();
        let end = (pos + len).min(self.get_ref().len());
        self.consume(len);
        &self.get_ref()[pos..end]
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // Vec len is always less than usize::MAX
    pub fn cursor(&self) -> usize {
        self.position() as usize
    }

    #[must_use]
    pub fn remaining_str(&self) -> &str {
        &self.get_ref()[self.cursor()..]
    }

    #[must_use]
    pub fn line_col(&self, cursor: usize) -> (usize, usize) {
        let lines = self.get_ref()[..cursor].lines();

        let (line, last_line) = lines.enumerate().last().unwrap_or((0, ""));

        if cursor > 0
            && let Some(c) = self
                .get_ref()
                .as_bytes()
                .get(cursor - 1)
                .map(|s| char::from(*s))
            && c == '\n'
        {
            return (line + 2, 1);
        }

        (line + 1, last_line.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_vec_tracks_cursor_and_slices() {
        let mut vec = CursorVec::from(vec![1, 2, 3, 4, 5]);

        assert_eq!(vec.capacity(), 5);
        assert_eq!(vec.cursor(), 0);

        assert_eq!(vec.take_slice(2), &[1, 2]);
        assert_eq!(vec.cursor(), 2);

        vec.unconsume(1);
        assert_eq!(vec.cursor(), 1);

        vec.consume(3);
        assert_eq!(vec.cursor(), 4);
        assert_eq!(vec.current(), Some(&5));

        assert_eq!(vec.read_single(), &5);
        assert_eq!(vec.cursor(), 5);
        assert!(vec.is_full());
    }

    #[test]
    fn cursor_vec_expand_and_write_fast_single() {
        let mut vec = CursorVec::new(2);
        vec.expand(3);

        assert_eq!(vec.capacity(), 5);
        vec.write_fast_single(7);
        assert_eq!(vec.cursor(), 1);
        vec.unconsume(1);
        assert_eq!(vec.take_slice(1), &[7]);
    }

    #[test]
    fn cursor_vec_fill_and_read_from_reader() {
        let mut vec = CursorVec::new(8);
        let mut reader = std::io::Cursor::new(b"abcd");

        vec.fill_from(&mut reader, 4).unwrap();
        assert_eq!(&vec.get_ref()[..4], b"abcd");

        let mut vec2 = CursorVec::new(3);
        vec2.read_from(&mut std::io::Cursor::new(b"xyz"), 3)
            .unwrap();
        assert_eq!(vec2.get_ref(), b"xyz");
    }

    #[test]
    fn cursor_string_peek_take_and_line_col() {
        let mut s = CursorString::from("hello\nworld");

        assert_eq!(s.peek(5), "hello");
        assert_eq!(s.take(5), "hello");
        assert_eq!(s.cursor(), 5);
        assert_eq!(s.remaining_str(), "\nworld");

        assert_eq!(s.line_col(1), (1, 1));
        assert_eq!(s.line_col(6), (2, 1));
    }
}
