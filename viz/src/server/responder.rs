use std::{
    convert::Infallible,
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use crate::{
    types::{Params, RouteInfo},
    Handler, IntoResponse, Method, Request, RequestExt, Response, StatusCode, Tree,
};

/// Handles the HTTP [`Request`] and retures the HTTP [`Response`].
#[derive(Debug)]
pub struct Responder {
    tree: Arc<Tree>,
    addr: Option<SocketAddr>,
}

impl Responder {
    /// Creates a Responder for handling the [`Request`].
    #[must_use]
    pub fn new(tree: Arc<Tree>, addr: Option<SocketAddr>) -> Self {
        Self { tree, addr }
    }

    /// Serves a request and returns a response.
    async fn serve(
        mut req: Request,
        tree: Arc<Tree>,
        addr: Option<SocketAddr>,
    ) -> Result<Response, Infallible> {
        let method = req.method().clone();
        let path = req.path().to_owned();
        let responded = Ok(
            match tree.find(&method, &path).or_else(|| {
                if method == Method::HEAD {
                    tree.find(&Method::GET, &path)
                } else {
                    None
                }
            }) {
                Some((handler, route)) => {
                    req.extensions_mut().insert(addr);
                    req.extensions_mut().insert(Arc::from(RouteInfo {
                        id: *route.id,
                        pattern: route.pattern(),
                        params: Into::<Params>::into(route.params()),
                    }));
                    // req.set_state(tree.clone());
                    handler
                        .call(req)
                        .await
                        .unwrap_or_else(IntoResponse::into_response)
                }
                None => StatusCode::NOT_FOUND.into_response(),
            },
        );
        responded
    }
}

impl hyper::service::Service<Request> for Responder {
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: Request) -> Self::Future {
        Box::pin(Self::serve(req, self.tree.clone(), self.addr))
    }
}
