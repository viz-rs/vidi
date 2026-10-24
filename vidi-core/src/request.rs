use crate::{
    Body, BodyState, Bytes, FromRequest, Future, Request, Result, header,
    types::{PayloadError, RealIp},
};
use headers::HeaderMapExt;
use http_body_util::{BodyExt, Collected};

use std::sync::Arc;

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
use crate::types::Multipart;

#[cfg(feature = "cookie")]
use crate::types::{Cookie, Cookies, CookiesError};

#[cfg(feature = "session")]
use crate::types::Session;

use crate::types::RouteInfo;

#[cfg(feature = "params")]
use crate::types::{ParamsError, PathDeserializer};

/// The [`Request`] Extension.
pub trait RequestExt: private::Sealed + Sized {
    /// Returns the URL scheme.
    fn schema(&self) -> Option<&http::uri::Scheme>;

    /// Returns the URL path.
    fn path(&self) -> &str;

    /// Returns the URL query string.
    fn query_string(&self) -> Option<&str>;

    /// Deserializes the query string.
    ///
    /// # Errors
    ///
    /// Returns [`PayloadError::UrlDecode`] if decoding the query string fails.
    #[cfg(feature = "query")]
    fn query<T>(&self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned;

    /// Returns a header parsed from the given key.
    fn header<K, T>(&self, key: K) -> Option<T>
    where
        K: header::AsHeaderName,
        T: std::str::FromStr;

    /// Returns a typed header.
    fn header_typed<H>(&self) -> Option<H>
    where
        H: headers::Header;

    /// Returns the body size.
    fn content_length(&self) -> Option<u64>;

    /// Returns the media type.
    fn content_type(&self) -> Option<mime::Mime>;

    /// Extracts a value of type `T` from the request.
    fn extract<T>(&mut self) -> impl Future<Output = Result<T, T::Error>> + Send
    where
        T: FromRequest;

    /// Takes the request body.
    ///
    /// # Errors
    ///
    /// Returns [`PayloadError::Empty`] or [`PayloadError::Used`] if the body is empty or has
    /// already been read.
    fn incoming(&mut self) -> Result<Body, PayloadError>;

    /// Returns the request body as bytes.
    fn bytes(&mut self) -> impl Future<Output = Result<Bytes, PayloadError>> + Send;

    /// Returns the request body as text.
    fn text(&mut self) -> impl Future<Output = Result<String, PayloadError>> + Send;

    /// Deserializes the form request body.
    #[cfg(feature = "form")]
    fn form<T>(&mut self) -> impl Future<Output = Result<T, PayloadError>> + Send
    where
        T: serde::de::DeserializeOwned;

    /// Deserializes the JSON request body.
    #[cfg(feature = "json")]
    fn json<T>(&mut self) -> impl Future<Output = Result<T, PayloadError>> + Send
    where
        T: serde::de::DeserializeOwned;

    /// Extracts the multipart request body.
    #[cfg(feature = "multipart")]
    fn multipart(&mut self) -> impl Future<Output = Result<Multipart, PayloadError>> + Send;

    /// Returns shared state of type `T`.
    #[cfg(feature = "state")]
    fn state<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static;

    /// Stores shared state of type `T`.
    #[cfg(feature = "state")]
    fn set_state<T>(&mut self, t: T) -> Option<T>
    where
        T: Clone + Send + Sync + 'static;

    /// Returns the cookie jar.
    ///
    /// # Errors
    ///
    /// Returns [`CookiesError`] if reading cookies fails.
    #[cfg(feature = "cookie")]
    fn cookies(&self) -> Result<Cookies, CookiesError>;

    /// Returns a cookie by name.
    #[cfg(feature = "cookie")]
    fn cookie<S>(&self, name: S) -> Option<Cookie<'_>>
    where
        S: AsRef<str>;

    /// Returns the current session.
    #[cfg(feature = "session")]
    fn session(&self) -> &Session;

    /// Deserializes the route params.
    ///
    /// # Errors
    ///
    /// Returns [`ParamsError`] if deserializing the params fails.
    #[cfg(feature = "params")]
    fn params<T>(&self) -> Result<T, ParamsError>
    where
        T: serde::de::DeserializeOwned;

    /// Parses a route param by name.
    ///
    /// # Errors
    ///
    /// Returns [`ParamsError`] if parsing the param fails.
    #[cfg(feature = "params")]
    fn param<T>(&self, name: &str) -> Result<T, ParamsError>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display;

    /// Returns the current route.
    fn route_info(&self) -> &Arc<RouteInfo>;

    /// Returns the remote address.
    fn remote_addr(&self) -> Option<&std::net::SocketAddr>;

    /// Returns the resolved real IP.
    fn realip(&self) -> Option<RealIp>;
}

impl RequestExt for Request {
    #[inline]
    fn schema(&self) -> Option<&http::uri::Scheme> {
        self.uri().scheme()
    }

