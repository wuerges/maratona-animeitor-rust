//! Validated process budgets shared by configuration, storage, and HTTP serving.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ResourceLimits {
    pub max_websockets: usize,
    pub max_connections: usize,
    pub max_public_requests: usize,
    pub max_internal_requests: usize,
    pub max_mutations: usize,
    pub max_response_bodies: usize,
    pub request_timeout_seconds: u64,
    pub tls_handshake_timeout_seconds: u64,
    pub header_timeout_seconds: u64,
    pub write_stall_timeout_seconds: u64,
    pub http_idle_timeout_seconds: u64,
}
impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_websockets: 50_000,
            max_connections: 60_000,
            max_public_requests: 128,
            max_internal_requests: 32,
            max_mutations: 32,
            max_response_bodies: 1024,
            request_timeout_seconds: 15,
            tls_handshake_timeout_seconds: 10,
            header_timeout_seconds: 10,
            write_stall_timeout_seconds: 10,
            http_idle_timeout_seconds: 60,
        }
    }
}
impl ResourceLimits {
    pub fn validate(&self) -> Result<(), &'static str> {
        for count in [
            self.max_websockets,
            self.max_connections,
            self.max_public_requests,
            self.max_internal_requests,
            self.max_mutations,
            self.max_response_bodies,
        ] {
            if count == 0 || count > tokio::sync::Semaphore::MAX_PERMITS {
                return Err("limits must be positive and within semaphore capacity");
            }
        }
        for seconds in [
            self.request_timeout_seconds,
            self.tls_handshake_timeout_seconds,
            self.header_timeout_seconds,
            self.write_stall_timeout_seconds,
            self.http_idle_timeout_seconds,
        ] {
            if seconds == 0 || seconds > 86400 {
                return Err("timeouts must be between 1 and 86400 seconds");
            }
        }
        if self.max_connections <= self.max_websockets {
            return Err("max_connections must exceed max_websockets for HTTP headroom");
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_capacity_and_timeouts() {
        let mut limits = ResourceLimits::default();
        assert!(limits.max_websockets >= 40_000);
        assert!(limits.validate().is_ok());
        limits.max_websockets = 0;
        assert!(limits.validate().is_err());
        limits.max_websockets = limits.max_connections;
        assert!(limits.validate().is_err());
        limits.max_websockets = 1;
        limits.request_timeout_seconds = u64::MAX;
        assert!(limits.validate().is_err());
    }
}
