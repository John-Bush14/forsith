use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternedString(usize);

#[derive(Debug, Clone)]
struct StringEntry {
    index: usize,
    len: usize,
}

#[derive(Debug, Clone, Default)]
pub struct StringInterner {
    key_map: HashMap<String, InternedString>,
    str_map: Vec<StringEntry>, // (index, len)
    buffer: String
}

impl StringInterner {
    pub fn interned(&mut self, s: &str) -> InternedString {
        match self.key_map.get(s) {
            Some(interned) => *interned,
            None => self.intern(s),
        }
    }

    fn intern(&mut self, s: &str) -> InternedString {
        let interned_s = InternedString(self.str_map.len());
        self.key_map.insert(s.to_string(), interned_s);
        self.str_map.push(StringEntry {
            index: self.buffer.len(),
            len: s.len()
        });
        self.buffer.push_str(s);

        interned_s
    }

    pub fn resolve(&self, interned: InternedString) -> &str {
        let StringEntry {index, len} = self.str_map[interned.0];
        &self.buffer[index..index + len]
    }
}

#[cfg(test)]
mod string_interner_tests {
    use super::*;

    impl StringInterner {
        fn asserted_interned(&mut self, s: &str) -> InternedString {
            let i = self.interned(s);
            assert_eq!(self.resolve(i), s);
            i
        }
    }

    #[test]
    fn empty_hello_inequality() {
        let mut interner = StringInterner::default();

        let i = interner.asserted_interned("");
        let i2 = interner.asserted_interned("hello");
        assert_ne!(i, i2);
    }

    #[test]
    fn hello_equality() {
        let mut interner = StringInterner::default();

        let i = interner.asserted_interned("hello");
        let i2 = interner.asserted_interned("hello");
        assert_eq!(i, i2);
    }


    #[test]
    fn empty_equality() {
        let mut interner = StringInterner::default();

        let i = interner.asserted_interned("");
        let i2 = interner.asserted_interned("");
        assert_eq!(i, i2);
    }
}