    #[inline]
    fn path(&self) -> &str {
        self.uri().path()
    }

    #[inline]
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

    #[inline]
    fn header<K, T>(&self, key: K) -> Option<T>
    where
        K: header::AsHeaderName,
        T: std::str::FromStr,
    {
        self.headers()
            .get(key)
            .map(header::HeaderValue::to_str)
            .and_then(Result::ok)
            .map(str::parse)
            .and_then(Result::ok)
    }

    #[inline]
    fn header_typed<H>(&self) -> Option<H>
    where
        H: headers::Header,
    {
        self.headers().typed_get()
    }

    #[inline]
    fn content_length(&self) -> Option<u64> {
        self.header(header::CONTENT_LENGTH)
    }

    #[inline]
    fn content_type(&self) -> Option<mime::Mime> {
        self.header(header::CONTENT_TYPE)
    }

    async fn extract<T>(&mut self) -> Result<T, T::Error>
    where
        T: FromRequest,
    {
        T::extract(self).await
    }

    fn incoming(&mut self) -> Result<Body, PayloadError> {
        if let Some(state) = self.extensions().get::<BodyState>() {
            match state {
                BodyState::Empty => Err(PayloadError::Empty)?,
                BodyState::Used => Err(PayloadError::Used)?,
                BodyState::Normal => {}
            }
        }

        let (state, result) = match std::mem::replace(self.body_mut(), Body::Empty) {
            Body::Empty => (BodyState::Empty, Err(PayloadError::Empty)),
            body => (BodyState::Used, Ok(body)),
        };

        self.extensions_mut().insert(state);
        result
    }

    async fn bytes(&mut self) -> Result<Bytes, PayloadError> {
        self.incoming()?
            .collect()
            .await
            .map_err(|err| {
                #[cfg(feature = "limits")]
                if err.is::<LengthLimitError>() {
                    return PayloadError::TooLarge;
                }
                if let Ok(err) = err.downcast::<hyper::Error>() {
                    return PayloadError::Hyper(err);
                }
                PayloadError::Read
            })
            .map(Collected::to_bytes)
    }

    async fn text(&mut self) -> Result<String, PayloadError> {
        let bytes = self.bytes().await?;
        String::from_utf8(bytes.to_vec()).map_err(PayloadError::Utf8)
    }

