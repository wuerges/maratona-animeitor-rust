//! Connection permits follow the IO through TLS and HTTP upgrades.
use axum_server::accept::Accept;
use service::resource_limits::ResourceLimits;
use std::future::Future;
use std::{
    future::{Ready, ready},
    io,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::TcpStream,
    sync::{OwnedSemaphorePermit, Semaphore},
    time::{Instant, Sleep},
};

#[derive(Clone)]
pub(crate) struct Acceptor {
    slots: Arc<Semaphore>,
    limits: ResourceLimits,
}
impl Acceptor {
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            slots: Arc::new(Semaphore::new(limits.max_connections)),
            limits,
        }
    }
}
impl<S> Accept<TcpStream, S> for Acceptor {
    type Stream = BudgetIo<TcpStream>;
    type Service = UpgradeAware<S>;
    type Future = Ready<io::Result<(Self::Stream, Self::Service)>>;
    fn accept(&self, stream: TcpStream, service: S) -> Self::Future {
        let result = self
            .slots
            .clone()
            .try_acquire_owned()
            .map_err(|_| {
                service::resource_metrics::event("connection_rejected");
                io::Error::other("connection capacity exceeded")
            })
            .and_then(|permit| {
                stream.set_nodelay(true)?;
                let io = BudgetIo::new(stream, permit, &self.limits);
                let service = UpgradeAware {
                    inner: service,
                    upgraded: io.upgraded.clone(),
                };
                Ok((io, service))
            });
        ready(result)
    }
}

#[derive(Clone)]
pub(crate) struct UpgradeAware<S> {
    inner: S,
    upgraded: Arc<AtomicBool>,
}
impl<S, B> tower::Service<axum::http::Request<B>> for UpgradeAware<S>
where
    S: tower::Service<axum::http::Request<B>, Response = axum::response::Response>,
    S::Future: Send + 'static,
    S::Error: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    fn call(&mut self, request: axum::http::Request<B>) -> Self::Future {
        let connect = request.method() == axum::http::Method::CONNECT;
        let future = self.inner.call(request);
        let upgraded = self.upgraded.clone();
        Box::pin(async move {
            let response = future.await?;
            if response.status() == axum::http::StatusCode::SWITCHING_PROTOCOLS
                || (connect && response.status().is_success())
            {
                upgraded.store(true, Ordering::Release);
            }
            Ok(response)
        })
    }
}

