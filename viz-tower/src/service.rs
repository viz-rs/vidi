use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use viz_core::{Error, Handler, Request, Response, Result};

/// An adapter that makes a [`Handler`] into a [`Service`](tower::Service).
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

impl<H> tower::Service<Request> for HandlerService<H>
where
    H: Handler<Request, Output = Result<Response>> + Clone,
{
    type Response = Response;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = H::Output> + Send>>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let h = self.0.clone();
        Box::pin(async move { h.call(req).await })
    }
}
