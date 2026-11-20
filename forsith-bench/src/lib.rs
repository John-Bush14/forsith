use core::time::Duration;
use std::io::Write;

use forsith_shared::escape;

pub trait Benchmark {
    fn bench(&mut self, id: &str, mut f: impl FnMut(), reporter: &Reporter) {
        self.bench_with_setup(id, || (), |()| f(), reporter);
    }

    fn bench_with_setup<T>(
        &mut self,
        id: &str,
        setup: impl FnMut() -> T,
        f: impl FnMut(T),
        reporter: &Reporter,
    );
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
        let reporter = Reporter(name.to_string());

        self.wallclock.bench(name, f, &reporter);
    }
}

pub struct Reporter(String);

impl Reporter {
    pub fn start(&self, msg: &str) {
        print!("{}", self.message(msg));
        std::io::stdout().flush().expect("Failed to flush stdout");
    }

    fn message(&self, msg: &str) -> String {
        format!("{}{}:{} {}", escape::GREEN, self.0, escape::RESET, msg)
    }

    pub fn update(&self, msg: &str) {
        print!("{}\r{}", escape::CLEAR_LINE, self.message(msg));
        std::io::stdout().flush().expect("Failed to flush stdout");
    }

    pub fn finish(&self, msg: &str) {
        self.update(&format!("\n\t\t{}\n", msg.replace("\n", "\n\t\t")));
    }
}
