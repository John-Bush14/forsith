use std::marker::PhantomData;

use crate::bitvec::BitArray;

#[repr(transparent)]
#[derive(Debug, Clone, Default)]
pub struct Bitflags<S: BitStorage, F: BitFlag>(S, PhantomData<F>);

impl<S: BitStorage, F: BitFlag> Bitflags<S, F> {
    #[must_use]
    pub fn new(flags_slice: &[F]) -> Self {
        assert!(
            S::MAX_BITS >= F::MAX_BITS,
            "BitStorage does not have enough bits to store all flags"
        );

        let mut flags = Self::empty();
        flags.set_flags(flags_slice, true);
        flags
    }

    #[must_use]
    pub fn empty() -> Self {
        Self(S::default(), PhantomData)
    }

    #[must_use]
    pub fn get(&self, flag: &F) -> bool {
        let index = flag.bit_index();
        self.0.get_bit(index)
    }

    pub fn set(&mut self, flag: &F, value: bool) {
        let index = flag.bit_index();
        self.0.set_bit(index, value);
    }

    pub fn set_flags(&mut self, flags: &[F], value: bool) {
        for flag in flags {
            self.set(flag, value);
        }
    }

    pub const fn storage(&self) -> &S {
        &self.0
    }
    pub const fn storage_mut(&mut self) -> &mut S {
        &mut self.0
    }
    pub fn into_storage(self) -> S {
        self.0
    }
    pub const fn from_storage(storage: S) -> Self {
        Self(storage, PhantomData)
    }
}

impl<S: BitStorage, F: BitFlag> From<S> for Bitflags<S, F> {
    fn from(storage: S) -> Self {
        Self::from_storage(storage)
    }
}

impl<S: BitStorage, F: BitFlag + BitFlagFromIndex> Bitflags<S, F> {
    #[must_use]
    pub fn flags(&self, value: bool) -> Vec<F> {
        let mut flags = Vec::new();
        for index in 0..S::MAX_BITS {
            if self.0.get_bit(index) == value
                && let Some(flag) = F::from_index(index)
            {
                flags.push(flag);
            }
        }
        flags
    }
}

pub trait BitStorage: Default + Clone {
    const MAX_BITS: usize;

    fn get_bit(&self, index: usize) -> bool;
    fn set_bit(&mut self, index: usize, value: bool);
}

pub trait BitFlag {
    const MAX_BITS: usize;

    fn bit_index(&self) -> usize;
}

pub trait BitFlagFromIndex: BitFlag {
    fn from_index(index: usize) -> Option<Self>
    where
        Self: Sized;
}

impl BitStorage for u32 {
    const MAX_BITS: usize = 32;

    fn get_bit(&self, index: usize) -> bool {
        assert!(index < Self::MAX_BITS, "Index out of bounds");
        (self & (1 << index)) != 0
    }

    fn set_bit(&mut self, index: usize, value: bool) {
        assert!(index < Self::MAX_BITS, "Index out of bounds");
        if value {
            *self |= 1 << index;
        } else {
            *self &= !(1 << index);
        }
    }
}

impl<const N: usize> BitStorage for BitArray<N> {
    const MAX_BITS: usize = N;

    fn get_bit(&self, index: usize) -> bool {
        self[index]
    }

    fn set_bit(&mut self, index: usize, value: bool) {
        self.set(index, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TestFlag {
        A,
        B,
        C,
    }

    impl BitFlag for TestFlag {
        const MAX_BITS: usize = 3;

        fn bit_index(&self) -> usize {
            match self {
                Self::A => 0,
                Self::B => 1,
                Self::C => 2,
            }
        }
    }

    impl BitFlagFromIndex for TestFlag {
        fn from_index(index: usize) -> Option<Self>
        where
            Self: Sized,
        {
            match index {
                0 => Some(Self::A),
                1 => Some(Self::B),
                2 => Some(Self::C),
                _ => None,
            }
        }
    }

    #[test]
    fn bitflags_set_and_get() {
        let mut flags = Bitflags::<u32, TestFlag>::empty();

        flags.set(&TestFlag::A, true);
        flags.set(&TestFlag::C, true);

        assert!(flags.get(&TestFlag::A));
        assert!(!flags.get(&TestFlag::B));
        assert!(flags.get(&TestFlag::C));
    }

    #[test]
    fn bitflags_new_and_flags() {
        let flags = Bitflags::<u32, TestFlag>::new(&[TestFlag::B, TestFlag::C]);

        assert!(!flags.get(&TestFlag::A));
        assert!(flags.get(&TestFlag::B));
        assert!(flags.get(&TestFlag::C));
        assert_eq!(flags.flags(true), vec![TestFlag::B, TestFlag::C]);
        assert_eq!(flags.flags(false), vec![TestFlag::A]);
    }
}
