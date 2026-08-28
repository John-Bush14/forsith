use std::io::BufRead;

use derive_more::{Deref, DerefMut};

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

