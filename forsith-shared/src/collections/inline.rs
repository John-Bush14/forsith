use crate::int::UInt;
use core::{
    mem::MaybeUninit,
    ops::{Index, RangeBounds},
    slice::SliceIndex,
};
use forsith_proc::{Deref, DerefMut};

#[derive(Deref, DerefMut)]
pub struct InlineArray<T, const N: usize>([MaybeUninit<T>; N]);

impl<T, const N: usize> Default for InlineArray<T, N> {
    fn default() -> Self {
        Self(unsafe { MaybeUninit::uninit().assume_init() })
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
impl<T, const N: usize> InlineArray<T, N> {
    /// # Safety
    /// The caller must ensure the values at the given indices are initialized.
    pub unsafe fn get<I: SliceIndex<[T]>>(&self, index: I) -> &I::Output {
        &(&*(&raw const self.0).cast::<[T; N]>())[index]
    }

    /// # Safety
    /// The caller must ensure the values at the given indices are initialized.
    ///
    /// WARNING: don't use this to set an uninitialized value, use `set` instead.
    pub unsafe fn get_mut<I: SliceIndex<[T]>>(&mut self, index: I) -> &mut I::Output {
        &mut (&mut *(&raw mut self.0).cast::<[T; N]>())[index]
    }

    pub const fn set(&mut self, index: usize, value: T) {
        self.0[index] = MaybeUninit::new(value);
    }

    /// # Safety
    /// The caller must ensure the value at the given index is initialized.
    pub const unsafe fn remove(&mut self, index: usize) -> T {
        self.0[index].assume_init_read()
    }

    pub fn move_within(&mut self, range: core::ops::Range<usize>, new_index: usize) {
        unsafe {
            core::ptr::copy(
                self.0.as_ptr().add(range.start),
                self.0.as_mut_ptr().add(new_index),
                range.len(),
            );
        };
    }

    pub fn move_to_other(
        &mut self,
        range: core::ops::Range<usize>,
        other: &mut Self,
        other_index: usize,
    ) {
        unsafe {
            core::ptr::copy(
                self.0.as_ptr().add(range.start),
                other.0.as_mut_ptr().add(other_index),
                range.len(),
            );
        };
    }
}

pub struct InlineVec<T, const N: usize, L: UInt> {
    array: InlineArray<T, N>,
    len: L,
}

impl<T, const N: usize, L: UInt> Default for InlineVec<T, N, L> {
    fn default() -> Self {
        Self {
            array: InlineArray::default(),
            len: L::from(0u8),
        }
    }
}

impl<T, const N: usize, L: UInt> InlineVec<T, N, L> {
    pub const fn len(&self) -> L {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == L::from(0u8)
    }

    pub fn get(&self, index: L) -> Option<&T> {
        (index < self.len).then(|| unsafe { self.array.get(index.as_usize()) })
    }

    pub fn get_mut(&mut self, index: L) -> Option<&mut T> {
        (index < self.len).then(|| unsafe { self.array.get_mut(index.as_usize()) })
    }

    pub const fn set_len(&mut self, len: L) {
        self.len = len;
    }

    /// # Panics
    /// Panics if the length exceeds the capacity of the `InlineVec`.
    pub fn push(&mut self, value: T) {
        assert!(
            self.len.as_usize() < N,
            "InlineVec length exceeded capacity of {N}"
        );

        self.array.set(self.len.as_usize(), value);
        self.len += L::from(1u8);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.len = self.len.checked_sub(L::from(1u8))?;
        unsafe { self.array.remove(self.len.as_usize()) }.into()
    }

    pub fn remove(&mut self, index: L) -> Option<T> {
        (index < self.len).then(|| {
            let range = index.as_usize()..self.len.as_usize();
            unsafe { self.array[range].shift_left([MaybeUninit::uninit()])[0].assume_init_read() }
        })
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn remove_range(&mut self, range: impl RangeBounds<L>) {
        let range = self.bounded_range(range);
        let len = self.len.as_usize();
        self.len -= L::try_from(range.len()).ok().unwrap();

        self.array.move_within(range.end..len, range.start);
    }

    fn bounded_range(&self, range: impl RangeBounds<L>) -> core::ops::Range<usize> {
        let start = match range.start_bound() {
            core::ops::Bound::Included(start) => start.as_usize(),
            core::ops::Bound::Excluded(start) => start.as_usize() + 1,
            core::ops::Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            core::ops::Bound::Included(end) => end.as_usize() + 1,
            core::ops::Bound::Excluded(end) => end.as_usize(),
            core::ops::Bound::Unbounded => self.len.as_usize(),
        };

        start..end
    }
}

impl<T, const N: usize, L: UInt> Index<L> for InlineVec<T, N, L> {
    type Output = T;

    fn index(&self, index: L) -> &Self::Output {
        self.get(index).unwrap_or_else(|| {
            panic!(
                "Index {} out of bounds for InlineVec of length {}",
                index.as_usize(),
                self.len.as_usize()
            )
        })
    }
}
