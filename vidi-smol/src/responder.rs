use std::{convert::Infallible, future::Future, pin::Pin, sync::Arc};

use crate::{Body, Handler, Incoming, IntoResponse, Method, Request, Response, StatusCode, Tree};

/// Handles the HTTP [`Request`] and retures the HTTP [`Response`].
#[derive(Debug)]
pub struct Responder<A> {
    tree: Arc<Tree>,
    remote_addr: Option<A>,
}

impl<A> Responder<A>
where
    A: Clone + Send + Sync + 'static,
{
    /// Creates a Responder for handling the [`Request`].
    #[must_use]
    pub fn new(tree: Arc<Tree>, remote_addr: Option<A>) -> Self {
        Self { tree, remote_addr }
    }
}

impl<A> hyper::service::Service<Request<Incoming>> for Responder<A>
where
    A: Clone + Send + Sync + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, mut req: Request<Incoming>) -> Self::Future {
        let method = req.method().clone();
        let path = req.uri().path().to_owned();

        let Some((handler, route)) = self.tree.find(&method, &path).or_else(|| {
            if method == Method::HEAD {
                self.tree.find(&Method::GET, &path)
            } else {
                None
            }
        }) else {
            return Box::pin(async move { Ok(StatusCode::NOT_FOUND.into_response()) });
        };

        req.extensions_mut().insert(self.remote_addr.clone());
        req.extensions_mut()
            .insert(Arc::from(crate::types::RouteInfo {
                id: *route.id,
                pattern: route.pattern(),
                params: route.params().into(),
            }));

        let handler = handler.clone();

        Box::pin(async move {
            Ok(handler
                .call(req.map(Body::Incoming))
                .await
                .unwrap_or_else(IntoResponse::into_response))
        })
    }
}
