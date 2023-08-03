#[cfg(feature = "json")]
use bytes::{BufMut, BytesMut};

use crate::{header, Body, Response, Result, StatusCode};

/// The [Response] Extension.
pub trait ResponseExt: Sized {
    /// Get the size of this response's body.
    fn content_length(&self) -> Option<u64>;

    /// The response with the specified [`Content-Type`][mdn].
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type>
    fn with<T>(t: T, c: &'static str) -> Response
    where
        T: Into<Body>,
    {
        let mut res = Response::new(t.into());
        res.headers_mut()
            .insert(header::CONTENT_TYPE, header::HeaderValue::from_static(c));
        res
    }

    /// The response with `text/plain; charset=utf-8` media type.
    fn text<T>(t: T) -> Response
    where
        T: Into<Body>,
    {
        Self::with(t, mime::TEXT_PLAIN_UTF_8.as_ref())
    }

    /// The response with `text/html; charset=utf-8` media type.
    fn html<T>(t: T) -> Response
    where
        T: Into<Body>,
    {
        Self::with(t, mime::TEXT_HTML_UTF_8.as_ref())
    }

    /// The response with `application/javascript; charset=utf-8` media type.
    ///
    /// # Errors
    ///
    /// Throws an error if serialization fails.
    #[cfg(feature = "json")]
    fn json<T>(t: T) -> Result<Response, crate::types::PayloadError>
    where
        T: serde::Serialize,
    {
        let mut buf = BytesMut::new().writer();
        serde_json::to_writer(&mut buf, &t)
            .map(|_| Self::with(buf.into_inner().freeze(), mime::APPLICATION_JSON.as_ref()))
            .map_err(crate::types::PayloadError::Json)
    }

    /// Responds to a stream.
    fn stream<S, O, E>(s: S) -> Response
    where
        S: futures_util::Stream<Item = Result<O, E>> + Send + 'static,
        O: Into<bytes::Bytes> + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        Response::new(Body::wrap_stream(s))
    }

    // TODO: Download transfers the file from path as an attachment.
    // fn download() -> Response<Body>

    /// The response was successful (status in the range [`200-299`][mdn]) or not.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/ok>
    fn ok(&self) -> bool;

    /// The [`Content-Location`][mdn] header indicates an alternate location for the returned data.
    ///
    /// [mdn]: <https://developer.mozilla.org/zh-CN/docs/Web/HTTP/Headers/Content-Location>
    fn location(location: &'static str) -> Self;

    /// The response redirects to the specified URL.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/redirect>
    fn redirect<T>(url: T) -> Response
    where
        T: AsRef<str>;

    /// The response redirects to the specified URL and the status code.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/redirect>
    fn redirect_with_status<T>(uri: T, status: StatusCode) -> Response
    where
        T: AsRef<str>;

    /// The response redirects to the [`303`][mdn].
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/303>
    fn see_other<T>(url: T) -> Response
    where
        T: AsRef<str>,
    {
        Self::redirect_with_status(url, StatusCode::SEE_OTHER)
    }

    /// The response redirects to the [`307`][mdn].
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/307>
    fn temporary<T>(url: T) -> Response
    where
        T: AsRef<str>,
    {
        Self::redirect_with_status(url, StatusCode::TEMPORARY_REDIRECT)
    }

    /// The response redirects to the [`308`][mdn].
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/308>
    fn permanent<T>(url: T) -> Response
    where
        T: AsRef<str>,
    {
        Self::redirect_with_status(url, StatusCode::PERMANENT_REDIRECT)
    }
}

impl ResponseExt for Response {
    fn content_length(&self) -> Option<u64> {
        self.headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
    }

    fn ok(&self) -> bool {
        self.status().is_success()
    }

    fn location(location: &'static str) -> Self {
        let mut res = Self::default();
        res.headers_mut().insert(
            header::CONTENT_LOCATION,
            header::HeaderValue::from_static(location),
        );
        res
    }

    fn redirect<T>(url: T) -> Response
    where
        T: AsRef<str>,
    {
        match header::HeaderValue::try_from(url.as_ref()) {
            Ok(val) => {
                let mut res = Self::default();
                res.headers_mut().insert(header::LOCATION, val);
                res
            }
            Err(err) => panic!("{}", err),
        }
    }

    fn redirect_with_status<T>(url: T, status: StatusCode) -> Response
    where
        T: AsRef<str>,
    {
        assert!(status.is_redirection(), "not a redirection status code");

        let mut res = Self::redirect(url);
        *res.status_mut() = status;
        res
    }
}
