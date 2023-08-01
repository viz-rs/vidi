use std::{net::IpAddr, str};

use rfc7239::{NodeIdentifier, NodeName};

use crate::{header::FORWARDED, Request, RequestExt, Result};

/// Gets real ip remote addr from request headers.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RealIp(pub IpAddr);

impl RealIp {
    /// X-Real-IP header.
    pub const X_REAL_IP: &str = "x-real-ip";

    /// X-Forwarded-For header.
    pub const X_FORWARDED_FOR: &str = "x-forwarded-for";

    /// Parse the headers.
    pub fn parse(req: &Request) -> Option<Self> {
        req.headers()
            .get(Self::X_REAL_IP)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<IpAddr>().ok())
            .or_else(|| {
                req.headers()
                    .get(FORWARDED)
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| rfc7239::parse(value).collect::<Result<Vec<_>, _>>().ok())
                    .and_then(|value| {
                        value.into_iter().find_map(|item| match item.forwarded_for {
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
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| {
                        value
                            .split(',')
                            .map(str::trim)
                            .find_map(|value| value.parse::<IpAddr>().ok())
                    })
            })
            .map(RealIp)
            .or_else(|| req.remote_addr().map(|addr| RealIp(addr.ip())))
    }
}
