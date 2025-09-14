use std::time::Duration;

use futures::Stream;
use futures_signals::signal::{Mutable, SignalExt};
use tokio::time::sleep;

pub struct Timer {
    current: Mutable<sdk::Time>,
    timeout: Duration,
}

impl Timer {
    pub fn new(timeout: Duration) -> Self {
        Self {
            current: Mutable::new(sdk::Time::unknown()),
            timeout,
        }
    }

    pub fn update(&self, time: sdk::Time) {
        self.current.set(time);
    }

    pub fn stream(&self) -> impl Stream<Item = sdk::Time> {
        self.current
            .signal()
            .throttle(|| sleep(self.timeout))
            .to_stream()
    }

    pub fn reset(&self) {
        self.current.set(sdk::Time::unknown());
    }
}
