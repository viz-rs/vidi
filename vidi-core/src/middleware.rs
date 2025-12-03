//! Built-in Middleware.

#[cfg(feature = "cookie")]
pub mod cookie;
#[cfg(feature = "cors")]
pub mod cors;
#[cfg(feature = "csrf")]
pub mod csrf;
#[cfg(feature = "limits")]
pub mod limits;
#[cfg(feature = "session")]
pub mod session;

#[cfg(all(feature = "params", feature = "otel"))]
pub mod otel;

#[cfg(feature = "cookie")]
pub mod helper;

#[cfg(feature = "compression")]
pub mod compression;
