use forsith_shared::{
    math::{Bootstrapper, SamplingStrategy, least_squares},
    rng::{Rng, SimpleRng},
};

use crate::Bencher;
use std::time::Duration;

pub struct BenchmarkReporter<'a> {
    bench_id: String,
    bencher: &'a Bencher,
    batched: bool,
    bootstrapper: Bootstrapper<(Duration, usize), Duration>,
}

impl<'a> BenchmarkReporter<'a> {
    pub fn start(name: String, batched: bool, bencher: &'a Bencher) -> Self {
        let reporter = Self {
            bench_id: name,
            batched,
            bencher,
            bootstrapper: Bootstrapper::new(100_000, |_| panic!(), Duration::cmp),
        };

        println!("{}", reporter.bench_id);
        reporter.log(&format!(
            "Warmup time: {:?}, Measurement limits: {:?}",
            reporter.bencher.warmup_time, reporter.bencher.measurement_limits
        ));

        reporter
    }

    fn log(&self, message: &str) {
        println!("[{}] {}", self.bench_id, message);
    }

    pub fn warmed_up(&self, elapsed: Duration, iterations: u32) {
        let avg_time = elapsed / iterations;
        let iterations =
            (self.bencher.measurement_limits.as_secs_f64() / avg_time.as_secs_f64()) as usize;

        self.log(&if self.batched {
            format!(
                "Collecting for ~{} s in estimated {iterations} iterations",
                self.bencher.measurement_limits.as_secs_f64()
            )
        } else {
            format!(
                "collecting {iterations} samples in estimated {} s",
                self.bencher.measurement_limits.as_secs_f64()
            )
        })
    }

    fn slope(&self, samples: &[(Duration, usize)]) -> Duration {
        (self.bootstrapper.f)(samples)
    }

    fn choose_slope_func(&mut self, sampling: SamplingStrategy) {
        self.bootstrapper.f = match sampling {
            SamplingStrategy::Linear(_) => |samples| {
                Duration::from_nanos(least_squares(
                    samples
                        .iter()
                        .map(|(time, iterations)| (*iterations as f64, time.as_nanos() as f64)),
                ) as u64)
            },
            SamplingStrategy::Constant(_) => |samples| {
                let (total_time, total_iters) = samples
                    .iter()
                    .cloned()
                    .reduce(|(st, si), (t, i)| (st + t, si + i))
                    .expect("samples should not be empty");

                total_time / total_iters as u32
            },
        };
    }

    pub fn finished(&mut self, samples: &[(Duration, usize)], sampling: SamplingStrategy) {
        self.choose_slope_func(sampling);

        let middle = self.slope(samples);

        let distribution = self.bootstrapper.bootstrap(samples);
        let lower = *distribution.quantile(0.025);
        let upper = *distribution.quantile(0.975);

        self.log(&format!("time: [{lower:#?} - {middle:#?} - {upper:#?}]",));
    }
}
