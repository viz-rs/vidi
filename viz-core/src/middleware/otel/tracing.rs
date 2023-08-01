//! Request tracing middleware with [`OpenTelemetry`].
//!
//! [`OpenTelemetry`]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/http.md

use std::{net::SocketAddr, sync::Arc};

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
    CLIENT_ADDRESS, CLIENT_SOCKET_ADDRESS, EXCEPTION_MESSAGE, HTTP_REQUEST_BODY_SIZE,
    HTTP_REQUEST_METHOD, HTTP_RESPONSE_CONTENT_LENGTH, HTTP_RESPONSE_STATUS_CODE, HTTP_ROUTE,
    NETWORK_PROTOCOL_VERSION, SERVER_ADDRESS, SERVER_PORT, URL_PATH, URL_QUERY, URL_SCHEME,
    USER_AGENT_ORIGINAL,
};

use crate::{
    async_trait,
    header::{HeaderMap, HeaderName},
    headers::UserAgent,
    Handler, IntoResponse, Request, RequestExt, Response, ResponseExt, Result, Transform,
};

/// `OpenTelemetry` tracing config.
#[derive(Debug)]
pub struct Config<T> {
    tracer: Arc<T>,
}

impl<T> Config<T> {
    /// Creats new `OpenTelemetry` tracing config.
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

/// `OpenTelemetry` tracing middleware.
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

        let http_route = &req.route_info().pattern;
        let attributes = build_attributes(&req, http_route.as_str());

        let mut span = self
            .tracer
            .span_builder(format!("{} {}", req.method(), http_route))
            .with_kind(SpanKind::Server)
            .with_attributes_map(attributes)
            .start_with_context(&*self.tracer, &parent_context);

        span.add_event("request.started".to_string(), vec![]);

        let resp = self
            .h
            .call(req)
            .with_context(Context::current_with_span(span))
            .await;

        let cx = Context::current();
        let span = cx.span();

        match resp {
            Ok(resp) => {
                let resp = resp.into_response();
                span.add_event("request.completed".to_string(), vec![]);
                span.set_attribute(
                    HTTP_RESPONSE_STATUS_CODE.i64(i64::from(resp.status().as_u16())),
                );
                if let Some(content_length) = resp.content_length() {
                    span.set_attribute(
                        HTTP_RESPONSE_CONTENT_LENGTH
                            .i64(i64::try_from(content_length).unwrap_or(i64::MAX)),
                    );
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
        self.headers.keys().map(HeaderName::as_str).collect()
    }
}

fn build_attributes(req: &Request, http_route: &str) -> OrderMap<Key, Value> {
    let mut attributes = OrderMap::<Key, Value>::with_capacity(10);
    // <https://github.com/open-telemetry/semantic-conventions/blob/v1.21.0/docs/http/http-spans.md#http-server>
    attributes.insert(HTTP_ROUTE, http_route.to_string().into());

    // <https://github.com/open-telemetry/semantic-conventions/blob/v1.21.0/docs/http/http-spans.md#common-attributes>
    attributes.insert(HTTP_REQUEST_METHOD, req.method().to_string().into());
    attributes.insert(
        NETWORK_PROTOCOL_VERSION,
        format!("{:?}", req.version()).into(),
    );

    let remote_addr = req.remote_addr();
    if let Some(remote_addr) = remote_addr {
        attributes.insert(CLIENT_ADDRESS, remote_addr.to_string().into());
    }
    if let Some(realip) = req.realip().map(|value| value.0).filter(|realip| {
        remote_addr
            .map(SocketAddr::ip)
            .map_or(true, |remoteip| &remoteip != realip)
    }) {
        attributes.insert(CLIENT_SOCKET_ADDRESS, realip.to_string().into());
    }

    let uri = req.uri();
    if let Some(host) = uri.host() {
        attributes.insert(SERVER_ADDRESS, host.to_string().into());
    }
    if let Some(port) = uri.port_u16().filter(|port| *port != 80 && *port != 443) {
        attributes.insert(SERVER_PORT, port as i64);
    }

    if let Some(path_query) = uri.path_and_query() {
        if path_query.path() != "/" {
            attributes.insert(URL_PATH, path_query.path().to_string().into());
        }
        if let Some(query) = path_query.query() {
            attributes.insert(URL_QUERY, query.to_string().into());
        }
    }

    attributes.insert(
        URL_SCHEME,
        uri.scheme().unwrap_or(&Scheme::HTTP).to_string().into(),
    );

    if let Some(content_length) = req
        .content_length()
        .filter(|len| *len > 0)
        .and_then(|len| i64::try_from(len).ok())
    {
        attributes.insert(HTTP_REQUEST_BODY_SIZE, content_length);
    }

    if let Some(user_agent) = req
        .header_typed::<UserAgent>()
        .as_ref()
        .map(UserAgent::as_str)
    {
        attributes.insert(USER_AGENT_ORIGINAL, user_agent.to_string().into());
    }

    attributes
}
