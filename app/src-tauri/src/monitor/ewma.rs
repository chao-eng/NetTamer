//! Exponentially weighted moving average used to smooth instantaneous rates.

/// `rate_t = a * instant + (1 - a) * prev`, where `a = 2 / (n + 1)`.
///
/// A larger `n` (window sample count) yields a smoother, slower-responding
/// average. The architecture default is `n = 5` (5-second window @ 1s ticks).
pub fn ewma(prev: f64, instant: f64, n: usize) -> f64 {
    if n == 0 {
        return instant;
    }
    let a = 2.0 / ((n as f64) + 1.0);
    a * instant + (1.0 - a) * prev
}

#[cfg(test)]
mod tests {
    use super::ewma;

    #[test]
    fn first_sample_tracks_instant() {
        // With prev = 0 and instant = 100, n = 1 -> a = 1.0 -> 100.
        assert_eq!(ewma(0.0, 100.0, 1), 100.0);
    }

    #[test]
    fn smoothing_reduces_spike() {
        let v = ewma(100.0, 1000.0, 5);
        assert!(v > 100.0 && v < 1000.0);
    }
}
