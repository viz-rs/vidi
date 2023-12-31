//! The `Accept` trait and supporting types.

use std::{future::Future, io::Result};

/// Asynchronously accept incoming connections.
pub trait Accept {
    /// An accepted stream of the connection.
    type Stream;
    /// An accepted remote address of the connection.
    type Addr;

    /// Accepts a new incoming connection from this listener.
    fn accept(&self) -> impl Future<Output = Result<(Self::Stream, Self::Addr)>> + Send;
}
