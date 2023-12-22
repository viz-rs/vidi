//! An adapter that makes a tower [`Service`] into a [`Handler`].

use http_body_util::BodyExt;
use tower::{Service, ServiceExt};
use viz_core::{async_trait, BoxError, Bytes, Error, Handler, HttpBody, Request, Response, Result};

/// Converts a tower [`Service`] into a [`Handler`].
#[derive(Debug, Clone)]
pub struct TowerServiceHandler<S>(S);

impl<S> TowerServiceHandler<S> {
    /// Creates a new [`TowerServiceHandler`].
    pub fn new(s: S) -> Self {
        Self(s)
    }
}

#[async_trait]
impl<O, S> Handler<Request> for TowerServiceHandler<S>
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
            .map(|resp| {
                resp.map(|body| {
                    body.map_frame(|f| f.map_data(Into::into))
                        .map_err(Error::boxed)
                        .boxed_unsync()
                        .into()
                })
            })
            .map_err(Error::boxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc,
        },
        time::Duration,
    };
    use tower::util::{MapErrLayer, MapRequestLayer, MapResponseLayer};
    use tower::{service_fn, ServiceBuilder};
    use tower_http::{
        limit::RequestBodyLimitLayer,
        request_id::{MakeRequestId, RequestId, SetRequestIdLayer},
        timeout::TimeoutLayer,
    };
    use viz_core::{
        Body, BoxHandler, Handler, HandlerExt, IntoResponse, Request, RequestExt, Response,
    };

    #[derive(Clone, Default, Debug)]
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
            .layer(TimeoutLayer::new(Duration::from_secs(10)))
            .service(hello_svc);

        let r0 = Request::new(Body::Full("12".into()));
        let h0 = TowerServiceHandler::new(svc);
        assert!(h0.call(r0).await.is_err());

        let r1 = Request::new(Body::Full("1".into()));
        let b0: BoxHandler = h0.boxed();
        assert!(b0.call(r1).await.is_ok());
    }
}
