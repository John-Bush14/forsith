use core::{cmp::Ordering, time::Duration};

use crate::rng::{Rng, SimpleRng};

/// # Panics
/// if the iterator is empty or if the number of samples exceeds the f64 max int.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn slope<I>(samples: I) -> f64
where
    I: ExactSizeIterator + Iterator<Item = (f64, f64)> + Clone,
{
    let n: f64 = samples.len() as f64;
    assert!(
        n as usize == samples.len(),
        "Sample size exceeds f64 precision"
    );

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

pub struct Bootstrapper<T, R> {
    distribution: Vec<R>,
    buffer: Vec<T>,
    pub f: fn(&[T]) -> R,
    pub cmp: fn(&R, &R) -> Ordering,
}

impl<T, R> Bootstrapper<T, R>
where
    T: Clone,
{
    pub fn new(samples: usize, f: fn(&[T]) -> R, cmp: fn(&R, &R) -> Ordering) -> Self {
        Self {
            distribution: Vec::with_capacity(samples),
            buffer: Vec::new(),
            f,
            cmp,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn bootstrap(&mut self, samples: &[T]) -> Distribution<'_, R> {
        self.buffer.reserve(samples.len());
        self.distribution.clear();

        for _ in 0..self.distribution.capacity() {
            self.buffer.clear();
            self.buffer.extend(samples.iter().map(|_| {
                let index = SimpleRng::random() as usize % samples.len();
                samples[index].clone()
            }));
            let result = (self.f)(&self.buffer);
            self.distribution.push(result);
        }

        self.sort();

        Distribution::new(&self.distribution)
    }

    fn sort(&mut self) {
        self.distribution.sort_by(self.cmp);
    }
}

pub struct Distribution<'a, T>(&'a [T]);

impl<'a, T> Distribution<'a, T> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn new(samples: &'a [T]) -> Self {
        assert!(
            !samples.is_empty(),
            "Cannot create a distribution from an empty sample set"
        );
        assert!(
            samples.len() <= f64::MAX as usize,
            "Sample size exceeds f64 precision"
        );

        Self(samples)
    }

    /// # Panics
    /// if the quantile is not between 0 and 1.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    #[must_use]
    pub fn quantile(&self, q: f64) -> &T {
        assert!((0.0..=1.0).contains(&q), "Quantile must be between 0 and 1");

        let index = (q * (self.0.len() - 1) as f64).round() as usize;
        self.0.get(index).unwrap()
    }
}

pub enum SamplingStrategy {
    Linear(f64),
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
            Self::Linear(d) => (d * i as f64).round() as usize,
            Self::Constant(n) => *n,
        }
    }

    pub fn iter(&self, samples: usize) -> impl Iterator<Item = usize> + '_ {
        (1..=samples).map(move |i| self.sample(i))
    }

    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn choose(iter_time: Duration, samples: usize, target_time: Duration) -> Self {
        let iter_time = iter_time.as_nanos() as f64;
        let target_time = target_time.as_nanos() as f64;
        let total_runs = (samples * (samples + 1) / 2) as f64;

        let d = (target_time / iter_time / total_runs).ceil().max(1.0);
        let linear_time = d * total_runs * iter_time;

        if linear_time < target_time * 2.0 {
            Self::Linear(d)
        } else {
            let base = (target_time / iter_time / samples as f64).round() as usize;
            Self::Constant(base)
        }
    }
}
