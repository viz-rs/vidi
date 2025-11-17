use std::{fmt, io::Result as IoResult, net::SocketAddr};

use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::{TlsStream, native_tls::TlsAcceptor as TlsAcceptorWrapper};

use crate::{Error, Result};

pub use tokio_native_tls::{TlsAcceptor, native_tls::Identity};

/// `native-tls`'s config.
pub struct Config {
    identity: Identity,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeTls Config").finish()
    }
}

impl Config {
    /// Creates a new config with the specified [`Identity`].
    #[must_use]
    pub fn new(identity: Identity) -> Self {
        Self { identity }
    }

    /// Creates a new [`TlsAcceptor`] wrapper with the specified [`Identity`].
    ///
    /// # Errors
    ///
    /// Will return `Err` if wrapping the identity fails.
    pub fn build(self) -> Result<TlsAcceptor> {
        TlsAcceptorWrapper::new(self.identity)
            .map(Into::into)
            .map_err(Error::boxed)
    }
}

impl crate::Listener for crate::tls::TlsListener<TcpListener, TlsAcceptor> {
    type Io = TlsStream<TcpStream>;
    type Addr = SocketAddr;

    async fn accept(&self) -> IoResult<(Self::Io, Self::Addr)> {
        let (stream, addr) = self.inner.accept().await?;
        let stream = self
            .acceptor
            .accept(stream)
            .await
            .map_err(std::io::Error::other)?;
        Ok((stream, addr))
    }

    fn local_addr(&self) -> IoResult<Self::Addr> {
        self.inner.local_addr()
    }
}
