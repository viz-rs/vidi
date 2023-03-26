use std::{net::IpAddr, str};

use rfc7239::{NodeIdentifier, NodeName};

use crate::{Request, RequestExt, Result};

/// Gets real ip remote addr from request headers.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RealIp(pub IpAddr);

impl RealIp {
    /// Parse the headers.
    pub fn parse(req: &Request) -> Option<Self> {
        if let Some(real_ip) = req
            .headers()
            .get("x-real-ip")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<IpAddr>().ok())
        {
            return Some(RealIp(real_ip));
        }

        if let Some(forwarded) = req
            .headers()
            .get("forwarded")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| rfc7239::parse(value).collect::<Result<Vec<_>, _>>().ok())
        {
            if let Some(real_ip) = forwarded
                .into_iter()
                .find_map(|item| match item.forwarded_for {
                    Some(NodeIdentifier {
                        name: NodeName::Ip(ip_addr),
                        ..
                    }) => Some(ip_addr),
                    _ => None,
                })
            {
                return Some(RealIp(real_ip));
            }
        }

        if let Some(real_ip) = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .find_map(|value| value.parse::<IpAddr>().ok())
            })
        {
            return Some(RealIp(real_ip));
        }

        req.remote_addr().map(|addr| RealIp(addr.ip()))
    }
}
