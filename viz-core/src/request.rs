use std::{mem::replace, sync::Arc};

use crate::{
    async_trait, header,
    types::{PayloadError, RealIp, RouteInfo},
    Bytes, FromRequest, Incoming, IncomingBody, Request, Result,
};
use headers::HeaderMapExt;
use http_body_util::{BodyExt, Collected};

#[cfg(feature = "limits")]
use crate::types::Limits;
#[cfg(feature = "limits")]
use http_body_util::{LengthLimitError, Limited};

#[cfg(any(feature = "form", feature = "json", feature = "multipart"))]
use crate::types::Payload;

#[cfg(feature = "form")]
use crate::types::Form;

#[cfg(feature = "json")]
use crate::types::Json;

#[cfg(feature = "multipart")]
use crate::types::{Multipart, MultipartLimits};

#[cfg(feature = "cookie")]
use crate::types::{Cookie, Cookies, CookiesError};

#[cfg(feature = "session")]
use crate::types::Session;

#[cfg(feature = "params")]
use crate::types::{ParamsError, PathDeserializer};

/// The [Request] Extension.
#[async_trait]
pub trait RequestExt: Sized {
    /// Get URL's schema of this request.
    fn schema(&self) -> Option<&http::uri::Scheme>;

    /// Get URL's path of this request.
    fn path(&self) -> &str;

    /// Get URL's query string of this request.
    fn query_string(&self) -> Option<&str>;

