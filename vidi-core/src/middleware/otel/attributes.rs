use http::uri::Scheme;
use opentelemetry::KeyValue;
use opentelemetry_semantic_conventions::trace::{
    CLIENT_ADDRESS, HTTP_REQUEST_METHOD, HTTP_ROUTE, NETWORK_PROTOCOL_VERSION, SERVER_ADDRESS,
    SERVER_PORT, URL_SCHEME,
};

use crate::{Request, RequestExt};

/// Builds the HTTP attributes shared by the tracing and metrics middleware.
pub(crate) fn build_attributes(req: &Request, http_route: &str) -> Vec<KeyValue> {
    let mut attributes = Vec::with_capacity(7);
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
