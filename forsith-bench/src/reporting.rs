use forsith_shared::rng::{Rng, SimpleRng};

use crate::Bencher;
use std::time::Duration;

const BOOTSTRAP: usize = 100_000;

pub struct BenchmarkReporter<'a> {
    bench_id: String,
    bencher: &'a Bencher,
    batched: bool,
}

impl<'a> BenchmarkReporter<'a> {
    pub fn start(name: String, batched: bool, bencher: &'a Bencher) -> Self {
        let reporter = Self {
            bench_id: name,
            batched,
            bencher,
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

    pub fn warmed_up(&self, elapsed: Duration, iterations: usize) {
        let avg_time = elapsed / iterations as u32;
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

    pub fn finished(&self, samples: &[(Duration, usize)]) {
        let point = Duration::from_nanos(slope(
            samples
                .iter()
                .map(|(duration, iterations)| (*iterations as f64, duration.as_nanos() as f64)),
        ) as u64);

        let mut resampled = Vec::with_capacity(samples.len());
        let mut bootstrap = Vec::from_fn(BOOTSTRAP, |_| {
            resampled.clear();
            resampled.extend(samples.iter().map(|_| {
                let index = SimpleRng::random() as usize % samples.len();
                samples[index]
            }));
            slope(
                resampled
                    .iter()
                    .map(|(duration, iterations)| (*iterations as f64, duration.as_nanos() as f64)),
            )
        });

        bootstrap.sort_by(f64::total_cmp);

        let percentile = |p: f64| bootstrap[((BOOTSTRAP - 1) as f64 * p) as usize];

        self.log(&format!(
            "time: [{:#?} - {:#?} - {:#?}]",
            Duration::from_nanos(percentile(0.025) as u64),
            point,
            Duration::from_nanos(percentile(0.975) as u64)
        ));
    }
}

fn slope<I>(samples: I) -> f64
where
    I: ExactSizeIterator + Iterator<Item = (f64, f64)> + Clone,
{
    let n = samples.len() as f64;

    let (mean_x, mean_y) = samples
        .clone()
        .reduce(|(sum_x, sum_y), (x, y)| (sum_x + x, sum_y + y))
        .map(|(sum_x, sum_y)| (sum_x / n, sum_y / n))
        .expect("Cannot compute slope of empty sample set");

    samples
        .map(|(x, y)| ((x - mean_x) * (y - mean_y), (x - mean_x).powi(2)))
        .reduce(|(sum_xy, sum_x2), (xy, x2)| (sum_xy + xy, sum_x2 + x2))
        .map(|(numerator, denominator)| numerator / denominator)
        .unwrap()
}
