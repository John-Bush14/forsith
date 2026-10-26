use core::time::Duration;
use std::time::SystemTime;

pub trait Rng<T> {
    fn random() -> T;
}

#[derive(Default)]
pub struct SimpleRng;
impl Rng<u64> for SimpleRng {
    fn random() -> u64 {
        #[cfg(not(miri))]
        let nanos = {
            let now = SystemTime::now();

            let duration_since_epoch = now
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::new(0, 0));
            duration_since_epoch.as_nanos()
        };

        #[cfg(miri)]
        let nanos = {
            use core::hash::{BuildHasher, Hasher};
            use std::hash::RandomState;

            RandomState::new().build_hasher().finish() as u128
        };

        ((nanos ^ (nanos >> 64)) & u128::from(u64::MAX))
            .try_into()
            .unwrap()
    }
}
