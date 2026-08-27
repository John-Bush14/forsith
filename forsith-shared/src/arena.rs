const CHUNK_SIZE: usize = 2usize.pow(12); // 4096

#[derive(Debug)]
pub struct Arena<'arena, T: Default + Copy> {
    chunks: Vec<Box<[T]>>,
    index: usize,
    phantom: std::marker::PhantomData<&'arena ()>,
}

impl<T: Default + Copy> Default for Arena<'_, T> {
    fn default() -> Self {
        Self {
            chunks: vec![],
            index: CHUNK_SIZE + 1,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'arena, T: Default + Copy> Arena<'arena, T> {
    pub fn alloc(&mut self, buf: &[T]) -> &'arena [T] {
        if self.index + buf.len() > CHUNK_SIZE {
            self.chunks.push(vec![T::default(); CHUNK_SIZE.max(buf.len())].into_boxed_slice());
            self.index = 0;
        }

        let chunk = self.chunks.last_mut().unwrap();
        let start = self.index;
        let end = start + buf.len();
        chunk[start..end].copy_from_slice(buf);
        self.index = end;

        unsafe {
            std::slice::from_raw_parts(chunk.as_ptr().add(start), buf.len())
        }
    }
}

impl<'arena> Arena<'arena, u8> {
    pub fn alloc_str(&mut self, s: &str) -> &'arena str {
        let bytes = self.alloc(s.as_bytes());
        unsafe {
            std::str::from_utf8_unchecked(bytes)
        }
    }
}

#[cfg(test)]
mod arena_tests {
    use super::*;

    impl<'arena> Arena<'arena, u8> {
        fn asserted_alloc_str(&mut self, s: &str) -> &'arena str {
            let allocated = self.alloc_str(s);
            assert_eq!(allocated, s);
            allocated
        }
    }

    #[test]
    fn alloc_hello_world() {
        let mut arena = Arena::<u8>::default();
        let hello = arena.asserted_alloc_str("hello");
        let _ = arena.asserted_alloc_str("world");
        assert_eq!(hello, "hello");
    }

    #[test]
    fn alloc_empty_hello_empty() {
        let mut arena = Arena::<u8>::default();
        let empty = arena.asserted_alloc_str("");
        let hello = arena.asserted_alloc_str("hello");
        let _ = arena.asserted_alloc_str("");
        assert_eq!(empty, "");
        assert_eq!(hello, "hello");
    }

    #[test]
    fn alloc_full_chunk_empty_hello() {
        let mut arena = Arena::<u8>::default();
        let full_chunk = "a".repeat(CHUNK_SIZE);
        let full = arena.asserted_alloc_str(&full_chunk);
        let empty = arena.asserted_alloc_str("");
        let _ = arena.asserted_alloc_str("hello");
        assert_eq!(full, full_chunk);
        assert_eq!(empty, "");
    }

    #[test]
    fn alloc_large_string() {
        let mut arena = Arena::<u8>::default();
        let large_string = "a".repeat(CHUNK_SIZE + 1);
        let _ = arena.asserted_alloc_str(&large_string);
    }

    #[test]
    fn alloc_hello_large_string_hello() {
        let mut arena = Arena::<u8>::default();
        let hello1 = arena.asserted_alloc_str("hello");
        let large_string = "a".repeat(CHUNK_SIZE + 1);
        let large = arena.asserted_alloc_str(&large_string);
        let _ = arena.asserted_alloc_str("hello");
        assert_eq!(hello1, "hello");
        assert_eq!(large, large_string);
    }

    #[test]
    fn move_arena_hello_hello() {
        let mut arena = Arena::<u8>::default();
        let hello = arena.asserted_alloc_str("hello");
        let mut moved_arena = std::mem::take(&mut arena);
        let _ = moved_arena.asserted_alloc_str("world");
        assert_eq!(hello, "hello");
    }

    #[test]
    fn move_arena_hello_large_string_hello() {
        let mut arena = Arena::<u8>::default();
        let hello1 = arena.asserted_alloc_str("hello");
        let mut moved_arena = std::mem::take(&mut arena);
        let large_string = "a".repeat(CHUNK_SIZE + 1);
        let large = moved_arena.asserted_alloc_str(&large_string);
        let _ = moved_arena.asserted_alloc_str("hello");
        assert_eq!(hello1, "hello");
        assert_eq!(large, large_string);
    }
}

