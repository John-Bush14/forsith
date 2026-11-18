use core::time::Duration;

use forsith_shared::math::SamplingStrategy;

use crate::reporting::BenchmarkReporter;

mod reporting;

impl Default for Bencher {
    fn default() -> Self {
        Self {
            warmup_time: Duration::from_secs(1),
            measurement_limits: Duration::from_secs(5),
            samples: 100,
        }
    }
}

pub struct Bencher {
    pub warmup_time: Duration,
    pub measurement_limits: Duration,
    pub samples: usize,
}

impl Bencher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bench<F>(&self, name: &str, mut f: F)
    where
        F: FnMut(),
    {
        let mut reporter = BenchmarkReporter::start(name.to_string(), false, self);

        let mut iterations = 1u32;
        let mut elapsed = Duration::ZERO;
        while elapsed < self.warmup_time {
            let start = std::time::Instant::now();
            for _ in 0..iterations {
                f();
            }
            elapsed += start.elapsed();

            iterations = iterations
                .checked_mul(2)
                .expect("Tested function is too fast to benchmark");
        }

        reporter.warmed_up(elapsed, iterations);

        let mut samples = Vec::with_capacity(self.samples);
        let sampling = self.choose_sampling_strategy(elapsed, iterations);

        for iterations in sampling.iter(self.samples) {
            let start = std::time::Instant::now();
            for _ in 0..iterations {
                f();
            }
            samples.push((start.elapsed(), iterations));
        }

        reporter.finished(&samples, sampling);
    }

    fn choose_sampling_strategy(&self, elapsed: Duration, iterations: u32) -> SamplingStrategy {
        SamplingStrategy::choose(elapsed / iterations, self.samples, self.measurement_limits)
    }
}
