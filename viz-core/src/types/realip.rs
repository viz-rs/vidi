use std::{
    net::{IpAddr, SocketAddr},
    str,
};

use rfc7239::{NodeIdentifier, NodeName};

use crate::{
    header::{HeaderValue, FORWARDED},
    Request, RequestExt, Result,
};

/// Gets real ip remote addr from request headers.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RealIp(pub IpAddr);

impl RealIp {
    /// X-Real-IP header.
    pub const X_REAL_IP: &'static str = "x-real-ip";

    /// X-Forwarded-For header.
    pub const X_FORWARDED_FOR: &'static str = "x-forwarded-for";

    /// Parse the headers.
    pub fn parse(req: &Request) -> Option<Self> {
        req.headers()
            .get(Self::X_REAL_IP)
            .map(HeaderValue::to_str)
            .and_then(Result::ok)
            .map(str::parse)
            .and_then(Result::ok)
            .or_else(|| {
                req.headers()
                    .get(FORWARDED)
                    .map(HeaderValue::to_str)
                    .and_then(Result::ok)
                    .map(rfc7239::parse)
                    .map(Iterator::collect)
                    .and_then(Result::ok)
                    .map(Vec::into_iter)
                    .and_then(|mut value| {
                        value.find_map(|item| match item.forwarded_for {
                            Some(NodeIdentifier {
                                name: NodeName::Ip(ip_addr),
                                ..
                            }) => Some(ip_addr),
                            _ => None,
                        })
                    })
            })
            .or_else(|| {
                req.headers()
                    .get(Self::X_FORWARDED_FOR)
                    .map(HeaderValue::to_str)
                    .and_then(Result::ok)
                    .and_then(|value| {
                        value
                            .split(',')
                            .map(str::trim)
                            .map(str::parse)
                            .find_map(Result::ok)
                    })
            })
            .map(RealIp)
            .or_else(|| req.remote_addr().map(SocketAddr::ip).map(RealIp))
    }
}
