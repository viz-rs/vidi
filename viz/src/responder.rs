use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use crate::{
    future::{FutureExt, TryFutureExt},
    types::RouteInfo,
    Body, BoxFuture, Handler, Incoming, IntoResponse, Method, Request, Response, StatusCode, Tree,
};

/// Handles the HTTP [`Request`] and retures the HTTP [`Response`].
#[derive(Debug)]
pub struct Responder {
    tree: Tree,
    addr: Option<SocketAddr>,
}

impl Responder {
    /// Creates a Responder for handling the [`Request`].
    #[must_use]
    pub fn new(tree: Tree, addr: Option<SocketAddr>) -> Self {
        Self { tree, addr }
    }
}

impl Handler<Request<Incoming>> for Responder {
    type Output = Result<Response, Infallible>;

    fn call(&self, mut req: Request<Incoming>) -> BoxFuture<Self::Output> {
        let method = req.method().clone();
        let path = req.uri().path().to_string();

        let matched = self.tree.find(&method, &path).or_else(|| {
            if method == Method::HEAD {
                self.tree.find(&Method::GET, &path)
            } else {
                None
            }
        });

        if let Some((handler, route)) = matched {
            req.extensions_mut().insert(self.addr);
            req.extensions_mut().insert(Arc::from(RouteInfo {
                id: *route.id,
                pattern: route.pattern(),
                params: route.params().into(),
            }));

            Box::pin(
                handler
                    .call(req.map(Body::Incoming))
                    .unwrap_or_else(IntoResponse::into_response)
                    .map(Result::Ok),
            )
        } else {
            Box::pin(async { Ok(StatusCode::NOT_FOUND.into_response()) })
        }
    }
}

impl hyper::service::Service<Request<Incoming>> for Responder {
    type Response = Response;
    type Error = Infallible;
    type Future = BoxFuture<Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        Handler::call(self, req)
    }
}
