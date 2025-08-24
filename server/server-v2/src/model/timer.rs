use std::time::Duration;

use futures::Stream;
use futures_signals::signal::{Mutable, SignalExt};
use tokio::time::sleep;

pub struct Timer {
    current: Mutable<data::TimeFile>,
    timeout: Duration,
}

impl Timer {
    pub fn new(timeout: Duration) -> Self {
        Self {
            current: Mutable::new(-1),
            timeout,
        }
    }

    pub fn update(&self, time: data::TimeFile) {
        self.current.set(time);
    }

    pub fn stream(&self) -> impl Stream<Item = data::TimeFile> {
        self.current
            .signal()
            .throttle(|| sleep(self.timeout))
            .to_stream()
    }
}
