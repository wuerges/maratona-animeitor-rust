//! Healthy public streams have no age limit. Only missing pong responses expire.
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, stream::SplitSink};
use std::time::Duration;
use tokio::time::{Instant, Interval, MissedTickBehavior};

pub(crate) struct Heartbeat {
    ticks: Interval,
    last_pong: Instant,
    pending: Option<[u8; 8]>,
    sequence: u64,
    budget: crate::remote_control::MessageBudget,
}
impl Heartbeat {
    pub fn new() -> Self {
        let now = Instant::now();
        let period = Duration::from_secs(30);
        let mut ticks = tokio::time::interval_at(now + period, period);
        ticks.set_missed_tick_behavior(MissedTickBehavior::Delay);
        Self {
            ticks,
            last_pong: now,
            pending: None,
            sequence: 0,
            budget: crate::remote_control::MessageBudget::new(),
        }
    }
    pub async fn tick(&mut self) {
        self.ticks.tick().await;
    }
    pub fn deadline(&self) -> Instant {
        self.last_pong + Duration::from_secs(90)
    }
    fn ping(&mut self) -> Option<Message> {
        if Instant::now() >= self.deadline() {
            return None;
        }
        let payload = *self.pending.get_or_insert_with(|| {
            self.sequence = self.sequence.wrapping_add(1);
            self.sequence.to_be_bytes()
        });
        Some(Message::Ping(payload.to_vec().into()))
    }
    fn pong(&mut self, payload: &[u8]) {
        if self
            .pending
            .as_ref()
            .is_some_and(|expected| expected.as_slice() == payload)
        {
            self.last_pong = Instant::now();
            self.pending = None;
        }
    }
    pub async fn send_ping(&mut self, sender: &mut SplitSink<WebSocket, Message>) -> bool {
        let Some(message) = self.ping() else {
            return false;
        };
        send(sender, message).await
    }
    pub fn allow_frame(&mut self) -> bool {
        self.budget.allow()
    }
    pub async fn receive(
        &mut self,
        incoming: Option<Result<Message, axum::Error>>,
        sender: &mut SplitSink<WebSocket, Message>,
    ) -> bool {
        if !self.allow_frame() {
            return false;
        }
        match incoming {
            Some(Ok(Message::Pong(payload))) => {
                self.pong(&payload);
                true
            }
            Some(Ok(Message::Ping(payload))) => send(sender, Message::Pong(payload)).await,
            // These public streams accept control frames, not application input.
            _ => false,
        }
    }
}
impl Drop for Heartbeat {
    fn drop(&mut self) {
        if Instant::now() >= self.deadline() {
            service::resource_metrics::event("heartbeat_expired");
        }
    }
}

async fn send(sender: &mut SplitSink<WebSocket, Message>, message: Message) -> bool {
    matches!(
        tokio::time::timeout(Duration::from_secs(10), sender.send(message)).await,
        Ok(Ok(()))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test(start_paused = true)]
    async fn responsive_browser_stays_healthy_for_eight_hours() {
        let mut heartbeat = Heartbeat::new();
        let started = Instant::now();
        for _ in 0..960 {
            heartbeat.tick().await;
            let Message::Ping(payload) =
                heartbeat.ping().expect("healthy browser remains connected")
            else {
                panic!("expected ping");
            };
            heartbeat.pong(&payload);
        }
        assert!(Instant::now().duration_since(started) >= Duration::from_secs(8 * 3600));
        assert!(heartbeat.deadline() > Instant::now());
    }
    #[tokio::test(start_paused = true)]
    async fn unmatched_pongs_do_not_extend_deadline() {
        let mut heartbeat = Heartbeat::new();
        heartbeat.tick().await;
        assert!(heartbeat.ping().is_some());
        heartbeat.pong(b"wrong challenge");
        tokio::time::sleep_until(heartbeat.deadline()).await;
        assert!(heartbeat.ping().is_none());
    }
}
