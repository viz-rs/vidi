use std::sync::Arc;

#[cfg(any(
    all(unix, feature = "unix-socket"),
    any(feature = "http1", feature = "http2")
))]
use std::{
    convert::Infallible,
    future::{ready, Ready},
    task::{Context, Poll},
};

use crate::{Router, Tree};

/// Service Maker
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ServiceMaker {
    pub(crate) tree: Arc<Tree>,
}

impl ServiceMaker {
    /// Creates a service from router
    pub fn new(router: Router) -> Self {
        Self {
            tree: Arc::new(router.into()),
        }
    }
}

impl From<Router> for ServiceMaker {
    fn from(router: Router) -> Self {
        Self::new(router)
    }
}

// hyper-v0.14
#[cfg(any(feature = "http1", feature = "http2"))]
impl hyper::service::Service<&hyper::server::conn::AddrStream> for ServiceMaker {
    type Response = super::Responder;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, socket: &hyper::server::conn::AddrStream) -> Self::Future {
        ready(Ok(super::Responder::new(
            self.tree.clone(),
            Some(socket.remote_addr()),
        )))
    }
}

// hyper-v1.0
/*
#[cfg(any(feature = "http1", feature = "http2"))]
impl Service<&tokio::net::TcpStream> for ServiceMaker {
    type Response = super::Responder;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, socket: &tokio::net::TcpStream) -> Self::Future {
        if let Err(_) = socket.set_nodelay(true) {
            // TODO: trace error
        }
        ready(Ok(super::Responder::new(self.tree.clone(), socket.peer_addr().ok())))
    }
}
*/

#[cfg(all(unix, feature = "unix-socket"))]
impl hyper::service::Service<&tokio::net::UnixStream> for ServiceMaker {
    type Response = super::Responder;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: &tokio::net::UnixStream) -> Self::Future {
        ready(Ok(super::Responder::new(self.tree.clone(), None)))
    }
}
