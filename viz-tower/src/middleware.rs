use tower::{Layer, Service, ServiceExt};
use viz_core::{
    future::{BoxFuture, TryFutureExt},
    Body, BoxError, Bytes, Error, Handler, HttpBody, Request, Response, Result,
};

use crate::HandlerService;

/// A [`Service`] created from a [`Handler`] by applying a Tower middleware.
#[derive(Debug, Clone)]
pub struct Middleware<L, H> {
    l: L,
    h: H,
}

impl<L, H> Middleware<L, H> {
    /// Creates a new tower middleware.
    pub fn new(l: L, h: H) -> Self {
        Self { l, h }
    }
}

impl<O, L, H> Handler<Request> for Middleware<L, H>
where
    L: Layer<HandlerService<H>> + Send + Sync + Clone + 'static,
    H: Handler<Request, Output = Result<Response>> + Send + Clone + 'static,
    O: HttpBody + Send + 'static,
    O::Data: Into<Bytes>,
    O::Error: Into<BoxError>,
    L::Service: Service<Request, Response = Response<O>> + Send + Clone + 'static,
    <L::Service as Service<Request>>::Future: Send,
    <L::Service as Service<Request>>::Error: Into<BoxError>,
{
    type Output = Result<Response>;

    fn call(&self, req: Request) -> BoxFuture<'static, Self::Output> {
        Box::pin(
            self.l
                .clone()
                .layer(HandlerService::new(self.h.clone()))
                .oneshot(req)
                .map_ok(|resp| resp.map(Body::wrap))
                .map_err(Error::boxed),
        )
    }
}
