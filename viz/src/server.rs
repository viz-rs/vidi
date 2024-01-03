use std::{fmt::Debug, future::Pending};

mod listener;
pub use listener::Listener;

#[cfg(not(feature = "smol"))]
mod tokio;
#[cfg(not(feature = "smol"))]
pub use self::tokio::serve;

#[cfg(feature = "smol")]
mod smol;
#[cfg(feature = "smol")]
pub use self::smol::serve;

#[cfg(any(feature = "native_tls", feature = "rustls"))]
#[path = "server/tls.rs"]
pub(super) mod internal;

/// TLS
#[cfg(any(feature = "native_tls", feature = "rustls"))]
pub mod tls {
    pub use super::internal::*;

    #[cfg(not(feature = "smol"))]
    pub use super::tokio::tls::*;

    #[cfg(feature = "smol")]
    pub use super::smol::tls::*;
}

/// A listening HTTP server that accepts connections.
#[derive(Debug)]
pub struct Server<L, E, F, S = Pending<()>> {
    signal: S,
    tree: crate::Tree,
    executor: E,
    listener: L,
    build: F,
}

impl<L, E, F, S> Server<L, E, F, S> {
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
