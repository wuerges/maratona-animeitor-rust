//! Browser retry policy; pure inputs keep timing tests deterministic.
#[derive(Default)]
pub(crate) struct Retry {
    attempt: u32,
}
impl Retry {
    pub fn stable_for(&mut self, milliseconds: f64) {
        if milliseconds >= 30_000.0 {
            self.reset();
        }
    }
    pub fn reset(&mut self) {
        self.attempt = 0;
    }
    pub fn delay_ms(&mut self, random: f64) -> u32 {
        let base = (1000u32.saturating_mul(1u32 << self.attempt.min(5))).min(30_000);
        self.attempt = self.attempt.saturating_add(1);
        base / 2 + ((base / 2) as f64 * random.clamp(0.0, 1.0)) as u32
    }
    pub async fn wait(&mut self) {
        gloo_timers::future::TimeoutFuture::new(self.delay_ms(random())).await;
    }
}
#[cfg(target_arch = "wasm32")]
pub fn random() -> f64 {
    js_sys::Math::random()
}
#[cfg(not(target_arch = "wasm32"))]
pub fn random() -> f64 {
    0.5
}
#[cfg(target_arch = "wasm32")]
pub fn now_ms() -> f64 {
    js_sys::Date::now()
}
#[cfg(not(target_arch = "wasm32"))]
pub fn now_ms() -> f64 {
    0.0
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backoff_is_bounded_jittered_and_resettable() {
        let mut low = Retry::default();
        let mut high = Retry::default();
        for expected in [1000, 2000, 4000, 8000, 16000, 30000, 30000] {
            assert_eq!(low.delay_ms(0.0), expected / 2);
            assert_eq!(high.delay_ms(1.0), expected);
        }
        high.stable_for(29_999.0);
        assert_eq!(high.delay_ms(1.0), 30_000);
        high.stable_for(30_000.0);
        assert_eq!(high.delay_ms(1.0), 1000);
        let samples: std::collections::HashSet<_> = (0..100)
            .map(|i| Retry::default().delay_ms(i as f64 / 100.0))
            .collect();
        assert_eq!(samples.len(), 100);
    }
}
