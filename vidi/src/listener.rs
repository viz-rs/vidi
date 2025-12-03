/// A trait for a listener: `TcpListener` and `UnixListener`.
pub trait Listener {
    /// The stream's type of this listener.
    type Io;
    /// The socket address type of this listener.
    type Addr;

    /// Accepts a new incoming connection from this listener.
    fn accept(
        &self,
    ) -> impl std::future::Future<Output = std::io::Result<(Self::Io, Self::Addr)>> + Send;

    /// Returns the local address that this listener is bound to.
    ///
    /// # Errors
    ///
    /// An error will return if got the socket address of the local half of this connection is
    /// failed.
    fn local_addr(&self) -> std::io::Result<Self::Addr>;
}
