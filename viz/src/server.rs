use std::{
    fmt::Debug,
    future::{pending, Future, IntoFuture, Pending},
    io,
    pin::Pin,
    sync::Arc,
};

use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    pin, select,
    sync::watch,
};

use crate::{future::FutureExt, Responder, Router, Tree};

mod accept;
pub use accept::Accept;

#[cfg(any(feature = "http1", feature = "http2"))]
mod tcp;

#[cfg(feature = "unix-socket")]
mod unix;

/// Starts a server and serves the connections.
pub fn serve<L>(listener: L, router: Router) -> Server<L>
where
    L: Accept + Send + 'static,
    L::Stream: AsyncWrite + AsyncRead + Send + Unpin,
    L::Addr: Send + Sync + Debug + 'static,
{
    Server::<L>::new(listener, router)
}

/// A listening HTTP server that accepts connections.
#[derive(Debug)]
pub struct Server<L, E = TokioExecutor, F = Pending<()>> {
    signal: F,
    tree: Tree,
    listener: L,
    builder: Builder<E>,
}

impl<L, E, F> Server<L, E, F> {
    /// Starts a [`Server`] with a listener and a [`Tree`].
    pub fn new(listener: L, router: Router) -> Server<L> {
        Server {
            listener,
            signal: pending(),
            tree: router.into(),
            builder: Builder::new(TokioExecutor::new()),
        }
    }

    /// Changes the signal for graceful shutdown.
    pub fn signal<T>(self, signal: T) -> Server<L, E, T> {
        Server {
            signal,
            tree: self.tree,
            builder: self.builder,
            listener: self.listener,
        }
    }

    /// Returns the HTTP1 or HTTP2 connection builder.
    pub fn builder(&mut self) -> &mut Builder<E> {
        &mut self.builder
    }
}

/// Copied from Axum. Thanks.
impl<L, F> IntoFuture for Server<L, TokioExecutor, F>
where
    L: Accept + Send + 'static,
    L::Stream: AsyncWrite + AsyncRead + Send + Unpin,
    L::Addr: Send + Sync + Debug + 'static,
    F: Future + Send + 'static,
{
    type Output = io::Result<()>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            tree,
            signal,
            builder,
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
                let builder = builder.clone();
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
