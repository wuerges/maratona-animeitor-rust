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

/// Only the hash and wall-clock expiry are durable; live relay state is transient.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RemoteSessionRecord {
    pub expires_at: u64,
}

pub fn remote_key_hash(key: &str) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(key.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock predates epoch")
        .as_secs()
}

/// A bounded room backed by durable credential metadata. Drop revokes live relays.
pub struct RemoteSession {
    pub record: RemoteSessionRecord,
    sender: ControlSender,
    expires: tokio::time::Instant,
    revoked: tokio::sync::watch::Sender<bool>,
    connections: std::sync::Arc<tokio::sync::Semaphore>,
}

pub struct RemoteConnection {
    pub sender: ControlSender,
    pub expires: tokio::time::Instant,
    pub expires_at: u64,
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
        tokio::time::Instant::now() >= self.expires || unix_seconds() >= self.record.expires_at
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
            expires_at: self.record.expires_at,
            revoked: self.revoked.subscribe(),
            _permit: permit,
        }))
    }
}

pub fn restore_remote_session(record: RemoteSessionRecord) -> Option<RemoteSession> {
    let remaining = record.expires_at.checked_sub(unix_seconds())?;
    if remaining == 0 {
        return None;
    }
    let (revoked, _) = tokio::sync::watch::channel(false);
    let expires =
        tokio::time::Instant::now().checked_add(std::time::Duration::from_secs(remaining))?;
    Some(RemoteSession {
        record,
        sender: create_remote_control(),
        expires,
        revoked,
        connections: std::sync::Arc::new(tokio::sync::Semaphore::new(8)),
    })
}

pub fn create_remote_session(
    key: String,
    lifetime_seconds: u64,
) -> (RemoteSession, data::remote_control::RemoteSessionInfo) {
    let expires_at = unix_seconds() + lifetime_seconds;
    let session = restore_remote_session(RemoteSessionRecord { expires_at })
        .expect("validated positive lifetime");
    (
        session,
        data::remote_control::RemoteSessionInfo { key, expires_at },
    )
}

#[cfg(test)]
mod session_tests {
    use super::*;
    #[tokio::test]
    async fn session_bounds_connections_releases_slots_and_revokes() {
        let (session, _) = create_remote_session("test".into(), 3600);
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
        let (mut session, _) = create_remote_session("test".into(), 3600);
        session.expires = tokio::time::Instant::now();
        assert!(session.connect().unwrap().is_none());
    }
}
