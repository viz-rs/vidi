use tower::{Layer, Service, ServiceExt};
use viz_core::{
    async_trait, Body, BoxError, Bytes, Error, Handler, HttpBody, Request, Response, Result,
    Transform,
};

use crate::HandlerService;

/// Transforms a Tower layer into Viz Middleware.
#[derive(Debug)]
pub struct Layered<L>(L);

impl<L> Layered<L> {
    /// Creates a new tower layer.
    pub fn new(l: L) -> Self {
        Self(l)
    }
}

impl<L, H> Transform<H> for Layered<L>
where
    L: Clone,
{
    type Output = Middleware<L, H>;

    fn transform(&self, h: H) -> Self::Output {
        Middleware::new(self.0.clone(), h)
    }
}

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

#[async_trait]
impl<O, L, H> Handler<Request> for Middleware<L, H>
where
    L: Layer<HandlerService<H>> + Send + Sync + Clone + 'static,
    H: Handler<Request, Output = Result<Response>> + Send + Sync + Clone + 'static,
    O: HttpBody + Send + 'static,
    O::Data: Into<Bytes>,
    O::Error: Into<BoxError>,
    L::Service: Service<Request, Response = Response<O>> + Send + Sync + Clone + 'static,
    <L::Service as Service<Request>>::Future: Send,
    <L::Service as Service<Request>>::Error: Into<BoxError>,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        self.l
            .clone()
            .layer(HandlerService::new(self.h.clone()))
            .oneshot(req)
            .await
            .map(|resp| resp.map(Body::wrap))
            .map_err(Error::boxed)
    }
}
