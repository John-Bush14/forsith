use core::time::Duration;
use std::time::SystemTime;

pub trait Rng<T> {
    fn random() -> T;
}

#[derive(Default)]
pub struct SimpleRng;
impl Rng<u64> for SimpleRng {
    fn random() -> u64 {
        let now = SystemTime::now();
        let duration_since_epoch = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::new(0, 0));
        let nanos = duration_since_epoch.as_nanos();
        ((nanos ^ (nanos >> 64)) & u128::from(u64::MAX))
            .try_into()
            .unwrap()
    }
}
