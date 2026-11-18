use core::cmp::Ordering;

use crate::rng::{Rng, SimpleRng};

/// # Panics
/// if the iterator is empty or if the number of samples exceeds the f64 max int.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn slope<I>(samples: I) -> f64
where
    I: ExactSizeIterator + Iterator<Item = (f64, f64)> + Clone,
{
    let n: f64 = samples.len() as f64;
    assert!(n as usize == samples.len(), "Sample size exceeds f64 precision");

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
    f: fn(&[T]) -> R,
    cmp: fn(&R, &R) -> Ordering,
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
        self.buffer.clear();
        self.distribution.clear();

        for _ in 0..self.distribution.capacity() {
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
        assert!(!samples.is_empty(), "Cannot create a distribution from an empty sample set");
        assert!(
            samples.len() <= f64::MAX as usize,
            "Sample size exceeds f64 precision"
        );

        Self(samples)
    }

    /// # Panics
    /// if the quantile is not between 0 and 1.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
    #[must_use]
    pub fn quantile(&self, q: f64) -> &T {
        assert!((0.0..=1.0).contains(&q), "Quantile must be between 0 and 1");

        let index = (q * (self.0.len() - 1) as f64).round() as usize;
        self.0.get(index).unwrap()
    }
}