    /// Get query data by type.
    #[cfg(feature = "query")]
    fn query<T>(&self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned;

    /// Get a header with the key.
    fn header<K, T>(&self, key: K) -> Option<T>
    where
        K: header::AsHeaderName,
        T: std::str::FromStr;

    /// Get a header with the specified type.
    fn header_typed<H>(&self) -> Option<H>
    where
        H: headers::Header;

    /// Get the size of this request's body.
    fn content_length(&self) -> Option<u64>;

    /// Get the media type of this request.
    fn content_type(&self) -> Option<mime::Mime>;

    /// Extract the data from this request by the specified type.
    async fn extract<T>(&mut self) -> Result<T, T::Error>
    where
        T: FromRequest;

    /// Get Incoming body.
    fn incoming_body(&mut self) -> IncomingBody;

    /// Get Incoming.
    fn incoming(&mut self) -> Result<Incoming, PayloadError>;

    /// Return with a [Bytes][mdn] representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/arrayBuffer>
    async fn bytes(&mut self) -> Result<Bytes, PayloadError>;

    #[cfg(feature = "limits")]
    /// Return with a [Bytes][mdn]  by a limit representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/arrayBuffer>
    async fn bytes_with(&mut self, limit: Option<u64>, max: u64) -> Result<Bytes, PayloadError>;

    /// Return with a [Text][mdn] representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/text>
    async fn text(&mut self) -> Result<String, PayloadError>;

    #[cfg(feature = "form")]
    /// Return with a `application/x-www-form-urlencoded` [FormData][mdn] by the specified type
    /// representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/FormData>
    async fn form<T>(&mut self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned;

    #[cfg(feature = "json")]
    /// Return with a [JSON][mdn] by the specified type representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/json>
    async fn json<T>(&mut self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned;

    #[cfg(feature = "multipart")]
    /// Return with a `multipart/form-data` [FormData][mdn] by the specified type
    /// representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/FormData>
    async fn multipart(&mut self) -> Result<Multipart, PayloadError>;

    #[cfg(feature = "state")]
    /// Return a shared state by the specified type.
    fn state<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static;

    #[cfg(feature = "state")]
    /// Store a shared state.
    fn set_state<T>(&mut self, t: T) -> Option<T>
    where
        T: Clone + Send + Sync + 'static;

    #[cfg(feature = "cookie")]
    /// Get a wrapper of `cookie-jar` for managing cookies.
    fn cookies(&self) -> Result<Cookies, CookiesError>;

    #[cfg(feature = "cookie")]
    /// Get a cookie by the specified name.
    fn cookie<S>(&self, name: S) -> Option<Cookie<'_>>
    where
        S: AsRef<str>;

    #[cfg(feature = "limits")]
    /// Get limits settings.
    fn limits(&self) -> &Limits;

    #[cfg(feature = "session")]
    /// Get current session.
    fn session(&self) -> &Session;

    #[cfg(feature = "params")]
    /// Get all parameters.
    fn params<T>(&self) -> Result<T, ParamsError>
    where
        T: serde::de::DeserializeOwned;

    #[cfg(feature = "params")]
    /// Get single parameter by name.
    fn param<T>(&self, name: &str) -> Result<T, ParamsError>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display;

    /// Get current route.
    fn route_info(&self) -> &Arc<RouteInfo>;

    /// Get remote addr.
    fn remote_addr(&self) -> Option<&std::net::SocketAddr>;

    /// Get realip.
    fn realip(&self) -> Option<RealIp>;
}

#[async_trait]
impl RequestExt for Request {
    fn schema(&self) -> Option<&http::uri::Scheme> {
        self.uri().scheme()
    }

    fn path(&self) -> &str {
        self.uri().path()
    }

    fn query_string(&self) -> Option<&str> {
        self.uri().query()
    }

    #[cfg(feature = "query")]
    fn query<T>(&self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_urlencoded::from_str(self.query_string().unwrap_or_default())
            .map_err(PayloadError::UrlDecode)
    }

    fn header<K, T>(&self, key: K) -> Option<T>
    where
        K: header::AsHeaderName,
        T: std::str::FromStr,
    {
        self.headers()
            .get(key)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<T>().ok())
    }

    fn header_typed<H>(&self) -> Option<H>
    where
        H: headers::Header,
    {
        self.headers().typed_get()
    }

    fn content_length(&self) -> Option<u64> {
        self.header(header::CONTENT_LENGTH)
    }

    fn content_type(&self) -> Option<mime::Mime> {
        self.header(header::CONTENT_TYPE)
    }

    async fn extract<T>(&mut self) -> Result<T, T::Error>
    where
        T: FromRequest,
    {
        T::extract(self).await
    }

    fn incoming_body(&mut self) -> IncomingBody {
        replace(self.body_mut(), IncomingBody::used())
    }

    fn incoming(&mut self) -> Result<Incoming, PayloadError> {
        match self.incoming_body() {
            IncomingBody::Empty => Err(PayloadError::Empty),
            IncomingBody::Incoming(None) => Err(PayloadError::Used),
            IncomingBody::Incoming(Some(incoming)) => Ok(incoming),
        }
    }

    async fn bytes(&mut self) -> Result<Bytes, PayloadError> {
        // self.body_mut()
        self.incoming()?
            .collect()
            .await
            .map_err(|_| PayloadError::Read)
            .map(Collected::to_bytes)
    }

    #[cfg(feature = "limits")]
    async fn bytes_with(&mut self, limit: Option<u64>, max: u64) -> Result<Bytes, PayloadError> {
        Limited::new(
            self.incoming()?,
            usize::try_from(limit.unwrap_or(max)).unwrap_or(usize::MAX),
        )
        .collect()
        .await
        .map_err(|err| {
            if err.downcast_ref::<LengthLimitError>().is_some() {
                return PayloadError::TooLarge;
            }
            if let Ok(err) = err.downcast::<hyper::Error>() {
                return PayloadError::Hyper(*err);
            }
            PayloadError::Read
        })
        .map(Collected::to_bytes)
    }

    async fn text(&mut self) -> Result<String, PayloadError> {
        #[cfg(feature = "limits")]
        let bytes = self
            .bytes_with(self.limits().get("text"), Limits::NORMAL)
            .await?;
        #[cfg(not(feature = "limits"))]
        let bytes = self.bytes().await?;

        String::from_utf8(bytes.to_vec()).map_err(PayloadError::Utf8)
    }

    #[cfg(feature = "form")]
    async fn form<T>(&mut self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned,
    {
        #[cfg(feature = "limits")]
        let limit = self.limits().get(<Form as Payload>::NAME);
        #[cfg(not(feature = "limits"))]
        let limit = None;

        <Form as Payload>::check_header(self.content_type(), self.content_length(), limit)?;

        #[cfg(feature = "limits")]
        let bytes = self.bytes_with(limit, <Form as Payload>::LIMIT).await?;
        #[cfg(not(feature = "limits"))]
        let bytes = self.bytes().await?;

        serde_urlencoded::from_reader(bytes::Buf::reader(bytes)).map_err(PayloadError::UrlDecode)
    }

    #[cfg(feature = "json")]
    async fn json<T>(&mut self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned,
    {
        #[cfg(feature = "limits")]
        let limit = self.limits().get(<Json as Payload>::NAME);
        #[cfg(not(feature = "limits"))]
        let limit = None;

        <Json as Payload>::check_header(self.content_type(), self.content_length(), limit)?;

        #[cfg(feature = "limits")]
        let bytes = self.bytes_with(limit, <Json as Payload>::LIMIT).await?;
        #[cfg(not(feature = "limits"))]
        let bytes = self.bytes().await?;

        serde_json::from_slice(&bytes).map_err(PayloadError::Json)
    }

    #[cfg(feature = "multipart")]
    async fn multipart(&mut self) -> Result<Multipart, PayloadError> {
        #[cfg(feature = "limits")]
        let limit = self.limits().get(<Multipart as Payload>::NAME);
        #[cfg(not(feature = "limits"))]
        let limit = None;

        let m = <Multipart as Payload>::check_header(
            self.content_type(),
            self.content_length(),
            limit,
        )?;

        let boundary = m
            .get_param(mime::BOUNDARY)
            .ok_or(PayloadError::MissingBoundary)?
            .as_str();

        Ok(Multipart::with_limits(
            self.incoming_body(),
            boundary,
            self.extensions()
                .get::<std::sync::Arc<MultipartLimits>>()
                .map(|ml| ml.as_ref().clone())
                .unwrap_or_default(),
        ))
    }

    #[cfg(feature = "state")]
    fn state<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.extensions().get().cloned()
    }

    #[cfg(feature = "state")]
    fn set_state<T>(&mut self, t: T) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.extensions_mut().insert(t)
    }

    #[cfg(feature = "cookie")]
    fn cookies(&self) -> Result<Cookies, CookiesError> {
        self.extensions()
            .get::<Cookies>()
            .cloned()
            .ok_or(CookiesError::Read)
    }

    #[cfg(feature = "cookie")]
    fn cookie<S>(&self, name: S) -> Option<Cookie<'_>>
    where
        S: AsRef<str>,
    {
        self.extensions().get::<Cookies>()?.get(name.as_ref())
    }

    #[cfg(feature = "limits")]
    fn limits(&self) -> &Limits {
        self.extensions()
            .get::<Limits>()
            .expect("Limits middleware is required")
    }

    #[cfg(feature = "session")]
    fn session(&self) -> &Session {
        self.extensions().get().expect("should get a session")
    }

    #[cfg(feature = "params")]
    fn params<T>(&self) -> Result<T, ParamsError>
    where
        T: serde::de::DeserializeOwned,
    {
        T::deserialize(PathDeserializer::new(&self.route_info().params)).map_err(ParamsError::Parse)
    }

    #[cfg(feature = "params")]
    fn param<T>(&self, name: &str) -> Result<T, ParamsError>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        self.route_info().params.find(name)
    }

    fn remote_addr(&self) -> Option<&std::net::SocketAddr> {
        self.extensions().get()
    }

    fn route_info(&self) -> &Arc<RouteInfo> {
        self.extensions().get().expect("should get current route")
    }

    fn realip(&self) -> Option<RealIp> {
        RealIp::parse(self)
    }
}
