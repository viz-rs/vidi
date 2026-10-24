use std::{convert::Infallible, future::Future, pin::Pin, sync::Arc};

use hyper::service::Service;
use vidi_core::{
    Body, Handler, Incoming, IntoResponse, Request, Response, StatusCode, types::RouteInfo,
};

use crate::Tree;

/// Handles the HTTP [`Request`] and returns the HTTP [`Response`].
#[derive(Debug)]
pub struct Responder<A> {
    tree: Arc<Tree>,
    remote_addr: Option<A>,
}

impl<A> Responder<A>
where
    A: Clone + Send + Sync + 'static,
{
    /// Creates a [`Responder`].
    #[must_use]
    pub fn new(tree: Arc<Tree>, remote_addr: Option<A>) -> Self {
        Self { tree, remote_addr }
    }
}

impl<A> Service<Request<Incoming>> for Responder<A>
where
    A: Clone + Send + Sync + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, mut req: Request<Incoming>) -> Self::Future {
        let method = req.method().clone();
        let path = req.uri().path().to_owned();

        let Some((handler, route)) = self.tree.find_with_head(&method, &path) else {
            return Box::pin(async move { Ok(StatusCode::NOT_FOUND.into_response()) });
        };

        let extensions = req.extensions_mut();

        extensions.insert(self.remote_addr.clone());
        extensions.insert(Arc::from(RouteInfo::new(
            *route.id,
            route.pattern(),
            route.params(),
        )));

        let handler = handler.clone();

        Box::pin(async move {
            Ok(handler
                .call(req.map(Body::Incoming))
                .await
                .unwrap_or_else(IntoResponse::into_response))
        })
    }
}
