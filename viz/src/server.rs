use std::{
    fmt::Debug,
    future::{pending, Future, IntoFuture, Pending},
    io,
    pin::Pin,
    sync::Arc,
};

use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    pin, select,
    sync::watch,
};

use crate::{future::FutureExt, Listener, Responder, Router};

/// TLS
#[cfg(any(feature = "native-tls", feature = "rustls"))]
pub mod tls;

#[cfg(any(feature = "http1", feature = "http2"))]
mod tcp;

#[cfg(all(unix, feature = "unix-socket"))]
mod unix;

/// Starts a server and serves the connections.
pub fn serve<L>(
    listener: L,
    router: Router,
) -> Server<L, TokioExecutor, fn(TokioExecutor) -> Builder<TokioExecutor>, Pending<()>> {
    Server::<L, TokioExecutor, fn(TokioExecutor) -> Builder<TokioExecutor>, Pending<()>>::new(
        TokioExecutor::new(),
        listener,
        router,
        |executor: TokioExecutor| Builder::new(executor),
    )
}

/// A listening HTTP server that accepts connections.
#[derive(Debug)]
pub struct Server<L, E, F, S> {
    listener: L,
    executor: E,
    build: F,
    signal: S,
    tree: crate::Tree,
}

impl<L, E, F, S> Server<L, E, F, S> {
    /// Starts a [`Server`] with a listener and a [`Router`].
    pub fn new(executor: E, listener: L, router: Router, build: F) -> Server<L, E, F, Pending<()>>
    where
        F: Fn(E) -> Builder<E> + Send + 'static,
    {
        Server {
            build,
            executor,
            listener,
            signal: pending(),
            tree: router.into(),
        }
    }

    /// Changes the signal for graceful shutdown.
    pub fn signal<X>(self, signal: X) -> Server<L, E, F, X> {
        Server {
            signal,
            tree: self.tree,
            build: self.build,
            executor: self.executor,
            listener: self.listener,
        }
    }
}

/// Copied from Axum. Thanks.
impl<L, F, S> IntoFuture for Server<L, TokioExecutor, F, S>
where
    L: Listener + Send + 'static,
    L::Io: AsyncRead + AsyncWrite + Send + Unpin,
    L::Addr: Send + Sync + Debug,
    F: Fn(TokioExecutor) -> Builder<TokioExecutor> + Send + 'static,
    S: Future + Send + 'static,
{
    type Output = io::Result<()>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            tree,
            build,
            signal,
            executor,
            listener,
        } = self;

        let (shutdown_tx, shutdown_rx) = watch::channel(());
        let shutdown_tx = Arc::new(shutdown_tx);

        tokio::spawn(async move {
            signal.await;
            tracing::trace!("received graceful shutdown signal");
            drop(shutdown_rx);
        });

        let (close_tx, close_rx) = watch::channel(());

        let tree = Arc::new(tree);

        Box::pin(async move {
            loop {
                let (stream, remote_addr) = select! {
                    res = listener.accept() => {
                        match res {
                            Ok(conn) => conn,
                            Err(e) => {
                                if !is_connection_error(&e) {
                                    // [From `hyper::Server` in 0.14](https://github.com/hyperium/hyper/blob/v0.14.27/src/server/tcp.rs#L186)
                                    tracing::error!("listener accept error: {e}");
                                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                                }
                                continue
                            }
                        }
                    }
                    () = shutdown_tx.closed() => {
                        tracing::trace!("server is closing");
                        break;
                    }
                };

                tracing::trace!("connection {:?} accepted", remote_addr);

                let io = TokioIo::new(stream);
                let remote_addr = Arc::new(remote_addr);
                let builder = (build)(executor.clone());
                let responder =
                    Responder::<Arc<L::Addr>>::new(tree.clone(), Some(remote_addr.clone()));

                let shutdown_tx = Arc::clone(&shutdown_tx);
                let close_rx = close_rx.clone();

                tokio::spawn(async move {
                    let conn = builder.serve_connection_with_upgrades(io, responder);
                    pin!(conn);

                    let shutdown = shutdown_tx.closed().fuse();
                    pin!(shutdown);

                    loop {
                        select! {
                            res = conn.as_mut() => {
                                if let Err(e) = res {
                                    tracing::error!("connection failed: {e}");
                                }
                                break;
                            }
                            () = &mut shutdown => {
                                tracing::trace!("connection is starting to graceful shutdown");
                                conn.as_mut().graceful_shutdown();
                            }
                        }
                    }

                    tracing::trace!("connection {:?} closed", remote_addr);

                    drop(close_rx);
                });
            }

            drop(close_rx);
            drop(listener);

            tracing::trace!(
                "waiting for {} task(s) to finish",
                close_tx.receiver_count()
            );
            close_tx.closed().await;

            tracing::trace!("server shutdown complete");

            Ok(())
        })
    }
}

fn is_connection_error(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionReset
    )
}
