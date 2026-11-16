use core::time::Duration;

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
        let reporter = BenchmarkReporter::start(name.to_string(), false, self);

        let mut iterations = 1;
        let mut elapsed = Duration::ZERO;
        while elapsed < self.warmup_time {
            let start = std::time::Instant::now();
            for _ in 0..iterations {
                f();
            }
            elapsed += start.elapsed();

            iterations *= 2;
        }

        reporter.warmed_up(elapsed, iterations);

        let mut samples = Vec::with_capacity(self.samples);
        for iterations in self.choose_sampling(elapsed, iterations * 2 - 1) {
            let start = std::time::Instant::now();
            for _ in 0..iterations {
                f();
            }
            samples.push((start.elapsed(), iterations));
        }

        reporter.finished(&samples);
    }

    fn choose_sampling(&self, elapsed: Duration, iterations: usize) -> impl Iterator<Item = usize> {
        let iter_time = (elapsed / iterations as u32).as_nanos() as f64;
        let target_time = self.measurement_limits.as_nanos() as f64;
        let total_runs = (self.samples * (self.samples + 1)) as f64 / 2.0;

        let d = (target_time / iter_time / total_runs).ceil().max(1.0);
        let linear_time = d * total_runs * iter_time;

        let choose_linear = linear_time < target_time * 2.0;
        let base = (!choose_linear)
            .then_some(target_time / iter_time / self.samples as f64)
            .map(|x| x.round() as usize);

        (1..=self.samples).map(move |i| {
            if choose_linear {
                (d * i as f64).round() as usize
            } else {
                base.unwrap()
            }
        })
    }
}
