use std::{
    borrow::Borrow,
    fmt::Debug,
    future::{pending, Future, IntoFuture, Pending},
    io,
    pin::Pin,
    sync::Arc,
};

use async_executor::Executor;
use futures_lite::io::{AsyncRead, AsyncWrite};
use hyper::rt::Timer;
use hyper_util::server::conn::auto::Builder;
use smol_hyper::rt::{FuturesIo, SmolExecutor, SmolTimer};

use crate::{future::FutureExt, Responder, Router, Tree};
use crate::{Listener, Server};

#[cfg(any(feature = "http1", feature = "http2"))]
mod tcp;

#[cfg(all(unix, feature = "unix-socket"))]
mod unix;

/// TLS
#[cfg(any(feature = "native_tls", feature = "rustls"))]
pub mod tls;

// impl<L, E, F, S> Server<L, E, F, S> {
//     /// Starts a [`Server`] with a listener and a [`Router`].
//     pub fn new<'ex>(executor: E, listener: L, router: Router) -> Server<L, E, F, Pending<()>>
//     where
//         E: Borrow<Executor<'ex>> + Clone + Send + 'ex,
//     {
//         Server {
//             executor,
//             listener,
//             signal: pending(),
//             tree: router.into(),
//             build:
//         }
//     }
// }

/// Serve a future using [`smol`]'s TCP listener.
pub async fn serve<'ex, E, L>(executor: E, listener: L, router: Router) -> io::Result<()>
where
    E: Borrow<Executor<'ex>> + Clone + Send + 'ex,
    L: Listener + Send + 'static,
    L::Io: AsyncRead + AsyncWrite + Send + Unpin,
    L::Addr: Send + Sync + Debug,
{
    let tree = Arc::<Tree>::new(router.into());

    loop {
        // Wait for a new connection.
        let (stream, remote_addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                if !is_connection_error(&e) {
                    // [From `hyper::Server` in 0.14](https://github.com/hyperium/hyper/blob/v0.14.27/src/server/tcp.rs#L186)
                    tracing::error!("listener accept error: {e}");
                    SmolTimer::new()
                        .sleep(std::time::Duration::from_secs(1))
                        .await;
                }
                continue;
            }
        };

        // Wrap it in a `FuturesIo`.
        let io = FuturesIo::new(stream);
        let remote_addr = Arc::new(remote_addr);
        let responder = Responder::<Arc<L::Addr>>::new(tree.clone(), Some(remote_addr.clone()));

        // Spawn the service on our executor.
        let task = executor.borrow().spawn({
            let executor = executor.clone();
            async move {
                let mut builder = Builder::new(SmolExecutor::new(AsRefExecutor(executor.borrow())));
                builder.http1().timer(SmolTimer::new());
                builder.http2().timer(SmolTimer::new());

                if let Err(err) = builder.serve_connection_with_upgrades(io, responder).await {
                    tracing::error!("unintelligible hyper error: {err}");
                }
            }
        });

        // Detach the task and let it run forever.
        task.detach();
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

#[derive(Clone)]
struct AsRefExecutor<'this, 'ex>(&'this Executor<'ex>);

impl<'ex> AsRef<Executor<'ex>> for AsRefExecutor<'_, 'ex> {
    #[inline]
    fn as_ref(&self) -> &Executor<'ex> {
        self.0
    }
}
