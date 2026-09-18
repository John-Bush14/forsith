use core::time::Duration;
use std::time::SystemTime;

pub trait Rng<T> {
    fn random() -> T;
}

pub struct SimpleRng;
impl Rng<u64> for SimpleRng {
    fn random() -> u64 {
        let now = SystemTime::now();
        let duration_since_epoch = now.duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::new(0, 0));
        let nanos = duration_since_epoch.as_nanos() as u64;
        let random_state = nanos ^ (nanos >> 32);
        random_state
    }
}