    #[cfg(feature = "form")]
    async fn form<T>(&mut self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned,
    {
        <Form as Payload>::check_type(self.content_type())?;
        let bytes = self.bytes().await?;
        serde_urlencoded::from_reader(bytes::Buf::reader(bytes)).map_err(PayloadError::UrlDecode)
    }

    #[cfg(feature = "json")]
    async fn json<T>(&mut self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned,
    {
        <Json as Payload>::check_type(self.content_type())?;
        let bytes = self.bytes().await?;
        serde_json::from_slice(&bytes).map_err(PayloadError::Json)
    }

    #[cfg(feature = "multipart")]
    #[allow(clippy::unused_async_trait_impl)]
    async fn multipart(&mut self) -> Result<Multipart, PayloadError> {
        let m = <Multipart as Payload>::check_type(self.content_type())?;

        let boundary = m
            .get_param(mime::BOUNDARY)
            .ok_or(PayloadError::MissingBoundary)?
            .as_str();

        Ok(Multipart::new(self.incoming()?, boundary))
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
        self.route_info()
            .params
            .iter()
            .find(|p| p.0 == name)
            .ok_or_else(|| ParamsError::SingleParse(format!("missing {name} param")))?
            .1
            .parse()
            .map_err(|e: T::Err| ParamsError::SingleParse(e.to_string()))
    }

    #[inline]
    fn remote_addr(&self) -> Option<&std::net::SocketAddr> {
        self.extensions().get()
    }

    #[inline]
    fn route_info(&self) -> &Arc<RouteInfo> {
        self.extensions().get().expect("should get current route")
    }

    #[inline]
    fn realip(&self) -> Option<RealIp> {
        RealIp::parse(self)
    }
}

/// Extension for requests with body limits.
#[cfg(feature = "limits")]
pub trait RequestLimitsExt: private::Sealed + Sized {
    /// Returns the limits.
    fn limits(&self) -> &Limits;

    /// Returns the request body as bytes, limited.
    fn bytes_with(
        &mut self,
        limit: Option<u64>,
        max: u64,
    ) -> impl Future<Output = Result<Bytes, PayloadError>> + Send;

    /// Returns the request body as text, limited.
    fn text_with_limit(&mut self) -> impl Future<Output = Result<String, PayloadError>> + Send;

    /// Deserializes the form request body, limited.
    #[cfg(feature = "form")]
    fn form_with_limit<T>(&mut self) -> impl Future<Output = Result<T, PayloadError>> + Send
    where
        T: serde::de::DeserializeOwned;

    /// Deserializes the JSON request body, limited.
    #[cfg(feature = "json")]
    fn json_with_limit<T>(&mut self) -> impl Future<Output = Result<T, PayloadError>> + Send
    where
        T: serde::de::DeserializeOwned;

    /// Extracts the multipart request body, limited.
    #[cfg(feature = "multipart")]
    fn multipart_with_limit(
        &mut self,
    ) -> impl Future<Output = Result<Multipart, PayloadError>> + Send;
}

#[cfg(feature = "limits")]
impl RequestLimitsExt for Request {
    #[inline]
    fn limits(&self) -> &Limits {
        self.extensions()
            .get::<Limits>()
            .expect("Limits middleware is required")
    }

    async fn bytes_with(&mut self, limit: Option<u64>, max: u64) -> Result<Bytes, PayloadError> {
        Limited::new(
            self.incoming()?,
            usize::try_from(limit.unwrap_or(max)).unwrap_or(usize::MAX),
        )
        .collect()
        .await
        .map_err(|err| {
            if err.is::<LengthLimitError>() {
                return PayloadError::TooLarge;
            }
            if let Ok(err) = err.downcast::<hyper::Error>() {
                return PayloadError::Hyper(*err);
            }
            PayloadError::Read
        })
        .map(Collected::to_bytes)
    }

    async fn text_with_limit(&mut self) -> Result<String, PayloadError> {
        let bytes = self
            .bytes_with(self.limits().get("text"), Limits::NORMAL)
            .await?;
        String::from_utf8(bytes.to_vec()).map_err(PayloadError::Utf8)
    }

    #[cfg(feature = "form")]
    async fn form_with_limit<T>(&mut self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned,
    {
        let limit = self.limits().get(<Form as Payload>::NAME);
        <Form as Payload>::check_header(self.content_type(), self.content_length(), limit)?;
        let bytes = self.bytes_with(limit, <Form as Payload>::LIMIT).await?;
        serde_urlencoded::from_reader(bytes::Buf::reader(bytes)).map_err(PayloadError::UrlDecode)
    }

    #[cfg(feature = "json")]
    async fn json_with_limit<T>(&mut self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned,
    {
        let limit = self.limits().get(<Json as Payload>::NAME);
        <Json as Payload>::check_header(self.content_type(), self.content_length(), limit)?;
        let bytes = self.bytes_with(limit, <Json as Payload>::LIMIT).await?;
        serde_json::from_slice(&bytes).map_err(PayloadError::Json)
    }

    #[cfg(feature = "multipart")]
    #[allow(clippy::unused_async_trait_impl)]
    async fn multipart_with_limit(&mut self) -> Result<Multipart, PayloadError> {
        let limit = self.limits().get(<Multipart as Payload>::NAME);
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
            self.incoming()?,
            boundary,
            self.extensions()
                .get::<std::sync::Arc<crate::types::MultipartLimits>>()
                .map(AsRef::as_ref)
                .cloned()
                .unwrap_or_default(),
        ))
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::Request {}
}
