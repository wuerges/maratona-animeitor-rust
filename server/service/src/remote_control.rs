use std::sync::atomic::{AtomicU64, Ordering};

use data::remote_control::ControlMessage;
use tokio::sync::broadcast::Sender;

/// Broadcast channel carrying remote-control messages for one key.
pub type ControlSender = Sender<ConnectionControlMessage>;

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionControlMessage {
    pub request_id: u64,
    pub message: ControlMessage,
}

pub fn create_remote_control() -> ControlSender {
    let (sender, _) = tokio::sync::broadcast::channel(100);
    sender
}

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(0);

/// Mints a fresh id for a remote-control connection, used to suppress
/// echoing a client's own messages back to it.
pub fn next_request_id() -> u64 {
    NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}
