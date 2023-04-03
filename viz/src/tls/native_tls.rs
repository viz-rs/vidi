use std::{fmt, net::SocketAddr};

use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::{native_tls::TlsAcceptor as TlsAcceptorWrapper, TlsStream};

use super::Listener;
use crate::{Error, Result};

pub use tokio_native_tls::{native_tls::Identity, TlsAcceptor};

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
    /// Creates a new config with the specified [Identity].
    pub fn new(identity: Identity) -> Self {
        Self { identity }
    }

    /// Creates a new [TlsAcceptor] wrapper with the specified [Identity].
    pub fn build(self) -> Result<TlsAcceptor> {
        TlsAcceptorWrapper::new(self.identity)
            .map(Into::into)
            .map_err(Error::normal)
    }
}

impl Listener<TcpListener, TlsAcceptor> {
    /// A [`TlsStream`] and [`SocketAddr] part for accepting TLS.
    pub async fn accept(&self) -> Result<(TlsStream<TcpStream>, SocketAddr)> {
        let (stream, addr) = self.inner.accept().await?;
        let tls_stream = self.acceptor.accept(stream).await.map_err(Error::normal)?;
        Ok((tls_stream, addr))
    }
}
