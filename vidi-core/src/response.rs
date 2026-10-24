use http_body_util::Full;

use crate::{Body, BoxError, Bytes, Response, Result, StatusCode, header};

/// The [`Response`] Extension.
pub trait ResponseExt: private::Sealed + Sized {
    /// Creates a response with an empty body.
    #[must_use]
    fn empty() -> Response {
        Response::new(Body::empty())
    }

    /// Returns the body size.
    fn content_length(&self) -> Option<u64>;

    /// Returns the media type.
    fn content_type(&self) -> Option<mime::Mime>;

    /// Returns a header parsed from the given key.
    fn header<K, T>(&self, key: K) -> Option<T>
    where
        K: header::AsHeaderName,
        T: std::str::FromStr;

    /// Returns whether the status is successful (200-299).
    fn ok(&self) -> bool;

    /// Creates a response with the given `Content-Type`.
    fn with<B>(body: B, content_type: &str) -> Response
    where
        B: Into<Body>,
    {
        let mut resp = Response::new(body.into());
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_str(content_type).expect("Invalid content type"),
        );
        resp
    }

    /// Creates a response with `text/plain; charset=utf-8`.
    fn text<B>(body: B) -> Response
    where
        B: Into<Full<Bytes>>,
    {
        Self::with(body.into(), mime::TEXT_PLAIN_UTF_8.as_ref())
    }

    /// Creates a response with `text/html; charset=utf-8`.
    fn html<B>(body: B) -> Response
    where
        B: Into<Full<Bytes>>,
    {
        Self::with(body.into(), mime::TEXT_HTML_UTF_8.as_ref())
    }

    /// Creates a response with `application/octet-stream`.
    fn binary<B>(body: B) -> Response
    where
        B: Into<Full<Bytes>>,
    {
        Self::with(body.into(), mime::APPLICATION_OCTET_STREAM.as_ref())
    }

    /// Creates a response with `application/json`.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    #[cfg(feature = "json")]
    fn json<T>(body: T) -> Result<Response, crate::types::PayloadError>
    where
        T: serde::Serialize,
    {
        let body = serde_json::to_vec(&body).map_err(crate::types::PayloadError::Json)?;
        Ok(Response::with(
            Full::from(body),
            mime::APPLICATION_JSON.as_ref(),
        ))
    }

    /// Creates a response from a stream.
    fn stream<S, D, E>(stream: S) -> Response
    where
        S: futures_util::Stream<Item = Result<D, E>> + Send + 'static,
        D: Into<Bytes> + 'static,
        E: Into<BoxError> + 'static,
    {
        Response::new(Body::from_stream(stream))
    }

    /// Transfers the file at `path` as an attachment.
    #[cfg(feature = "fs")]
    fn download<T>(
        path: T,
        name: Option<&str>,
    ) -> impl std::future::Future<Output = Result<Self>> + Send
    where
        T: AsRef<std::path::Path> + Send;

    /// Creates a response with the given `Content-Disposition` header.
    fn attachment(value: &str) -> Response {
        let val = header::HeaderValue::from_str(value)
            .expect("content-disposition is not the correct value");
        let mut resp = Response::default();
        resp.headers_mut().insert(header::CONTENT_DISPOSITION, val);
        resp
    }

    /// Creates a response with the given `Content-Location` header.
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

    /// Creates a redirect response to the URL.
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

    /// Creates a redirect response to the URL with the given status.
    fn redirect_with_status<T>(url: T, status: StatusCode) -> Response
    where
        T: AsRef<str>,
    {
        assert!(status.is_redirection(), "not a redirection status code");

        let mut resp = Self::redirect(url);
        *resp.status_mut() = status;
        resp
    }

    /// Creates a `303 See Other` redirect.
    fn see_other<T>(url: T) -> Response
    where
        T: AsRef<str>,
    {
        Self::redirect_with_status(url, StatusCode::SEE_OTHER)
    }

    /// Creates a `307 Temporary Redirect`.
    fn temporary<T>(url: T) -> Response
    where
        T: AsRef<str>,
    {
        Self::redirect_with_status(url, StatusCode::TEMPORARY_REDIRECT)
    }

    /// Creates a `308 Permanent Redirect`.
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
        let value = name
            .unwrap_or_else(|| {
                path.as_ref()
                    .file_name()
                    .and_then(std::ffi::OsStr::to_str)
                    .map_or("download", |filename| filename)
            })
            .escape_default();

        let mut resp = Self::attachment(&format!("attachment; filename=\"{value}\""));
        *resp.body_mut() = Body::from_stream(tokio_util::io::ReaderStream::new(
            tokio::fs::File::open(path)
                .await
                .map_err(crate::Error::from)?,
        ));
        Ok(resp)
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::Response {}
}
