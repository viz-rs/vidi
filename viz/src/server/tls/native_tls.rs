use std::{
    convert::Infallible,
    future::{self, Ready},
    io::{Error as IoError, ErrorKind},
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{future::BoxFuture, ready};
use hyper::{
    server::{
        accept::Accept,
        conn::{AddrIncoming, AddrStream},
    },
    service::Service,
};
use tokio_native_tls::{TlsAcceptor as TlsAcceptorWrapper, TlsStream};

use crate::{Error, Responder, Result, ServiceMaker};

use super::{Listener, Stream};

pub use tokio_native_tls::native_tls::{Identity, TlsAcceptor};

/// `native-lts`'s config.
#[derive(Debug)]
pub struct Config {
    identity: Identity,
}

impl Config {
    /// Creates a new config with the specified [Identity].
    pub fn new(identity: Identity) -> Self {
        Self { identity }
    }

    /// Creates a new [TlsAcceptor] wrapper with the specified [Identity].
    pub fn build(self) -> Result<TlsAcceptorWrapper> {
        TlsAcceptor::new(self.identity)
            .map(Into::into)
            .map_err(Error::normal)
    }
}

type TlsAccept<T> = BoxFuture<'static, Result<TlsStream<T>, IoError>>;

impl Accept for Listener<AddrIncoming, TlsAcceptorWrapper, AddrStream> {
    type Conn = Stream<TlsAccept<AddrStream>, TlsStream<AddrStream>>;
    type Error = IoError;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        match ready!(Pin::new(&mut self.inner).poll_accept(cx)) {
            Some(Ok(sock)) => Poll::Ready(Some(Ok({
                let remote_addr = sock.remote_addr();
                let acceptor = self.acceptor.clone();
                Stream::new(
                    Box::pin(async move {
                        acceptor
                            .accept(sock)
                            .await
                            .map_err(|e| IoError::new(ErrorKind::InvalidData, e))
                    }),
                    Some(remote_addr),
                )
            }))),
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}

impl Service<&Stream<TlsAccept<AddrStream>, TlsStream<AddrStream>>> for ServiceMaker {
    type Response = Responder;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, t: &Stream<TlsAccept<AddrStream>, TlsStream<AddrStream>>) -> Self::Future {
        future::ready(Ok(Responder::new(self.tree.clone(), t.remote_addr)))
    }
}
