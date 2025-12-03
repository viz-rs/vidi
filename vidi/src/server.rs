use std::{
    fmt::Debug,
    future::{Future, IntoFuture, Pending, pending},
    io,
    pin::{Pin, pin},
    sync::Arc,
    time::Duration,
};

#[cfg(any(feature = "http1", feature = "http2"))]
use hyper_util::server::conn::auto::Builder;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::graceful::GracefulShutdown,
};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{Listener, Responder, Router};

/// TLS
#[cfg(any(feature = "native-tls", feature = "rustls"))]
pub mod tls;

#[cfg(any(feature = "http1", feature = "http2"))]
mod tcp;

#[cfg(all(unix, feature = "unix-socket"))]
mod unix;

/// Starts a server and serves the connections.
pub fn serve<L>(listener: L, router: Router) -> Server<L> {
    Server::<L>::new(listener, router)
}

/// A listening HTTP server that accepts connections.
#[derive(Debug)]
pub struct Server<L, S = Pending<()>> {
    listener: L,
    signal: S,
    tree: crate::Tree,
    builder: Builder<TokioExecutor>,
}

impl<L> Server<L> {
    /// Starts a [`Server`] with a listener and a [`Router`].
    pub fn new(listener: L, router: Router) -> Self {
        Self::with_builder(listener, router, Builder::new(TokioExecutor::new()))
    }

    /// Starts a [`Server`] with a listener, a [`Router`] and a [`Builder`].
    pub fn with_builder(listener: L, router: Router, builder: Builder<TokioExecutor>) -> Self {
        Server {
            listener,
            builder,
            signal: pending(),
            tree: router.into(),
        }
    }

    /// Specifies a signal for graceful shutdown.
    pub fn signal<S>(self, signal: S) -> Server<L, S> {
        Server {
            signal,
            tree: self.tree,
            builder: self.builder,
            listener: self.listener,
        }
    }
}

impl<L, S> IntoFuture for Server<L, S>
where
    L: Listener + Send + 'static,
    L::Io: AsyncRead + AsyncWrite + Send + Unpin,
    L::Addr: Send + Sync + Debug,
    S: Future + Send + 'static,
    S::Output: Send,
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

        Box::pin(async move {
            let graceful = GracefulShutdown::new();
            let tree = Arc::new(tree);
            let mut signal = pin!(signal);

            loop {
                tokio::select! {
                    conn = listener.accept() => {
                        let (stream, peer_addr) = match conn {
                            Ok(conn) => conn,
                            Err(err) => {
                                if !is_connection_error(&err) {
                                    tracing::error!("listener accept error: {err}");
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                }
                                continue;
                            }
                        };

                        tracing::trace!("incomming connection accepted: {:?}", peer_addr);

                        let peer_addr = Arc::new(peer_addr);

                        let stream = TokioIo::new(Box::pin(stream));

                        let responder = Responder::new(tree.clone(), Some(peer_addr.clone()));

                        let conn = builder.serve_connection_with_upgrades(stream, responder);

                        let conn = graceful.watch(conn.into_owned());

                        tokio::spawn(async move {
                            if let Err(err) = conn.await {
                                tracing::error!("connection error: {}", err);
                            }
                            tracing::trace!("connection dropped: {:?}", peer_addr);
                        });
                    },

                    _ = signal.as_mut() => {
                        drop(listener);
                        tracing::trace!("Signal received, starting shutdown");
                        break;
                    }
                }
            }

            tokio::select! {
                () = graceful.shutdown() => {
                    tracing::trace!("Gracefully shutdown!");
                },
                () = tokio::time::sleep(Duration::from_secs(10)) => {
                    tracing::error!("Waited 10 seconds for graceful shutdown, aborting...");
                }
            }

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
