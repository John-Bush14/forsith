use crate::Benchmark;
use forsith_proc::IsVariant;
use forsith_shared::math::{Bootstrapper, least_squares};
use std::time::Duration;

pub struct Wallclock {
    analyzer: WallclockAnalyzer,
    pub warmup_time: Duration,
    pub measurement_limits: Duration,
    pub samples: usize,
}

impl Default for Wallclock {
    fn default() -> Self {
        Self::new()
    }
}

impl Wallclock {
    pub fn new() -> Self {
        Self {
            analyzer: WallclockAnalyzer::new(),
            warmup_time: Duration::from_secs(3),
            measurement_limits: Duration::from_secs(5),
            samples: 100,
        }
    }

    fn choose_sampling_strategy(&self, iter_time: usize) -> SamplingStrategy {
        SamplingStrategy::choose(iter_time, self.measurement_limits.as_nanos().try_into().expect("Measurement limits exceed usize precision"), self.samples)
    }
}

impl Benchmark for Wallclock {
    fn bench(&mut self, id: &str, mut f: impl FnMut()) {
        println!("[{id}] Warming up for ~{:?}...", self.warmup_time);

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
        let iter_time: usize = (elapsed / iterations)
            .as_nanos()
            .try_into()
            .expect("Iteration time exceeds usize precision");

        let mut samples = Vec::with_capacity(self.samples);
        let sampling = self.choose_sampling_strategy(iter_time);

        let total_runs = sampling.total_runs(self.samples);
        println!(
            "Collecting {} samples in estimated {:?} ({} iterations))",
            self.samples,
            Duration::from_nanos((total_runs * iter_time) as u64),
            total_runs
        );

        if sampling.is_constant() {
            println!(
                "Warning! The benchmarked function was too slow to be measured accurately, increase measurement limits or reduce number of samples."
            );
        }

        for iterations in sampling.iter(self.samples) {
            let start = std::time::Instant::now();
            for _ in 0..iterations {
                f();
            }
            samples.push((start.elapsed(), iterations));
        }

        let [l, m, u] = self.analyzer.analyze_samples(&samples, sampling);

        println!(
            "[{id}] Elapsed time per iteration: {:?} (95% CI: {:?} - {:?})",
            m, l, u
        );
    }

    fn bench_with_setup<T>(&mut self, _id: &str, _setup: impl FnMut() -> T, _f: impl FnMut(T)) {
        todo!()
    }
}

pub struct WallclockAnalyzer {
    bootstrapper: Bootstrapper<(Duration, usize), Duration>,
}

impl WallclockAnalyzer {
    pub fn new() -> Self {
        Self {
            bootstrapper: Bootstrapper::new(100_000, |_| panic!(), Duration::cmp),
        }
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

    pub fn analyze_samples(&mut self, samples: &[(Duration, usize)], sampling: SamplingStrategy) -> [Duration; 3] {
        self.choose_slope_func(sampling);

        let middle = self.slope(samples);

        let distribution = self.bootstrapper.bootstrap(samples);
        let lower = *distribution.quantile(0.025);
        let upper = *distribution.quantile(0.975);

        [lower, middle, upper]
    }
}

#[derive(IsVariant)]
pub enum SamplingStrategy {
    Linear(usize),
    Constant(usize),
}

impl SamplingStrategy {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    #[must_use]
    pub fn sample(&self, i: usize) -> usize {
        match self {
            Self::Linear(d) => d * i,
            Self::Constant(n) => *n,
        }
    }

    pub fn iter(&self, samples: usize) -> impl Iterator<Item = usize> + '_ {
        (1..=samples).map(move |i| self.sample(i))
    }

    pub fn total_runs(&self, samples: usize) -> usize {
        match self {
            Self::Linear(d) => d * samples * (samples + 1) / 2,
            Self::Constant(n) => n * samples,
        }
    }

    pub fn choose(iter_time: usize, measurement_limits: usize, samples: usize) -> Self {
        let max_iters = measurement_limits / iter_time;
        let linear_iters = samples * (samples + 1) / 2;

        if linear_iters <= max_iters {
            Self::Linear(max_iters / linear_iters)
        } else {
            Self::Constant(max_iters.div_ceil(samples))
        }
    }
}
