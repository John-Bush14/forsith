use std::collections::HashMap;

use crate::arena::Arena;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternedString(usize);

#[derive(Debug, Default)]
pub struct StringInterner<'arena> {
    key_map: HashMap<String, InternedString>,
    str_map: Vec<&'arena str>,
    arena: Arena<'arena, u8>,
}

impl StringInterner<'_> {
    pub fn interned(&mut self, s: &str) -> InternedString {
        #[allow(clippy::option_if_let_else)]
        match self.key_map.get(s) {
            Some(interned) => *interned,
            None => self.intern(s),
        }
    }

    fn intern(&mut self, s: &str) -> InternedString {
        let interned_s = InternedString(self.str_map.len());
        self.key_map.insert(s.to_string(), interned_s);
        self.str_map.push(self.arena.alloc_str(s));

        interned_s
    }

    #[must_use]
    pub fn resolve(&self, interned: InternedString) -> &str {
        self.str_map[interned.0]
    }
}

#[cfg(test)]
mod string_interner_tests {
    use super::*;

    impl StringInterner<'_> {
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

    #[test]
    fn duplicate_interns_are_stable() {
        let mut interner = StringInterner::default();

        let first = interner.interned("hello");
        let second = interner.interned("hello");
        let third = interner.interned("world");

        assert_eq!(first, second);
        assert_ne!(first, third);
        assert_eq!(interner.resolve(first), "hello");
        assert_eq!(interner.resolve(third), "world");
    }

    #[test]
    fn large_strings_stay_resolved() {
        let mut interner = StringInterner::default();
        let large = "a".repeat(10_000);

        let interned = interner.asserted_interned(&large);
        assert_eq!(interner.resolve(interned), large);
    }

    #[test]
    fn punctuation_and_unicode_are_preserved() {
        let mut interner = StringInterner::default();

        let interned = interner.asserted_interned("Hello, world! 💡");
        assert_eq!(interner.resolve(interned), "Hello, world! 💡");

        let duplicate = interner.interned("Hello, world! 💡");
        assert_eq!(duplicate, interned);
    }
}