pub(crate) struct BudgetIo<I> {
    inner: I,
    _active: service::resource_metrics::Active,
    upgraded: Arc<AtomicBool>,
    _permit: OwnedSemaphorePermit,
    idle: Pin<Box<Sleep>>,
    stalled: Option<Pin<Box<Sleep>>>,
    flush_stalled: Option<Pin<Box<Sleep>>>,
    idle_duration: Duration,
    write_duration: Duration,
}
impl<I> BudgetIo<I> {
    fn new(inner: I, permit: OwnedSemaphorePermit, limits: &ResourceLimits) -> Self {
        let idle_duration = Duration::from_secs(limits.http_idle_timeout_seconds);
        Self {
            inner,
            _active: service::resource_metrics::Active::new("transport"),
            upgraded: Arc::new(AtomicBool::new(false)),
            _permit: permit,
            idle: Box::pin(tokio::time::sleep(idle_duration)),
            stalled: None,
            flush_stalled: None,
            idle_duration,
            write_duration: Duration::from_secs(limits.write_stall_timeout_seconds),
        }
    }
    fn progress(&mut self) {
        self.idle
            .as_mut()
            .reset(Instant::now() + self.idle_duration);
    }
    fn stalled(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let timer = self
            .stalled
            .get_or_insert_with(|| Box::pin(tokio::time::sleep(self.write_duration)));
        if timer.as_mut().poll(cx).is_ready() {
            service::resource_metrics::event("write_stalled");
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "write stalled",
            )))
        } else {
            Poll::Pending
        }
    }
}
impl<I: AsyncRead + Unpin> AsyncRead for BudgetIo<I> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let before = buf.filled().len();
        match Pin::new(&mut this.inner).poll_read(cx, buf) {
            Poll::Ready(result) => {
                if result.is_ok() && buf.filled().len() > before {
                    this.progress();
                }
                Poll::Ready(result)
            }
            Poll::Pending => {
                if this.upgraded.load(Ordering::Acquire) {
                    return Poll::Pending;
                }
                if this.idle.as_mut().poll(cx).is_ready() {
                    Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "connection idle",
                    )))
                } else {
                    Poll::Pending
                }
            }
        }
    }
}
impl<I: AsyncWrite + Unpin> AsyncWrite for BudgetIo<I> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        data: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        match Pin::new(&mut this.inner).poll_write(cx, data) {
            Poll::Ready(result) => {
                if result.as_ref().is_ok_and(|n| *n > 0) {
                    this.progress();
                    this.stalled = None;
                }
                Poll::Ready(result)
            }
            Poll::Pending => this.stalled(cx).map_ok(|_| 0),
        }
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        match Pin::new(&mut this.inner).poll_flush(cx) {
            Poll::Ready(result) => {
                this.flush_stalled = None;
                Poll::Ready(result)
            }
            Poll::Pending => {
                let timer = this
                    .flush_stalled
                    .get_or_insert_with(|| Box::pin(tokio::time::sleep(this.write_duration)));
                if timer.as_mut().poll(cx).is_ready() {
                    Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "flush stalled",
                    )))
                } else {
                    Poll::Pending
                }
            }
        }
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}
pub(crate) fn configure(
    builder: &mut hyper_util::server::conn::auto::Builder<hyper_util::rt::TokioExecutor>,
    limits: &ResourceLimits,
) {
    builder
        .http1()
        .timer(hyper_util::rt::TokioTimer::new())
        .header_read_timeout(Duration::from_secs(limits.header_timeout_seconds))
        .max_headers(100)
        .max_buf_size(32 * 1024);
    builder
        .http2()
        .timer(hyper_util::rt::TokioTimer::new())
        .max_concurrent_streams(128)
        .max_header_list_size(32 * 1024)
        .keep_alive_interval(Some(Duration::from_secs(30)))
        .keep_alive_timeout(Duration::from_secs(10));
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    fn limits() -> ResourceLimits {
        ResourceLimits {
            max_connections: 2,
            max_websockets: 1,
            http_idle_timeout_seconds: 2,
            write_stall_timeout_seconds: 1,
            header_timeout_seconds: 1,
            ..Default::default()
        }
    }
    #[tokio::test(start_paused = true)]
    async fn idle_and_write_stalls_expire_and_release_permits() {
        let slots = Arc::new(Semaphore::new(1));
        let (io, _peer) = tokio::io::duplex(1);
        let mut io = BudgetIo::new(io, slots.clone().acquire_owned().await.unwrap(), &limits());
        assert_eq!(
            io.write_all(b"blocked").await.unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        assert_eq!(slots.available_permits(), 0);
        drop(io);
        assert_eq!(slots.available_permits(), 1);
        let (io, _peer) = tokio::io::duplex(1);
        let mut io = BudgetIo::new(io, slots.clone().acquire_owned().await.unwrap(), &limits());
        assert_eq!(
            io.read_u8().await.unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        drop(io);
        assert_eq!(slots.available_permits(), 1);
    }
    #[tokio::test(start_paused = true)]
    async fn useful_output_resets_idle_but_input_does_not_reset_write_stall() {
        let slots = Arc::new(Semaphore::new(1));
        let (io, mut peer) = tokio::io::duplex(1);
        let mut io = BudgetIo::new(io, slots.acquire_owned().await.unwrap(), &limits());
        for _ in 0..5 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            io.write_all(b"x").await.unwrap();
            assert_eq!(peer.read_u8().await.unwrap(), b'x');
        }
        assert!(!io.idle.is_elapsed());
        io.write_all(b"x").await.unwrap();
        // Incoming traffic must not extend a stalled outgoing operation.
        let stalled = futures::future::poll_fn(|cx| Pin::new(&mut io).poll_write(cx, b"y"));
        assert!(futures::poll!(Box::pin(stalled)).is_pending());
        peer.write_all(b"z").await.unwrap();
        assert_eq!(io.read_u8().await.unwrap(), b'z');
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert_eq!(
            io.write_all(b"y").await.unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
    }
    async fn start(
        acceptor: Acceptor,
    ) -> (
        std::net::SocketAddr,
        axum_server::Handle<std::net::SocketAddr>,
    ) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = axum_server::Handle::new();
        let mut server = axum_server::from_tcp(listener)
            .unwrap()
            .acceptor(acceptor.clone())
            .handle(handle.clone());
        configure(server.http_builder(), &acceptor.limits);
        let app = axum::Router::new()
            .route("/", axum::routing::get(|| async { "ok" }))
            .route(
                "/ws",
                axum::routing::get(|ws: axum::extract::WebSocketUpgrade| async {
                    ws.on_upgrade(
                        |mut socket| async move { while socket.recv().await.is_some() {} },
                    )
                }),
            );
        tokio::spawn(async move {
            server.serve(app.into_make_service()).await.unwrap();
        });
        handle.listening().await.unwrap();
        (addr, handle)
    }
    #[tokio::test]
    async fn shared_acceptors_reject_capacity_and_upgrades_hold_transport_slot() {
        let acceptor = Acceptor::new(limits());
        let (a, handle_a) = start(acceptor.clone()).await;
        let (b, handle_b) = start(acceptor.clone()).await;
        let (socket, _) = tokio_tungstenite::connect_async(format!("ws://{a}/ws"))
            .await
            .unwrap();
        assert_eq!(acceptor.slots.available_permits(), 1);
        let mut incomplete = TcpStream::connect(b).await.unwrap();
        incomplete
            .write_all(b"GET / HTTP/1.1\r\nHost:")
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            while acceptor.slots.available_permits() != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let mut rejected = TcpStream::connect(a).await.unwrap();
        let mut buf = [0; 256];
        let result = tokio::time::timeout(Duration::from_secs(2), rejected.read(&mut buf))
            .await
            .unwrap();
        assert!(result.is_err() || result.unwrap() == 0);
        // The incomplete header expires; it never becomes a WebSocket slot.
        let _ = tokio::time::timeout(Duration::from_secs(3), incomplete.read(&mut buf))
            .await
            .unwrap();
        drop(socket);
        drop(incomplete);
        tokio::time::timeout(Duration::from_secs(3), async {
            while acceptor.slots.available_permits() != 2 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        handle_a.shutdown();
        handle_b.shutdown();
    }
    #[tokio::test]
    async fn tls_admission_precedes_handshake_and_http2_remains_available() {
        let directory = tempfile::tempdir().unwrap();
        let cert = directory.path().join("cert.pem");
        let key = directory.path().join("key.pem");
        let status = std::process::Command::new("openssl")
            .args([
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-nodes",
                "-days",
                "1",
                "-subj",
                "/CN=localhost",
                "-addext",
                "subjectAltName=DNS:localhost",
            ])
            .arg("-keyout")
            .arg(&key)
            .arg("-out")
            .arg(&cert)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success());
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let acceptor = Acceptor::new(limits());
        let (plain, http_handle) = start(acceptor.clone()).await;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let mut tls = service::http::load_rustls_config(&cert, &key).unwrap();
        tls.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        let tls_acceptor = axum_server::tls_rustls::RustlsAcceptor::new(
            axum_server::tls_rustls::RustlsConfig::from_config(Arc::new(tls)),
        )
        .handshake_timeout(Duration::from_secs(1))
        .acceptor(acceptor.clone());
        let tls_handle = axum_server::Handle::new();
        let mut server = axum_server::from_tcp(listener)
            .unwrap()
            .acceptor(tls_acceptor)
            .handle(tls_handle.clone());
        configure(server.http_builder(), &limits());
        let app = axum::Router::new().route("/", axum::routing::get(|| async { "tls ok" }));
        tokio::spawn(async move {
            server.serve(app.into_make_service()).await.unwrap();
        });
        tls_handle.listening().await.unwrap();
        let (socket, _) = tokio_tungstenite::connect_async(format!("ws://{plain}/ws"))
            .await
            .unwrap();
        let mut stalled = TcpStream::connect(addr).await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            while acceptor.slots.available_permits() != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let mut byte = [0];
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(3), stalled.read(&mut byte))
                .await
                .unwrap()
                .unwrap(),
            0
        );
        let client = reqwest::Client::builder()
            .no_proxy()
            .add_root_certificate(
                reqwest::Certificate::from_pem(&std::fs::read(&cert).unwrap()).unwrap(),
            )
            .http2_prior_knowledge()
            .build()
            .unwrap();
        let response = client
            .get(format!("https://localhost:{}/", addr.port()))
            .send()
            .await
            .unwrap();
        assert_eq!(response.version(), reqwest::Version::HTTP_2);
        assert_eq!(response.text().await.unwrap(), "tls ok");
        drop(socket);
        drop(client);
        http_handle.shutdown();
        tls_handle.shutdown();
    }
    #[tokio::test(start_paused = true)]
    async fn upgraded_socket_uses_heartbeat_instead_of_http_idle_policy() {
        let slots = Arc::new(Semaphore::new(1));
        let (raw, _peer) = tokio::io::duplex(1);
        let mut io = BudgetIo::new(raw, slots.clone().acquire_owned().await.unwrap(), &limits());
        io.upgraded.store(true, Ordering::Release);
        tokio::time::sleep(Duration::from_secs(100)).await;
        assert!(futures::poll!(Box::pin(io.read_u8())).is_pending());
        assert_eq!(slots.available_permits(), 0);
        drop(io);
        assert_eq!(slots.available_permits(), 1);
    }
}
