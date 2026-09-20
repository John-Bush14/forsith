use crate::rng::{Rng, SimpleRng};
use core::hash::{BuildHasher, Hasher};

pub type DefaultHasher = SimpleHasher;

pub struct RandomState<H: StateHasher = SimpleHasher> {
    state: u64,
    _hasher: core::marker::PhantomData<H>,
}

impl<H: StateHasher> RandomState<H> {
    #[must_use]
    pub fn new<R: Rng<u64>>() -> Self {
        Self {
            state: R::random(),
            _hasher: core::marker::PhantomData,
        }
    }
}

impl<H: StateHasher> Default for RandomState<H> {
    fn default() -> Self {
        Self::new::<SimpleRng>()
    }
}

impl<H: StateHasher> BuildHasher for RandomState<H> {
    type Hasher = H;

    fn build_hasher(&self) -> Self::Hasher {
        H::new(self.state)
    }
}

#[derive(Default)]
pub struct SimpleHasher {
    state: u64,
}

impl Hasher for SimpleHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.state = self
                .state
                .wrapping_mul(0x10000_0001b3)
                .wrapping_add(u64::from(byte));
        }
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

impl StateHasher for SimpleHasher {
    fn new(state: u64) -> Self {
        Self { state }
    }
}

pub trait StateHasher: Hasher {
    fn new(state: u64) -> Self;
}
