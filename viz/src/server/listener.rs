use std::{future::Future, io::Result};

/// A trait for a listener: `TcpListener` and `UnixListener`.
pub trait Listener {
    /// The stream's type of this listener.
    type Io;
    /// The socket address type of this listener.
    type Addr;

    /// Accepts a new incoming connection from this listener.
    fn accept(&self) -> impl Future<Output = Result<(Self::Io, Self::Addr)>> + Send;

    /// Returns the local address that this listener is bound to.
    fn local_addr(&self) -> Result<Self::Addr>;
}
