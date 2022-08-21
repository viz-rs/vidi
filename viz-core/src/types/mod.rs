//! Built-in Extractors types and traits.

#[cfg(feature = "cookie")]
mod cookie;
#[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
pub use self::cookie::CookieKey;
#[cfg(feature = "cookie")]
pub use self::cookie::{Cookie, CookieJar, Cookies, CookiesError, SameSite};

#[cfg(feature = "state")]
mod state;
#[cfg(feature = "state")]
pub use state::State;

#[cfg(feature = "form")]
mod form;
#[cfg(feature = "form")]
pub use form::Form;

#[cfg(feature = "json")]
mod json;
#[cfg(feature = "json")]
pub use json::Json;

#[cfg(feature = "limits")]
mod limits;
#[cfg(feature = "limits")]
pub use limits::Limits;

#[cfg(feature = "multipart")]
mod multipart;
#[cfg(feature = "multipart")]
pub use multipart::{Multipart, MultipartError, MultipartLimits};

#[cfg(feature = "params")]
mod params;
#[cfg(feature = "params")]
pub(crate) use params::PathDeserializer;
#[cfg(feature = "params")]
pub use params::{Params, ParamsError};

#[cfg(feature = "query")]
mod query;
#[cfg(feature = "query")]
pub use query::Query;

#[cfg(feature = "session")]
mod session;
#[cfg(feature = "session")]
pub use session::Session;

#[cfg(feature = "sse")]
mod sse;
#[cfg(feature = "sse")]
pub use sse::{Event, Sse};

#[cfg(feature = "websocket")]
mod websocket;
#[cfg(feature = "websocket")]
pub use websocket::{Message, WebSocket, WebSocketConfig, WebSocketError, WebSocketStream};

mod header;
mod payload;

pub use header::{Header, HeaderError};
pub use payload::{Payload, PayloadError};
