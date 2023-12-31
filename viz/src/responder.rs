use std::{convert::Infallible, sync::Arc};

use crate::{
    future::{FutureExt, TryFutureExt},
    types::RouteInfo,
    Body, BoxFuture, Handler, Incoming, IntoResponse, Method, Request, Response, StatusCode, Tree,
};

/// Handles the HTTP [`Request`] and retures the HTTP [`Response`].
#[derive(Debug)]
pub struct Responder<A> {
    tree: Tree,
    remote_addr: Option<A>,
}

impl<A> Responder<A> {
    /// Creates a Responder for handling the [`Request`].
    #[must_use]
    pub fn new(tree: Tree, remote_addr: Option<A>) -> Self {
        Self { tree, remote_addr }
    }
}

impl<A> Handler<Request<Incoming>> for Responder<A>
where
    A: Clone + Send + Sync + 'static,
{
    type Output = Result<Response, Infallible>;

    fn call(&self, mut req: Request<Incoming>) -> BoxFuture<Self::Output> {
        let Self { remote_addr, tree } = self;
        let method = req.method().clone();
        let path = req.uri().path().to_string();

        let matched = tree.find(&method, &path).or_else(|| {
            if method == Method::HEAD {
                tree.find(&Method::GET, &path)
            } else {
                None
            }
        });

        if let Some((handler, route)) = matched {
            req.extensions_mut().insert(remote_addr.clone());
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
            Box::pin(not_found())
        }
    }
}

impl<A> hyper::service::Service<Request<Incoming>> for Responder<A>
where
    A: Clone + Send + Sync + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = BoxFuture<Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        Handler::call(self, req)
    }
}

#[inline(always)]
async fn not_found() -> Result<Response, Infallible> {
    Ok(StatusCode::NOT_FOUND.into_response())
}
