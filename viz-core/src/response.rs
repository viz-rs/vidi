use http_body_util::Full;

use crate::{header, Body, BoxError, Bytes, Error, Future, Response, Result, StatusCode};

/// The [`Response`] Extension.
pub trait ResponseExt: private::Sealed + Sized {
    /// Get the size of this response's body.
    fn content_length(&self) -> Option<u64>;

    /// Get the media type of this response.
    fn content_type(&self) -> Option<mime::Mime>;

    /// Get a header with the key.
    fn header<K, T>(&self, key: K) -> Option<T>
    where
        K: header::AsHeaderName,
        T: std::str::FromStr;

    /// The response was successful (status in the range [`200-299`][mdn]) or not.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/ok>
    fn ok(&self) -> bool;

    /// Creates a response with an empty body.
    #[must_use]
    fn empty() -> Response {
        Response::new(Body::empty())
    }

    /// The response with the specified [`Content-Type`][mdn].
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type>
    fn with<B>(body: B, content_type: &'static str) -> Response
    where
        B: Into<Body>,
    {
        let mut resp = Response::new(body.into());
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static(content_type),
        );
        resp
    }

    /// The response with `text/plain; charset=utf-8` media type.
    fn text<B>(body: B) -> Response
    where
        B: Into<Full<Bytes>>,
    {
        Self::with(body.into(), mime::TEXT_PLAIN_UTF_8.as_ref())
    }

    /// The response with `text/html; charset=utf-8` media type.
    fn html<B>(body: B) -> Response
    where
        B: Into<Full<Bytes>>,
    {
        Self::with(body.into(), mime::TEXT_HTML_UTF_8.as_ref())
    }

    /// The response with `application/javascript; charset=utf-8` media type.
    ///
    /// # Errors
    ///
    /// Throws an error if serialization fails.
    #[cfg(feature = "json")]
    fn json<T>(body: T) -> Result<Response, crate::types::PayloadError>
    where
        T: serde::Serialize,
    {
        use bytes::{BufMut, BytesMut};

        // See <https://docs.rs/serde_json/latest/src/serde_json/ser.rs.html#2179>
        let mut buf = BytesMut::with_capacity(128).writer();
        serde_json::to_writer(&mut buf, &body)
            .map(|()| {
                Self::with(
                    Full::new(buf.into_inner().freeze()),
                    mime::APPLICATION_JSON.as_ref(),
                )
            })
            .map_err(crate::types::PayloadError::Json)
    }

    /// Responds to a stream.
    fn stream<S, D, E>(stream: S) -> Response
    where
        S: futures_util::Stream<Item = Result<D, E>> + Send + 'static,
        D: Into<Bytes> + 'static,
        E: Into<BoxError> + 'static,
    {
        Response::new(Body::from_stream(stream))
    }

    /// Downloads transfers the file from path as an attachment.
    #[cfg(feature = "fs")]
    fn download<T>(path: T, name: Option<&str>) -> impl Future<Output = Result<Self>> + Send
    where
        T: AsRef<std::path::Path> + Send;

    /// The [`Content-Disposition`][mdn] header indicates if the content is expected to be
    /// displayed inline in the browser, that is, as a Web page or as part of a Web page,
    /// or as an attachment, that is downloaded and saved locally.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Disposition>
    fn attachment(value: &str) -> Response {
        let val = header::HeaderValue::from_str(value)
            .expect("content-disposition is not the correct value");
        let mut resp = Response::default();
        resp.headers_mut().insert(header::CONTENT_DISPOSITION, val);
        resp
    }

    /// The [`Content-Location`][mdn] header indicates an alternate location for the returned data.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Location>
    fn location<T>(location: T) -> Response
    where
        T: AsRef<str>,
    {
        let val = header::HeaderValue::try_from(location.as_ref())
            .expect("location is not the correct value");
        let mut resp = Response::default();
        resp.headers_mut().insert(header::CONTENT_LOCATION, val);
        resp
    }

    /// The response redirects to the specified URL.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/redirect>
    fn redirect<T>(url: T) -> Response
    where
        T: AsRef<str>,
    {
        let val =
            header::HeaderValue::try_from(url.as_ref()).expect("url is not the correct value");
        let mut resp = Response::default();
        resp.headers_mut().insert(header::LOCATION, val);
        resp
    }

    /// The response redirects to the specified URL and the status code.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/redirect>
    fn redirect_with_status<T>(url: T, status: StatusCode) -> Response
    where
        T: AsRef<str>,
    {
        assert!(status.is_redirection(), "not a redirection status code");

        let mut resp = Self::redirect(url);
        *resp.status_mut() = status;
        resp
    }

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
            .map(header::HeaderValue::to_str)
            .and_then(Result::ok)
            .map(str::parse)
            .and_then(Result::ok)
    }

    fn content_type(&self) -> Option<mime::Mime> {
        self.header(header::CONTENT_TYPE)
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

    fn ok(&self) -> bool {
        self.status().is_success()
    }

    #[cfg(feature = "fs")]
    async fn download<T>(path: T, name: Option<&str>) -> Result<Self>
    where
        T: AsRef<std::path::Path> + Send,
    {
        let value = if let Some(filename) = name {
            filename
        } else if let Some(filename) = path.as_ref().file_name().and_then(std::ffi::OsStr::to_str) {
            filename
        } else {
            "download"
        }
        .escape_default();

        let mut resp = Self::attachment(&format!("attachment; filename=\"{value}\""));
        *resp.body_mut() = Body::from_stream(tokio_util::io::ReaderStream::new(
            tokio::fs::File::open(path).await.map_err(Error::from)?,
        ));
        Ok(resp)
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::Response {}
}
