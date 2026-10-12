use crate::alloc::{boxed::Box, vec::Vec};
use core::slice;
use forsith_base::buffer;

const BASE_CHUNK_SIZE: usize = 4 * 1024; // linux page size
const MAX_CHUNK_SIZE: usize = 1024 * 1024; // linux huge page size

#[derive(Debug, Default)]
pub struct Arena<'arena, T: Default + Copy> {
    chunks: Vec<Box<[T]>>,
    index: usize,
    phantom: core::marker::PhantomData<&'arena ()>,
}

impl<'arena, T: Default + Copy> Arena<'arena, T> {
    const BASE_CHUNK_SIZE: usize = BASE_CHUNK_SIZE / core::mem::size_of::<T>();
    const MAX_CHUNK_SIZE: usize = MAX_CHUNK_SIZE / core::mem::size_of::<T>();

    fn next_chunk_size(&self) -> usize {
        (Self::BASE_CHUNK_SIZE * (1 << self.chunks.len())).min(Self::MAX_CHUNK_SIZE)
    }

    #[rustfmt::skip]
    fn grow_if_needed(&mut self, additional: usize) {
        if self.chunks.last().is_none_or(|chunk| self.index + additional > chunk.len()) {
            let new_chunk_size = self.next_chunk_size().max(additional);
            self.chunks.push(buffer![T::default(); new_chunk_size].into());
            self.index = 0;
        }
    }

    // grow_if_needed makes sure that chunks is never empty and that there
    // is enough space in the last chunk for buf, so unwrap and from_raw_parts_mut are safe here.
    #[allow(clippy::missing_panics_doc)]
    pub fn alloc(&mut self, buf: &[T]) -> &'arena mut [T] {
        self.grow_if_needed(buf.len());

        let chunk = self.chunks.last_mut().unwrap();
        let slice = unsafe {
            slice::from_raw_parts_mut(chunk.as_mut_ptr().add(self.index), buf.len()) // for lifetime
        };
        self.index += slice.len();
        slice.copy_from_slice(buf);

        slice
    }
}

impl<'arena> Arena<'arena, u8> {
    pub fn alloc_str(&mut self, s: &str) -> &'arena mut str {
        let bytes = self.alloc(s.as_bytes());
        unsafe { core::str::from_utf8_unchecked_mut(bytes) }
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
        assert_eq!(arena.chunks.last().unwrap().len(), BASE_CHUNK_SIZE);
    }

    #[test]
    fn alloc_full_chunk_empty_hello() {
        let mut arena = Arena::<u8>::default();
        let full_chunk = "a".repeat(BASE_CHUNK_SIZE);
        let full = arena.asserted_alloc_str(&full_chunk);
        let empty = arena.asserted_alloc_str("");
        let _ = arena.asserted_alloc_str("hello");
        assert_eq!(full, full_chunk);
        assert_eq!(empty, "");
        assert_eq!(arena.chunks.last().unwrap().len(), BASE_CHUNK_SIZE * 2);
    }

    #[test]
    fn alloc_large_string() {
        let mut arena = Arena::<u8>::default();
        let large_string = "a".repeat(BASE_CHUNK_SIZE + 1);
        let _ = arena.asserted_alloc_str(&large_string);
        assert_eq!(arena.chunks.last().unwrap().len(), BASE_CHUNK_SIZE + 1);
    }

    #[test]
    fn alloc_hello_large_string_hello() {
        let mut arena = Arena::<u8>::default();
        let hello1 = arena.asserted_alloc_str("hello");
        let large_string = "a".repeat(BASE_CHUNK_SIZE + 1);
        let large = arena.asserted_alloc_str(&large_string);
        let _ = arena.asserted_alloc_str("hello");
        assert_eq!(hello1, "hello");
        assert_eq!(large, large_string);
    }

    #[test]
    fn move_arena_hello_hello() {
        let mut arena = Arena::<u8>::default();
        let hello = arena.asserted_alloc_str("hello");
        let mut moved_arena = core::mem::take(&mut arena);
        let _ = moved_arena.asserted_alloc_str("world");
        assert_eq!(hello, "hello");
    }

    #[test]
    fn move_arena_hello_large_string_hello() {
        let mut arena = Arena::<u8>::default();
        let hello1 = arena.asserted_alloc_str("hello");
        let mut moved_arena = core::mem::take(&mut arena);
        let large_string = "a".repeat(BASE_CHUNK_SIZE + 1);
        let large = moved_arena.asserted_alloc_str(&large_string);
        let _ = moved_arena.asserted_alloc_str("hello");
        assert_eq!(hello1, "hello");
        assert_eq!(large, large_string);
    }
}
