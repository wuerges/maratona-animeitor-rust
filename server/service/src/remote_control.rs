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

/// A bounded ephemeral room. Dropping it revokes every connected relay.
pub struct RemoteSession {
    sender: ControlSender,
    expires: tokio::time::Instant,
    revoked: tokio::sync::watch::Sender<bool>,
    connections: std::sync::Arc<tokio::sync::Semaphore>,
}

pub struct RemoteConnection {
    pub sender: ControlSender,
    pub expires: tokio::time::Instant,
    pub revoked: tokio::sync::watch::Receiver<bool>,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl Drop for RemoteSession {
    fn drop(&mut self) {
        let _ = self.revoked.send(true);
    }
}
impl RemoteSession {
    pub fn is_expired(&self) -> bool {
        tokio::time::Instant::now() >= self.expires
    }
    pub fn connect(&self) -> Result<Option<RemoteConnection>, crate::event_store::StoreError> {
        if self.is_expired() {
            return Ok(None);
        }
        let permit = self
            .connections
            .clone()
            .try_acquire_owned()
            .map_err(|_| crate::event_store::StoreError::LimitExceeded)?;
        Ok(Some(RemoteConnection {
            sender: self.sender.clone(),
            expires: self.expires,
            revoked: self.revoked.subscribe(),
            _permit: permit,
        }))
    }
}

pub fn create_remote_session(
    key: String,
) -> (RemoteSession, data::remote_control::RemoteSessionInfo) {
    let lifetime = std::time::Duration::from_secs(3600);
    let (revoked, _) = tokio::sync::watch::channel(false);
    (
        RemoteSession {
            sender: create_remote_control(),
            expires: tokio::time::Instant::now() + lifetime,
            revoked,
            connections: std::sync::Arc::new(tokio::sync::Semaphore::new(8)),
        },
        data::remote_control::RemoteSessionInfo {
            key,
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock predates epoch")
                .as_secs()
                + lifetime.as_secs(),
        },
    )
}

#[cfg(test)]
mod session_tests {
    use super::*;
    #[tokio::test]
    async fn session_bounds_connections_releases_slots_and_revokes() {
        let (session, _) = create_remote_session("test".into());
        let mut connections: Vec<_> = (0..8)
            .map(|_| session.connect().unwrap().unwrap())
            .collect();
        assert!(matches!(
            session.connect(),
            Err(crate::event_store::StoreError::LimitExceeded)
        ));
        connections.pop();
        let mut last = session.connect().unwrap().unwrap();
        drop(session);
        last.revoked.changed().await.unwrap();
        assert!(*last.revoked.borrow());
    }
    #[tokio::test]
    async fn expired_session_rejects_connections() {
        let (mut session, _) = create_remote_session("test".into());
        session.expires = tokio::time::Instant::now();
        assert!(session.connect().unwrap().is_none());
    }
}
