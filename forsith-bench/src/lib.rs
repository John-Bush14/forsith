use core::time::Duration;

pub trait Benchmark {
    fn bench(&mut self, id: &str, mut f: impl FnMut()) {
        self.bench_with_setup(id, || (), |()| f());
    }

    fn bench_with_setup<T>(&mut self, id: &str, setup: impl FnMut() -> T, f: impl FnMut(T));
}

mod wallclock;

#[derive(Default)]
pub struct Bencher {
    wallclock: wallclock::Wallclock,
}

impl Bencher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bench<F>(&mut self, name: &str, f: F)
    where
        F: FnMut(),
    {
        self.wallclock.bench(name, f);
    }
}
