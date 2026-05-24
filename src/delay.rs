//! A flexible delay generator with a minimum guaranteed delay, variable extra delay
use anyhow::Result;
use rand::Rng;
use rand_distr::{Distribution, Gamma};
use std::time::Duration;

/// Delay model descriptor.
///
/// Generates delays with:
/// - a strict minimum delay,
/// - a variable extra delay sampled from a gamma distribution,
///
/// This is useful when you need delays that are not perfectly regular while
/// still guaranteeing that every delay is at least `min_delay`.
///
/// # Delay model
///
/// The generated delay is:
///
/// `min_delay + gamma_sample`
///
/// Where:
/// - `min_delay` is always enforced,
/// - `gamma_sample` provides most of the variability,
///
/// # Notes
///
/// - The gamma distribution is a good fit for generating positive-only extra
///   delays with a right-skewed shape.
/// - `short_shape * short_scale_ms` is approximately the mean extra delay in
///   milliseconds
/// - If `max_delay` is set, the final result is clamped to that maximum.
#[derive(Debug, Clone)]
pub struct DelayModel {
    /// Absolute lower bound for every generated delay.
    min_delay: Duration,

    /// Optional upper bound for the final generated delay.
    max_delay: Option<Duration>,

    /// Gamma distribution shape parameter for the "normal" extra delay.
    ///
    /// Must be positive.
    short_shape: f64,

    /// Gamma distribution scale parameter, expressed in milliseconds.
    ///
    /// Must be positive.
    short_scale_ms: f64,
}

impl DelayModel {
    /// Creates a new delay model with sensible defaults.
    ///
    /// The provided `min_delay` is always enforced.
    ///
    /// # Default behavior
    ///
    /// - `max_delay`: none
    /// - short gamma shape: `2.0`
    /// - short gamma scale: `250.0 ms`
    ///
    /// # Example
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let model = DelayModel::new(Duration::from_millis(800));
    /// ```
    pub fn new(min_delay: Duration) -> Self {
        Self {
            min_delay,
            max_delay: None,
            short_shape: 2.0,
            short_scale_ms: 250.0,
        }
    }

    /// Sets an upper bound for generated delays.
    ///
    /// If a generated delay exceeds `max_delay`, it will be clamped down to
    /// that value.
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = Some(max_delay);
        self
    }

    /// Configures the gamma distribution used for the normal extra delay.
    ///
    /// `shape` and `scale_ms` must both be positive.
    ///
    /// Approximate mean extra delay:
    ///
    /// `shape * scale_ms`
    ///
    /// # Example
    ///
    /// A shape of `2.0` and a scale of `300.0` gives an average extra delay
    /// of about `600 ms`.
    pub fn with_short_gamma(mut self, shape: f64, scale_ms: f64) -> Self {
        self.short_shape = shape;
        self.short_scale_ms = scale_ms;
        self
    }

    /// Generates the next delay value.
    ///
    /// The returned delay is always at least `min_delay`.
    ///
    /// # How it works
    ///
    /// 1. Sample a gamma-distributed extra delay in milliseconds.
    /// 2. If `max_delay` is configured, clamp the result to that maximum.
    pub fn next_delay<R: Rng + ?Sized>(&self, rng: &mut R) -> Result<Duration> {
        let gamma = Gamma::new(self.short_shape, self.short_scale_ms)?;

        // Sample the ordinary extra delay and round it to whole milliseconds.
        let extra_ms = gamma.sample(rng).round() as u64;

        let total_ms = u64::try_from(self.min_delay.as_millis())? + extra_ms;
        let delay = Duration::from_millis(total_ms);

        match self.max_delay {
            Some(max) => Ok(delay.min(max)),
            Option::None => Ok(delay),
        }
    }
}
