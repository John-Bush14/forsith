use std::io::{BufRead, Cursor, Read, Seek};
use derive_more::{Deref, DerefMut};
use crate::int::Int;

#[derive(Debug, Deref, DerefMut)]
pub struct CursorVec<T>(Cursor<Vec<T>>);

impl<T: Default + Clone> CursorVec<T> {
    pub fn new(len: usize) -> Self {Self(Cursor::new(vec![T::default(); len]))}

    pub fn expand(&mut self, len: usize) {
        let cap = self.capacity();
        self.get_mut().resize(cap + len, T::default());
    }
}

impl<T> Default for CursorVec<T> {fn default() -> Self {Self(Cursor::new(Vec::new()))}}

impl<T> CursorVec<T> {
    pub fn into_inner(self) -> Cursor<Vec<T>> {self.0}
    pub fn read_single(&mut self) -> &T {&self.take_slice(1)[0]}
    pub fn remaining(&self) -> usize {self.capacity() - self.cursor()}
    pub fn capacity(&self) -> usize {self.get_ref().len()}
    pub fn cursor(&self) -> usize {usize::try_from(self.position()).unwrap()}
    pub fn set_cursor(&mut self, cursor: usize) {self.set_position(cursor as u64);}
    pub fn consume(&mut self, len: usize) {self.set_cursor(self.cursor() + len);}
    pub fn unconsume(&mut self, len: usize) {self.set_cursor(self.cursor().saturating_sub(len));}
    #[must_use]
    pub fn is_empty(&self) -> bool {self.capacity() == 0}
    pub fn is_full(&self) -> bool {self.cursor() == self.capacity()}
    pub fn current(&self) -> Option<&T> {self.get_ref().get(self.cursor())}

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

impl<T> From<Vec<T>> for CursorVec<T> where T: Default + Clone {
    fn from(vec: Vec<T>) -> Self {
        Self(Cursor::new(vec))
    }
}

impl Read for CursorVec<u8> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {(**self).read(buf)}
}

impl Seek for CursorVec<u8> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {(**self).seek(pos)}
}

#[derive(Debug, Deref, DerefMut)]
pub struct CursorString<'input>(std::io::Cursor<&'input str>);
impl<'input> From<&'input str> for CursorString<'input> {
    fn from(s: &'input str) -> Self {
        Self(std::io::Cursor::new(s))
    }
}
impl CursorString<'_> {
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

    pub fn cursor(&self) -> usize {
        usize::try_from(self.position()).expect("str len is always less then usize::MAX")
    }

    pub fn remaining_str(&self) -> &str {
        &self.get_ref()[self.cursor()..]
    }

    pub fn line_col(&self, cursor: usize) -> (usize, usize) {
        let lines = self.get_ref()[..cursor].lines();

        let (line, last_line) = lines.enumerate().last().unwrap_or((0, ""));

        (line + 1, last_line.len())
    }
}

