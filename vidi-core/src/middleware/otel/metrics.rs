//! Request metrics middleware with [`OpenTelemetry`].
//!
//! [`OpenTelemetry`]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/semantic_conventions/http-metrics.md

use std::time::SystemTime;

use opentelemetry::{
    KeyValue,
    metrics::{Histogram, Meter, UpDownCounter},
};
use opentelemetry_semantic_conventions::trace::HTTP_RESPONSE_STATUS_CODE;

use crate::{Handler, IntoResponse, Request, RequestExt, Response, ResponseExt, Result, Transform};

const HTTP_SERVER_ACTIVE_REQUESTS: &str = "http.server.active_requests";
const HTTP_SERVER_DURATION: &str = "http.server.duration";
const HTTP_SERVER_REQUEST_SIZE: &str = "http.server.request.size";
const HTTP_SERVER_RESPONSE_SIZE: &str = "http.server.response.size";

/// Request metrics middleware configuration.
#[derive(Clone, Debug)]
pub struct Config {
    active_requests: UpDownCounter<i64>,
    duration: Histogram<f64>,
    request_size: Histogram<u64>,
    response_size: Histogram<u64>,
}

impl Config {
    /// Creates a [`Config`].
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
        let mut attributes = super::build_attributes(&req, req.route_info().pattern.as_str());

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
