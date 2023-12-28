use std::task::{Context, Poll};

use tower::Service;
use viz_core::{future::BoxFuture, Error, Handler, Request, Response, Result};

/// An adapter that makes a [`Handler`] into a [`Service`].
#[derive(Debug)]
pub struct HandlerService<H>(H);

impl<H> HandlerService<H> {
    /// Creates a new [`HandlerService`].
    pub fn new(h: H) -> Self {
        Self(h)
    }
}

impl<H> Clone for HandlerService<H>
where
    H: Clone,
{
    fn clone(&self) -> Self {
        HandlerService(self.0.clone())
    }
}

impl<H> Service<Request> for HandlerService<H>
where
    H: Handler<Request, Output = Result<Response>> + Send + Clone + 'static,
{
    type Response = Response;
    type Error = Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let handler = self.0.clone();
        Box::pin(async move { handler.call(req).await })
    }
}
