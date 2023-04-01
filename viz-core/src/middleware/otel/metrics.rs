//! Request metrics middleware with [`OpenTelemetry`].
//!
//! [`OpenTelemetry`]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/semantic_conventions/http-metrics.md

use std::time::SystemTime;

use http::uri::Scheme;
use opentelemetry::{
    metrics::{Histogram, Meter, Unit, UpDownCounter},
    Context, KeyValue,
};
use opentelemetry_semantic_conventions::trace::{
    HTTP_CLIENT_IP,
    HTTP_FLAVOR,
    HTTP_METHOD,
    // HTTP_RESPONSE_CONTENT_LENGTH,
    HTTP_ROUTE,
    HTTP_SCHEME, // , HTTP_SERVER_NAME
    HTTP_STATUS_CODE,
    HTTP_TARGET,
    HTTP_USER_AGENT,
    NET_HOST_PORT,
    NET_SOCK_PEER_ADDR,
};

use super::HTTP_HOST;

use crate::{
    async_trait, headers::UserAgent, types::RealIp, Handler, IntoResponse, Request, RequestExt,
    Response, Result, Transform,
};

const HTTP_SERVER_ACTIVE_REQUESTS: &str = "http.server.active_requests";
const HTTP_SERVER_DURATION: &str = "http.server.duration";

/// Request metrics middleware config.
#[derive(Clone, Debug)]
pub struct Config {
    active_requests: UpDownCounter<i64>,
    duration: Histogram<f64>,
}

impl Config {
    /// Creates a new Config
    #[must_use]
    pub fn new(meter: &Meter) -> Self {
        let active_requests = meter
            .i64_up_down_counter(HTTP_SERVER_ACTIVE_REQUESTS)
            .with_description("HTTP concurrent in-flight requests per route")
            .init();

        let duration = meter
            .f64_histogram(HTTP_SERVER_DURATION)
            .with_description("HTTP inbound request duration per route")
            .with_unit(Unit::new("ms"))
            .init();

        Config {
            active_requests,
            duration,
        }
    }
}

impl<H> Transform<H> for Config {
    type Output = MetricsMiddleware<H>;

    fn transform(&self, h: H) -> Self::Output {
        MetricsMiddleware {
            h,
            active_requests: self.active_requests.clone(),
            duration: self.duration.clone(),
        }
    }
}

/// Request metrics middleware with `OpenTelemetry`.
#[derive(Debug, Clone)]
pub struct MetricsMiddleware<H> {
    h: H,
    active_requests: UpDownCounter<i64>,
    duration: Histogram<f64>,
}

#[async_trait]
impl<H, O> Handler<Request> for MetricsMiddleware<H>
where
    O: IntoResponse,
    H: Handler<Request, Output = Result<O>> + Clone,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        let timer = SystemTime::now();
        let cx = Context::current();
        let mut attributes = build_attributes(&req, req.route_info().pattern.as_str());

        self.active_requests.add(&cx, 1, &attributes);

        let resp = self
            .h
            .call(req)
            .await
            .map(IntoResponse::into_response)
            .map(|resp| {
                self.active_requests.add(&cx, -1, &attributes);

                attributes.push(HTTP_STATUS_CODE.i64(i64::from(resp.status().as_u16())));

                resp
            });

        self.duration.record(
            &cx,
            timer
                .elapsed()
                .map(|t| t.as_secs_f64() * 1000.0)
                .unwrap_or_default(),
            &attributes,
        );

        resp
    }
}

fn build_attributes(req: &Request, http_route: &str) -> Vec<KeyValue> {
    let mut attributes = Vec::with_capacity(10);
    attributes.push(
        HTTP_SCHEME.string(
            req.schema()
                .or(Some(&Scheme::HTTP))
                .map(ToString::to_string)
                .unwrap(),
        ),
    );
    attributes.push(HTTP_FLAVOR.string(format!("{:?}", req.version())));
    attributes.push(HTTP_METHOD.string(req.method().to_string()));
    attributes.push(HTTP_ROUTE.string(http_route.to_string()));
    if let Some(path_and_query) = req.uri().path_and_query() {
        attributes.push(HTTP_TARGET.string(path_and_query.as_str().to_string()));
    }
    if let Some(host) = req.uri().host() {
        attributes.push(HTTP_HOST.string(host.to_string()));
    }
    if let Some(user_agent) = req
        .header_typed::<UserAgent>()
        .as_ref()
        .map(UserAgent::as_str)
    {
        attributes.push(HTTP_USER_AGENT.string(user_agent.to_string()));
    }
    let realip = RealIp::parse(req);
    if let Some(realip) = realip {
        attributes.push(HTTP_CLIENT_IP.string(realip.0.to_string()));
    }
    // if server_name != host {
    //     attributes.insert(HTTP_SERVER_NAME, server_name.to_string().into());
    // }
    if let Some(remote_ip) = req.remote_addr().map(std::net::SocketAddr::ip) {
        if realip.map_or(true, |realip| realip.0 != remote_ip) {
            // Client is going through a proxy
            attributes.push(NET_SOCK_PEER_ADDR.string(remote_ip.to_string()));
        }
    }
    if let Some(port) = req
        .uri()
        .port_u16()
        .filter(|port| *port != 80 || *port != 443)
    {
        attributes.push(NET_HOST_PORT.i64(i64::from(port)));
    }

    attributes
}
