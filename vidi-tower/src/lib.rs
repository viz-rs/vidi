//! An adapter that makes a tower [`Service`] into a [`Handler`].

use tower::{Service, ServiceExt};
use vidi_core::{Body, BoxError, Bytes, Error, Handler, HttpBody, Request, Response, Result};

mod service;
pub use service::HandlerService;

mod middleware;
pub use middleware::Middleware;

mod layer;
pub use layer::Layered;

/// Converts a tower [`Service`] into a [`Handler`].
#[derive(Clone, Debug)]
pub struct ServiceHandler<S>(S);

impl<S> ServiceHandler<S> {
    /// Creates a new [`ServiceHandler`].
    pub const fn new(s: S) -> Self {
        Self(s)
    }
}

#[vidi_core::async_trait]
impl<O, S> Handler<Request> for ServiceHandler<S>
where
    O: HttpBody + Send + 'static,
    O::Data: Into<Bytes>,
    O::Error: Into<BoxError>,
    S: Service<Request, Response = Response<O>> + Send + Sync + Clone + 'static,
    S::Future: Send,
    S::Error: Into<BoxError>,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        self.0
            .clone()
            .oneshot(req)
            .await
            .map_err(Error::boxed)
            .map(|resp| resp.map(Body::wrap))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{
            Arc,
            atomic::{AtomicU64, Ordering},
        },
        time::Duration,
    };
    use tower::util::{MapErrLayer, MapRequestLayer, MapResponseLayer};
    use tower::{ServiceBuilder, service_fn};
    use tower_http::{
        limit::RequestBodyLimitLayer,
        request_id::{MakeRequestId, RequestId, SetRequestIdLayer},
        timeout::TimeoutLayer,
    };
    use vidi_core::{
        Body, BoxHandler, Handler, HandlerExt, IntoResponse, Request, RequestExt, Response,
        StatusCode,
    };

    #[derive(Clone, Debug, Default)]
    struct MyMakeRequestId {
        counter: Arc<AtomicU64>,
    }

    impl MakeRequestId for MyMakeRequestId {
        fn make_request_id<B>(&mut self, _: &Request<B>) -> Option<RequestId> {
            let request_id = self
                .counter
                .fetch_add(1, Ordering::SeqCst)
                .to_string()
                .parse()
                .unwrap();

            Some(RequestId::new(request_id))
        }
    }

    async fn hello(mut req: Request) -> Result<Response> {
        let bytes = req.bytes().await?;
        Ok(bytes.into_response())
    }

    #[tokio::test]
    async fn tower_service_into_handler() {
        let hello_svc = service_fn(hello);

        let svc = ServiceBuilder::new()
            .layer(RequestBodyLimitLayer::new(1))
            .layer(MapErrLayer::new(Error::from))
            .layer(SetRequestIdLayer::x_request_id(MyMakeRequestId::default()))
            .layer(MapResponseLayer::new(IntoResponse::into_response))
            .layer(MapRequestLayer::new(|req: Request<_>| req.map(Body::wrap)))
            .layer(TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                Duration::from_secs(10),
            ))
            .service(hello_svc);

        let r0 = Request::new(Body::Full("12".into()));
        let h0 = ServiceHandler::new(svc);
        assert!(h0.call(r0).await.is_err());

        let r1 = Request::new(Body::Full("1".into()));
        let b0: BoxHandler = h0.boxed();
        assert!(b0.call(r1).await.is_ok());
    }
}
