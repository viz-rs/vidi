use std::{
    fmt,
    io::{Error as IoError, ErrorKind},
    net::SocketAddr,
};

use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::{native_tls::TlsAcceptor as TlsAcceptorWrapper, TlsStream};

use super::Listener;
use crate::{Error, Result};

pub use tokio_native_tls::{native_tls::Identity, TlsAcceptor};

/// [`native-tls`]'s config.
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

impl crate::Accept for Listener<TcpListener, TlsAcceptor> {
    type Stream = TlsStream<TcpStream>;
    type Addr = SocketAddr;

    async fn accept(&self) -> std::io::Result<(Self::Stream, Self::Addr)> {
        let (stream, addr) = self.inner.accept().await?;
        let tls_stream = self
            .acceptor
            .accept(stream)
            .await
            .map_err(|e| IoError::new(ErrorKind::Other, e))?;
        Ok((tls_stream, addr))
    }
}
