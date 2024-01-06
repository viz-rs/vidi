use crate::{
    header,
    types::{PayloadError, RealIp},
    Body, BodyState, Bytes, FromRequest, Future, Request, Result,
};
use headers::HeaderMapExt;
use http_body_util::{BodyExt, Collected};

#[cfg(any(feature = "params", feature = "multipart"))]
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

#[cfg(feature = "params")]
use crate::types::{ParamsError, PathDeserializer, RouteInfo};

/// The [`Request`] Extension.
pub trait RequestExt: private::Sealed + Sized {
    /// Get URL's schema of this request.
    fn schema(&self) -> Option<&http::uri::Scheme>;

    /// Get URL's path of this request.
    fn path(&self) -> &str;

    /// Get URL's query string of this request.
    fn query_string(&self) -> Option<&str>;

    /// Get query data by type.
    ///
    /// # Errors
    ///
    /// Will return [`PayloadError::UrlDecode`] if decoding the query string fails.
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
    fn extract<T>(&mut self) -> impl Future<Output = Result<T, T::Error>> + Send
    where
        T: FromRequest;

    /// Get an incoming body.
    ///
    /// # Errors
    ///
    /// Will return [`PayloadError::Empty`] or [`PayloadError::Used`] if the incoming does not
    /// exist or be used.
    fn incoming(&mut self) -> Result<Body, PayloadError>;

    /// Return with a [Bytes][mdn] representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/arrayBuffer>
    fn bytes(&mut self) -> impl Future<Output = Result<Bytes, PayloadError>> + Send;

    /// Return with a [Text][mdn] representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/text>
    fn text(&mut self) -> impl Future<Output = Result<String, PayloadError>> + Send;

    /// Return with a `application/x-www-form-urlencoded` [FormData][mdn] by the specified type
    /// representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/FormData>
    #[cfg(feature = "form")]
    fn form<T>(&mut self) -> impl Future<Output = Result<T, PayloadError>> + Send
    where
        T: serde::de::DeserializeOwned;

    /// Return with a [JSON][mdn] by the specified type representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/json>
    #[cfg(feature = "json")]
    fn json<T>(&mut self) -> impl Future<Output = Result<T, PayloadError>> + Send
    where
        T: serde::de::DeserializeOwned;

    /// Return with a `multipart/form-data` [FormData][mdn] by the specified type
    /// representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/FormData>
    #[cfg(feature = "multipart")]
    fn multipart(&mut self) -> impl Future<Output = Result<Multipart, PayloadError>> + Send;

    /// Return a shared state by the specified type.
    #[cfg(feature = "state")]
    fn state<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static;

    /// Store a shared state.
    #[cfg(feature = "state")]
    fn set_state<T>(&mut self, t: T) -> Option<T>
    where
        T: Clone + Send + Sync + 'static;

    /// Get a wrapper of `cookie-jar` for managing cookies.
    ///
    /// # Errors
    ///
    /// Will return [`CookiesError`] if getting cookies fails.
    #[cfg(feature = "cookie")]
    fn cookies(&self) -> Result<Cookies, CookiesError>;

    /// Get a cookie by the specified name.
    #[cfg(feature = "cookie")]
    fn cookie<S>(&self, name: S) -> Option<Cookie<'_>>
    where
        S: AsRef<str>;

    /// Get current session.
    #[cfg(feature = "session")]
    fn session(&self) -> &Session;

    /// Get all parameters.
    ///
    /// # Errors
    ///
    /// Will return [`ParamsError`] if deserializer the parameters fails.
    #[cfg(feature = "params")]
    fn params<T>(&self) -> Result<T, ParamsError>
    where
        T: serde::de::DeserializeOwned;

    /// Get single parameter by name.
    ///
    /// # Errors
    ///
    /// Will return [`ParamsError`] if deserializer the single parameter fails.
    #[cfg(feature = "params")]
    fn param<T>(&self, name: &str) -> Result<T, ParamsError>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display;

    /// Get current route.
    #[cfg(feature = "params")]
    fn route_info(&self) -> &Arc<RouteInfo>;

    /// Get remote addr.
    fn remote_addr(&self) -> Option<&std::net::SocketAddr>;

    /// Get realip.
    fn realip(&self) -> Option<RealIp>;
}

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
            .map(header::HeaderValue::to_str)
            .and_then(Result::ok)
            .map(str::parse)
            .and_then(Result::ok)
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
        self.route_info().params.find(name)
    }

    fn remote_addr(&self) -> Option<&std::net::SocketAddr> {
        self.extensions().get()
    }

    #[cfg(feature = "params")]
    fn route_info(&self) -> &Arc<RouteInfo> {
        self.extensions().get().expect("should get current route")
    }

    fn realip(&self) -> Option<RealIp> {
        RealIp::parse(self)
    }
}

/// The [`Request`] Extension with a limited body.
#[cfg(feature = "limits")]
pub trait RequestLimitsExt: private::Sealed + Sized {
    /// Get limits settings.
    fn limits(&self) -> &Limits;

    /// Return with a [Bytes][mdn] by a limit representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/arrayBuffer>
    fn bytes_with(
        &mut self,
        limit: Option<u64>,
        max: u64,
    ) -> impl Future<Output = Result<Bytes, PayloadError>> + Send;

    /// Return with a limited [Text][mdn] representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/text>
    fn text_with_limit(&mut self) -> impl Future<Output = Result<String, PayloadError>> + Send;

    /// Return with a limited `application/x-www-form-urlencoded` [FormData][mdn] by the specified type
    /// representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/FormData>
    #[cfg(feature = "form")]
    fn form_with_limit<T>(&mut self) -> impl Future<Output = Result<T, PayloadError>> + Send
    where
        T: serde::de::DeserializeOwned;

    /// Return with a limited [JSON][mdn] by the specified type representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/json>
    #[cfg(feature = "json")]
    fn json_with_limit<T>(&mut self) -> impl Future<Output = Result<T, PayloadError>> + Send
    where
        T: serde::de::DeserializeOwned;

    /// Return with a limited `multipart/form-data` [FormData][mdn] by the specified type
    /// representation of the request body.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/FormData>
    #[cfg(feature = "multipart")]
    fn multipart_with_limit(
        &mut self,
    ) -> impl Future<Output = Result<Multipart, PayloadError>> + Send;
}

#[cfg(feature = "limits")]
impl RequestLimitsExt for Request {
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
