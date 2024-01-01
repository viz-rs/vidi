use std::{
    fmt,
    io::{Error as IoError, ErrorKind, Result as IoResult},
    net::SocketAddr,
    task::{Context, Poll},
};

use futures_util::FutureExt;
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

impl tokio_util::net::Listener for Listener<TcpListener, TlsAcceptor> {
    type Io = TlsStream<TcpStream>;
    type Addr = SocketAddr;

    fn poll_accept(&mut self, cx: &mut Context<'_>) -> Poll<IoResult<(Self::Io, Self::Addr)>> {
        let Poll::Ready((stream, addr)) = self.inner.poll_accept(cx)? else {
            return Poll::Pending;
        };
        Box::pin(self.acceptor.accept(stream))
            .poll_unpin(cx)
            .map_ok(|stream| (stream, addr))
            .map_err(|e| IoError::new(ErrorKind::Other, e))
    }

    fn local_addr(&self) -> IoResult<Self::Addr> {
        self.inner.local_addr()
    }
}
