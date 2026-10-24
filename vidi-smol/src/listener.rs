/// A trait for a listener: `TcpListener` and `UnixListener`.
pub trait Listener {
    /// The type of the accepted stream.
    type Io;
    /// The type of the local address.
    type Addr;

    /// Accepts a new incoming connection from this listener.
    fn accept(
        &self,
    ) -> impl std::future::Future<Output = std::io::Result<(Self::Io, Self::Addr)>> + Send;

    /// Returns the local address that this listener is bound to.
    ///
    /// # Errors
    ///
    /// Returns an error if the local address cannot be resolved.
    fn local_addr(&self) -> std::io::Result<Self::Addr>;
}
