use crate::{envelope::send_json, websocket_liveness::Heartbeat};
use axum::extract::ws::{Message, WebSocketUpgrade};
use axum::response::Response;
use data::remote_control::ControlMessage;
use futures::StreamExt;
use service::remote_control::{ConnectionControlMessage, RemoteConnection, next_request_id};

fn valid_message(message: &ControlMessage) -> bool {
    match message {
        ControlMessage::QueryString(q) => q.query.len() <= 2048,
        ControlMessage::PhotoState(data::remote_control::PhotoState::Show(team)) => {
            team.len() <= 256
        }
        _ => true,
    }
}

pub(crate) async fn relay_remote_control(
    mut connection: RemoteConnection,
    ws: WebSocketUpgrade,
    permit: tokio::sync::OwnedSemaphorePermit,
) -> Response {
    crate::resource_limits::websocket(ws).on_upgrade(move |socket| async move {
        let _permit = permit;
        let _active = service::resource_metrics::Active::new("remote_socket");
        let sender = connection.sender.clone();
        let mut rec = sender.subscribe();
        let request_id = next_request_id();
        let (mut output, mut input) = socket.split();
        let mut heartbeat = Heartbeat::new();
        loop {
            tokio::select! {
                _ = tokio::time::sleep_until(connection.expires) => break,
                _ = connection.revoked.changed() => break,
                _ = tokio::time::sleep_until(heartbeat.deadline()) => break,
                _ = heartbeat.tick() => {
                    if service::remote_control::unix_seconds() >= connection.expires_at
                        || !heartbeat.send_ping(&mut output).await { break; }
                }
                received = rec.recv() => {
                    let Ok(message) = received else { break; };
                    if message.request_id != request_id && !send_json(&mut output, &message.message).await { break; }
                }
                incoming = input.next() => {
                    match incoming {
                        Some(Ok(Message::Text(text))) => {
                            if !heartbeat.allow_frame() { break; }
                            let Ok(message) = serde_json::from_str::<ControlMessage>(&text) else { break; };
                            if !valid_message(&message) { break; }
                            if sender.send(ConnectionControlMessage { request_id, message }).is_err() { break; }
                        }
                        other => { if !heartbeat.receive(other, &mut output).await { break; } }
                    }
                }
            }
        }
    })
}

/// Token bucket: sustained 30 frames/second, initial burst of 60 (including control frames).
pub struct MessageBudget {
    tokens: f64,
    last: std::time::Instant,
}
impl MessageBudget {
    pub fn new() -> Self {
        Self {
            tokens: 60.0,
            last: std::time::Instant::now(),
        }
    }
    pub fn allow(&mut self) -> bool {
        let now = std::time::Instant::now();
        self.tokens = (self.tokens + now.duration_since(self.last).as_secs_f64() * 30.0).min(60.0);
        self.last = now;
        if self.tokens < 1.0 {
            return false;
        }
        self.tokens -= 1.0;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn message_budget_bounds_burst() {
        let mut budget = MessageBudget::new();
        for _ in 0..60 {
            assert!(budget.allow());
        }
        assert!(!budget.allow());
    }
}
