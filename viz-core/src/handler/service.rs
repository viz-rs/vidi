use hyper::service::Service;

use crate::{
    future::TryFutureExt, Body, BoxError, BoxFuture, Bytes, Error, Handler, HttpBody, Request,
    Response, Result,
};

/// Converts a hyper [`Service`] to a viz [`Handler`].
#[derive(Debug, Clone)]
pub struct ServiceHandler<S>(S);

impl<S> ServiceHandler<S> {
    /// Creates a new [`ServiceHandler`].
    pub fn new(s: S) -> Self {
        Self(s)
    }
}

impl<I, O, S> Handler<Request<I>> for ServiceHandler<S>
where
    I: HttpBody + Send + 'static,
    O: HttpBody + Send + 'static,
    O::Data: Into<Bytes>,
    O::Error: Into<BoxError>,
    S: Service<Request<I>, Response = Response<O>> + Send + Sync + Clone + 'static,
    S::Future: Send,
    S::Error: Into<BoxError>,
{
    type Output = Result<Response>;

    fn call(&self, req: Request<I>) -> BoxFuture<Self::Output> {
        Box::pin(
            self.0
                .call(req)
                .map_ok(|resp| resp.map(Body::wrap))
                .map_err(Error::boxed),
        )
    }
}
