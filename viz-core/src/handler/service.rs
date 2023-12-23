use hyper::service::Service;

use crate::{
    async_trait, Body, BoxError, Bytes, Error, Handler, HttpBody, Request, Response, Result,
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

#[async_trait]
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

    async fn call(&self, req: Request<I>) -> Self::Output {
        self.0
            .call(req)
            .await
            .map(|resp| resp.map(Body::wrap))
            .map_err(Error::boxed)
    }
}
