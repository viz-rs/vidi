//! Request tracing middleware with [OpenTelemetry].
//!
//! [OpenTelemetry]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/http.md

use std::sync::Arc;

use http::uri::Scheme;
use opentelemetry::{
    global,
    propagation::Extractor,
    trace::{
        FutureExt as OtelFutureExt, OrderMap, Span, SpanKind, Status, TraceContextExt, Tracer,
    },
    Context, Key, Value,
};
use opentelemetry_semantic_conventions::trace::{
    EXCEPTION_MESSAGE,
    HTTP_CLIENT_IP,
    HTTP_FLAVOR,
    HTTP_HOST,
    HTTP_METHOD,
    HTTP_RESPONSE_CONTENT_LENGTH,
    HTTP_ROUTE,
    HTTP_SCHEME,
    // , HTTP_SERVER_NAME
    HTTP_STATUS_CODE,
    HTTP_TARGET,
    HTTP_USER_AGENT,
    NET_HOST_PORT,
    NET_PEER_IP,
};

use crate::{
    async_trait,
    header::{HeaderMap, USER_AGENT},
    headers::{self, HeaderMapExt},
    types::RealIp,
    Handler, IntoResponse, Request, RequestExt, Response, Result, Transform,
};

/// Opentelemetry tracing config.
#[derive(Debug)]
pub struct Config<T> {
    tracer: Arc<T>,
}

impl<T> Config<T> {
    /// Creats new Opentelemetry tracing config.
    pub fn new(t: T) -> Self {
        Self {
            tracer: Arc::new(t),
        }
    }
}

impl<H, T> Transform<H> for Config<T> {
    type Output = TracingMiddleware<H, T>;

    fn transform(&self, h: H) -> Self::Output {
        TracingMiddleware {
            h,
            tracer: self.tracer.clone(),
        }
    }
}

/// OpenTelemetry tracing middleware.
#[derive(Debug, Clone)]
pub struct TracingMiddleware<H, T> {
    h: H,
    tracer: Arc<T>,
}

#[async_trait]
impl<H, O, T> Handler<Request> for TracingMiddleware<H, T>
where
    T: Tracer + Send + Sync + Clone + 'static,
    T::Span: Send + Sync + 'static,
    O: IntoResponse,
    H: Handler<Request, Output = Result<O>> + Clone,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        let parent_context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&RequestHeaderCarrier::new(req.headers()))
        });

        let http_route = &req.route().path;
        let attributes = build_attributes(&req, http_route);

        let mut span = self
            .tracer
            .span_builder(format!("{} {}", req.method(), http_route))
            .with_kind(SpanKind::Server)
            .with_attributes_map(attributes)
            .start_with_context(&*self.tracer, &parent_context);

        span.add_event("request.started".to_string(), vec![]);

        let res = self
            .h
            .call(req)
            .with_context(Context::current_with_span(span))
            .await;

        let cx = Context::current();
        let span = cx.span();

        match res {
            Ok(resp) => {
                let resp = resp.into_response();
                span.add_event("request.completed".to_string(), vec![]);
                span.set_attribute(HTTP_STATUS_CODE.i64(resp.status().as_u16() as i64));
                if let Some(content_length) = resp.headers().typed_get::<headers::ContentLength>() {
                    span.set_attribute(HTTP_RESPONSE_CONTENT_LENGTH.i64(content_length.0 as i64));
                }
                if resp.status().is_server_error() {
                    span.set_status(Status::error(
                        resp.status()
                            .canonical_reason()
                            .map(ToString::to_string)
                            .unwrap_or_default(),
                    ));
                };
                span.end();
                Ok(resp)
            }
            Err(err) => {
                span.add_event(
                    "request.error".to_string(),
                    vec![EXCEPTION_MESSAGE.string(err.to_string())],
                );
                span.set_status(Status::error(err.to_string()));
                span.end();
                Err(err)
            }
        }
    }
}

struct RequestHeaderCarrier<'a> {
    headers: &'a HeaderMap,
}

impl<'a> RequestHeaderCarrier<'a> {
    fn new(headers: &'a HeaderMap) -> Self {
        RequestHeaderCarrier { headers }
    }
}

impl<'a> Extractor for RequestHeaderCarrier<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|header| header.as_str()).collect()
    }
}

fn build_attributes(req: &Request, http_route: &String) -> OrderMap<Key, Value> {
    let mut attributes = OrderMap::<Key, Value>::with_capacity(10);
    attributes.insert(
        HTTP_SCHEME,
        req.schema()
            .or_else(|| Some(&Scheme::HTTP))
            .map(ToString::to_string)
            .unwrap()
            .into(),
    );
    attributes.insert(HTTP_FLAVOR, format!("{:?}", req.version()).into());
    attributes.insert(HTTP_METHOD, req.method().to_string().into());
    attributes.insert(HTTP_ROUTE, http_route.to_owned().into());
    if let Some(path_and_query) = req.uri().path_and_query() {
        attributes.insert(HTTP_TARGET, path_and_query.as_str().to_string().into());
    }
    if let Some(host) = req.uri().host() {
        attributes.insert(HTTP_HOST, host.to_string().into());
    }
    if let Some(user_agent) = req.headers().get(USER_AGENT).and_then(|s| s.to_str().ok()) {
        attributes.insert(HTTP_USER_AGENT, user_agent.to_string().into());
    }
    let realip = RealIp::parse(req);
    if let Some(realip) = realip {
        attributes.insert(HTTP_CLIENT_IP, realip.0.to_string().into());
    }
    // if server_name != host {
    //     attributes.insert(HTTP_SERVER_NAME, server_name.to_string().into());
    // }
    if let Some(remote_ip) = req.remote_addr().map(|add| add.ip()) {
        if realip.map(|realip| realip.0 != remote_ip).unwrap_or(true) {
            // Client is going through a proxy
            attributes.insert(NET_PEER_IP, remote_ip.to_string().into());
        }
    }
    if let Some(port) = req
        .uri()
        .port_u16()
        .filter(|port| *port != 80 || *port != 443)
    {
        attributes.insert(NET_HOST_PORT, port.to_string().into());
    }

    attributes
}
