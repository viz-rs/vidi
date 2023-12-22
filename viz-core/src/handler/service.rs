use http_body_util::BodyExt;
use hyper::service::Service;

use crate::{async_trait, Bytes, Error, Handler, HttpBody, Request, Response, Result};

/// Converts a hyper [`Service`] to a viz [`Handler`].
#[derive(Debug, Clone)]
pub struct ServiceHandler<S> {
    s: S,
}

impl<S> ServiceHandler<S> {
    /// Creates a new [`ServiceHandler`].
    pub fn new(s: S) -> Self {
        Self { s }
    }
}

#[async_trait]
impl<I, O, S> Handler<Request<I>> for ServiceHandler<S>
where
    I: HttpBody + Send + Unpin + 'static,
    O: HttpBody + Send + 'static,
    O::Data: Into<Bytes>,
    O::Error: Into<Error>,
    S: Service<Request<I>, Response = Response<O>> + Send + Sync + Clone + 'static,
    S::Future: Send,
    S::Error: Into<Error>,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request<I>) -> Self::Output {
        self.s
            .call(req)
            .await
            .map(|resp| {
                resp.map(|body| {
                    body.map_frame(|f| f.map_data(Into::into))
                        .map_err(Into::into)
                        .boxed_unsync()
                        .into()
                })
            })
            .map_err(Into::into)
    }
}
