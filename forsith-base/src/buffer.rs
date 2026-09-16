use std::{alloc::{Layout, alloc}, ops::{Deref, DerefMut}, ptr};

#[macro_export]
macro_rules! buffer {
    () => (
        $crate::buffer::Buffer::new()
    );
    ($elem:expr; $n:expr) => (
        $crate::buffer::Buffer::from_elem($elem, $n)
    );
    ($($x:expr),+ $(,)?) => (
        $crate::buffer::Buffer::from([$($x),+])
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer<T: Clone>(Box<[T]>);

impl<T: Clone> Default for Buffer<T> {
    fn default() -> Self {
        Self(Box::new([]))
    }
}

impl<T: Clone> Deref for Buffer<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone> DerefMut for Buffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Clone> From<Vec<T>> for Buffer<T> {
    fn from(vec: Vec<T>) -> Self {
        Self(vec.into_boxed_slice())
    }
}

impl<T: Clone> From<Buffer<T>> for Vec<T> {
    fn from(buffer: Buffer<T>) -> Self {
        buffer.0.into_vec()
    }
}

impl<T: Clone> From<&[T]> for Buffer<T> {
    fn from(slice: &[T]) -> Self {
        unsafe {Self::copy_from_ptr(slice.as_ptr(), slice.len())}
    }
}

impl<T: Clone, const N: usize> From<&[T; N]> for Buffer<T> {
    fn from(array: &[T; N]) -> Self {
        unsafe {Self::copy_from_ptr(array.as_ptr(), N)}
    }
}

impl<T: Clone> From<Box<[T]>> for Buffer<T> {
    fn from(boxed_slice: Box<[T]>) -> Self {
        Self(boxed_slice)
    }
}

impl<T: Clone> From<Buffer<T>> for Box<[T]> {
    fn from(buffer: Buffer<T>) -> Self {
        buffer.0
    }
}

impl<T: Clone, const N: usize> From<[T; N]> for Buffer<T> {
    fn from(array: [T; N]) -> Self {
        unsafe {Self::copy_from_ptr(array.as_ptr(), N)}
    }
}

impl<T: Clone> Buffer<T> {
    #[must_use]
    pub fn new() -> Self {Self::default()}

    /// # Safety
    /// ptr must be a valid pointer to a slice of length `len` and must have been allocated with
    /// the global allocator.
    #[must_use]
    pub unsafe fn copy_from_ptr(ptr: *const T, len: usize) -> Self {
        let buf = Self::alloc(len);

        unsafe {
            buf.copy_from_nonoverlapping(ptr, len);
            Self::from_raw_parts(buf, len)
        }
    }

    /// # Safety
    /// ptr must be a valid pointer to a slice of length `len` and must have been allocated with
    /// the global allocator.
    #[must_use]
    pub unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> Self {
        let ptr = ptr::slice_from_raw_parts_mut(ptr, len);
        unsafe {Self(Box::from_raw(ptr))}
    }

    fn alloc(size: usize) -> *mut T {
        unsafe {
            let layout = Layout::array::<T>(size).expect("could not allocate buffer");
            let ptr = alloc(layout).cast::<T>();
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            ptr
        }
    }

    unsafe fn init(ptr: *mut T, n: usize, value: T)
    {
        unsafe {
            for i in 0..n {
                ptr.add(i).write(value.clone());
            }
        }
    }

    pub fn from_elem(elem: T, n: usize) -> Self
    {
        unsafe {
            let ptr = Self::alloc(n);

            Self::init(ptr, n, elem);

            Self::from_raw_parts(ptr, n)
        }
    }

    pub fn resize(&mut self, new_size: usize, value: T) {
        let old_size = self.len();

        if old_size == new_size {return}

        let new_buffer = Self::alloc(new_size);
        unsafe {
            if new_size > old_size {
                new_buffer.copy_from_nonoverlapping(self.0.as_ptr(), old_size);

                Self::init(new_buffer.add(old_size), new_size - old_size, value);
            } else {
                new_buffer.copy_from_nonoverlapping(self.0.as_ptr(), new_size);
            }

            *self = Self::from_raw_parts(new_buffer, new_size);
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod buffer_tests {
    use super::*;

    #[test]
    fn test_buffer_conversion_slice() {
        let slice = &[1, 2, 3, 4, 5] as &[i32];
        let buffer = Buffer::from(slice);
        assert_eq!(&*buffer, slice);
    }

    #[test]
    fn test_buffer_conversion_array() {
        let array = [1, 2, 3, 4, 5];
        let buffer = Buffer::from(array);
        assert_eq!(&*buffer, &array);
    }

    #[test]
    fn test_buffer_conversion_vec() {
        let vec = vec![1, 2, 3, 4, 5];
        let buffer = Buffer::from(vec.clone());
        assert_eq!(&*buffer, &vec[..]);
    }

    #[test]
    fn test_buffer_resize_bigger() {
        let mut buffer = Buffer::from_elem(0, 5);
        buffer.resize(10, 1);
        assert_eq!(&*buffer, &[0, 0, 0, 0, 0, 1, 1, 1, 1, 1]);
    }

    #[test]
    fn test_buffer_clear() {
        let mut buffer = Buffer::from_elem(0, 5);
        buffer.clear();
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_buffer_from_elem() {
        let buffer = Buffer::from_elem(42, 5);
        assert_eq!(&*buffer, &[42, 42, 42, 42, 42]);
    }

    #[test]
    fn test_buffer_from_macro() {
        let buffer = buffer![1, 2, 3, 4, 5];
        assert_eq!(&*buffer, &[1, 2, 3, 4, 5]);

        let buffer = buffer![0; 5];
        assert_eq!(&*buffer, &[0, 0, 0, 0, 0]);

        let buffer: Buffer<i32> = buffer![];
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_buffer_resize_smaller() {
        let mut buffer = Buffer::from_elem(1, 5);
        buffer.resize(3, 0);
        assert_eq!(&*buffer, &[1, 1, 1]);
    }

    #[test]
    fn test_buffer_resize_same() {
        let mut buffer = Buffer::from_elem(1, 5);
        buffer.resize(5, 0);
        assert_eq!(buffer, Buffer::from_elem(1, 5));
    }

    #[test]
    fn test_buffer_equality() {
        let buffer1 = Buffer::from_elem(1, 5);
        let buffer2 = Buffer::from_elem(1, 5);
        let buffer3 = Buffer::from_elem(2, 5);
        assert_eq!(buffer1, buffer2);
        assert_ne!(buffer1, buffer3);
    }

    #[test]
    fn test_buffer_clone() {
        let buffer1 = Buffer::from_elem(1, 5);
        let buffer2 = buffer1.clone();
        assert_eq!(buffer1, buffer2);
    }

    #[test]
    fn test_buffer_index() {
        let buffer = Buffer::from_elem(1, 5);
        assert_eq!(buffer[0], 1);
        assert_eq!(buffer[4], 1);
    }

    #[test]
    fn test_buffer_index_mut() {
        let mut buffer = Buffer::from_elem(1, 5);
        buffer[0] = 2;
        assert_eq!(buffer[0], 2);
    }

    #[test]
    fn test_buffer_conversion_boxed_slice() {
        let boxed_slice: Box<[i32]> = vec![1, 2, 3, 4, 5].into_boxed_slice();
        let buffer = Buffer::from(boxed_slice.clone());
        assert_eq!(&*buffer, &boxed_slice[..]);
    }

    #[test]
    fn test_buffer_slicing() {
        let buffer = Buffer::from_elem(1, 5);
        let slice = &buffer[1..4];
        assert_eq!(slice, &[1, 1, 1]);
    }

    #[test]
    fn test_buffer_mut_slicing() {
        let mut buffer = Buffer::from_elem(1, 5);
        let slice = &mut buffer[1..4];
        slice[0] = 2;
        assert_eq!(&*buffer, &[1, 2, 1, 1, 1]);
    }
}
