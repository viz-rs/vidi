use tower::{Layer, Service, ServiceExt};
use vidi_core::{Body, BoxError, Bytes, Error, Handler, HttpBody, Request, Response, Result};

use crate::HandlerService;

/// A [`Service`] created from a [`Handler`] by applying a Tower middleware.
#[derive(Clone, Debug)]
pub struct Middleware<L, H> {
    l: L,
    h: H,
}

impl<L, H> Middleware<L, H> {
    /// Creates a new tower middleware.
    pub const fn new(l: L, h: H) -> Self {
        Self { l, h }
    }
}

#[vidi_core::async_trait]
impl<O, L, H> Handler<Request> for Middleware<L, H>
where
    L: Layer<HandlerService<H>> + Send + Sync + 'static,
    H: Handler<Request, Output = Result<Response>> + Clone,
    O: HttpBody + Send + 'static,
    O::Data: Into<Bytes>,
    O::Error: Into<BoxError>,
    L::Service: Service<Request, Response = Response<O>> + Send + Sync + 'static,
    <L::Service as Service<Request>>::Future: Send,
    <L::Service as Service<Request>>::Error: Into<BoxError>,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        self.l
            .layer(HandlerService::new(self.h.clone()))
            .oneshot(req)
            .await
            .map_err(Error::boxed)
            .map(|resp| resp.map(Body::wrap))
    }
}
