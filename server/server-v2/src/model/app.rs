use std::time::Duration;

use super::runs::Runs;
use super::timer::Timer;

pub struct App {
    pub runs: Runs,
    pub time: Timer,

    timeout: Duration,
}

impl App {
    pub fn new(timeout: Duration) -> Self {
        Self {
            runs: Runs::new(timeout),
            time: Timer::new(timeout),

            timeout,
        }
    }
}
