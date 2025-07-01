//! Request metrics middleware with [`OpenTelemetry`].
//!
//! [`OpenTelemetry`]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/semantic_conventions/http-metrics.md

use std::time::SystemTime;

use http::uri::Scheme;
use opentelemetry::{
    KeyValue,
    metrics::{Histogram, Meter, UpDownCounter},
};
use opentelemetry_semantic_conventions::trace::{
    CLIENT_ADDRESS, HTTP_REQUEST_METHOD, HTTP_RESPONSE_STATUS_CODE, HTTP_ROUTE,
    NETWORK_PROTOCOL_VERSION, SERVER_ADDRESS, SERVER_PORT, URL_SCHEME,
};

use crate::{Handler, IntoResponse, Request, RequestExt, Response, ResponseExt, Result, Transform};

const HTTP_SERVER_ACTIVE_REQUESTS: &str = "http.server.active_requests";
const HTTP_SERVER_DURATION: &str = "http.server.duration";
const HTTP_SERVER_REQUEST_SIZE: &str = "http.server.request.size";
const HTTP_SERVER_RESPONSE_SIZE: &str = "http.server.response.size";

/// Request metrics middleware config.
#[derive(Clone, Debug)]
pub struct Config {
    active_requests: UpDownCounter<i64>,
    duration: Histogram<f64>,
    request_size: Histogram<u64>,
    response_size: Histogram<u64>,
}

impl Config {
    /// Creates a new Config
    #[must_use]
    pub fn new(meter: &Meter) -> Self {
        let active_requests = meter
            .i64_up_down_counter(HTTP_SERVER_ACTIVE_REQUESTS)
            .with_description(
                "Measures the number of concurrent HTTP requests that are currently in-flight.",
            )
            .with_unit("{request}")
            .build();

        let duration = meter
            .f64_histogram(HTTP_SERVER_DURATION)
            .with_description("Measures the duration of inbound HTTP requests.")
            .with_unit("s")
            .build();

        let request_size = meter
            .u64_histogram(HTTP_SERVER_REQUEST_SIZE)
            .with_description("Measures the size of HTTP request messages (compressed).")
            .with_unit("By")
            .build();

        let response_size = meter
            .u64_histogram(HTTP_SERVER_RESPONSE_SIZE)
            .with_description("Measures the size of HTTP request messages (compressed).")
            .with_unit("By")
            .build();

        Self {
            active_requests,
            duration,
            request_size,
            response_size,
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
            request_size: self.request_size.clone(),
            response_size: self.response_size.clone(),
        }
    }
}

/// Request metrics middleware with `OpenTelemetry`.
#[derive(Clone, Debug)]
pub struct MetricsMiddleware<H> {
    h: H,
    active_requests: UpDownCounter<i64>,
    duration: Histogram<f64>,
    request_size: Histogram<u64>,
    response_size: Histogram<u64>,
}

#[crate::async_trait]
impl<H, O> Handler<Request> for MetricsMiddleware<H>
where
    H: Handler<Request, Output = Result<O>>,
    O: IntoResponse,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        let Self {
            active_requests,
            duration,
            request_size,
            response_size,
            h,
        } = self;

        let timer = SystemTime::now();
        let mut attributes = build_attributes(&req, req.route_info().pattern.as_str());

        active_requests.add(1, &attributes);

        request_size.record(req.content_length().unwrap_or(0), &attributes);

        let resp = h
            .call(req)
            .await
            .map(IntoResponse::into_response)
            .inspect(|resp| {
                active_requests.add(-1, &attributes);

                attributes.push(KeyValue::new(
                    HTTP_RESPONSE_STATUS_CODE,
                    i64::from(resp.status().as_u16()),
                ));

                response_size.record(resp.content_length().unwrap_or(0), &attributes);
            });

        duration.record(
            timer.elapsed().map(|t| t.as_secs_f64()).unwrap_or_default(),
            &attributes,
        );

        resp
    }
}

fn build_attributes(req: &Request, http_route: &str) -> Vec<KeyValue> {
    let mut attributes = Vec::with_capacity(5);
    // <https://github.com/open-telemetry/semantic-conventions/blob/v1.21.0/docs/http/http-spans.md#http-server>
    attributes.push(KeyValue::new(HTTP_ROUTE, http_route.to_string()));

    // <https://github.com/open-telemetry/semantic-conventions/blob/v1.21.0/docs/http/http-spans.md#common-attributes>
    attributes.push(KeyValue::new(HTTP_REQUEST_METHOD, req.method().to_string()));
    attributes.push(KeyValue::new(
        NETWORK_PROTOCOL_VERSION,
        format!("{:?}", req.version()),
    ));

    if let Some(remote_addr) = req.remote_addr() {
        attributes.push(KeyValue::new(CLIENT_ADDRESS, remote_addr.to_string()));
    }

    let uri = req.uri();
    if let Some(host) = uri.host() {
        attributes.push(KeyValue::new(SERVER_ADDRESS, host.to_string()));
    }
    if let Some(port) = uri
        .port_u16()
        .map(i64::from)
        .filter(|port| *port != 80 && *port != 443)
    {
        attributes.push(KeyValue::new(SERVER_PORT, port));
    }

    attributes.push(KeyValue::new(
        URL_SCHEME,
        uri.scheme().unwrap_or(&Scheme::HTTP).to_string(),
    ));

    attributes
}
