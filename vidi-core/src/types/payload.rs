//! Request payload trait and error.

use crate::{Error, IntoResponse, Response, Result, StatusCode, ThisError};

/// Error returned when request body extraction fails.
#[derive(Debug, ThisError)]
pub enum PayloadError {
    /// 400
    #[error("payload is empty")]
    Empty,

    /// 400
    #[error("failed to read payload")]
    Read,

    /// 400
    #[error("failed to parse payload")]
    Parse,

    /// 400
    #[error("multipart missing boundary")]
    MissingBoundary,

    /// 400
    #[error("parse utf8 failed, {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// 400
    #[error("{0}")]
    Hyper(#[from] hyper::Error),

    #[cfg(feature = "json")]
    /// 400
    #[error("JSON serialize or deserialize failed, {0}")]
    Json(#[from] serde_json::Error),

    /// 400
    #[cfg(any(feature = "form", feature = "query"))]
    #[error("url decode failed, {0}")]
    UrlDecode(#[from] serde_urlencoded::de::Error),

    /// 411
    #[error("content-length is required")]
    LengthRequired,

    /// 413
    #[error("payload is too large")]
    TooLarge,

    /// 415
    #[error("unsupported media type, `{}` is required", .0.to_string())]
    UnsupportedMediaType(mime::Mime),

    /// 500
    #[error("payload has been used")]
    Used,
}

impl IntoResponse for PayloadError {
    fn into_response(self) -> Response {
        (
            match self {
                Self::Empty
                | Self::Read
                | Self::Parse
                | Self::MissingBoundary
                | Self::Utf8(_)
                | Self::Hyper(_) => StatusCode::BAD_REQUEST,
                #[cfg(feature = "json")]
                Self::Json(_) => StatusCode::BAD_REQUEST,
                #[cfg(any(feature = "form", feature = "query"))]
                Self::UrlDecode(_) => StatusCode::BAD_REQUEST,
                Self::LengthRequired => StatusCode::LENGTH_REQUIRED,
                Self::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
                Self::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
                Self::Used => StatusCode::INTERNAL_SERVER_ERROR,
            },
            self.to_string(),
        )
            .into_response()
    }
}

impl_from_error!(PayloadError);

/// An interface for processing the payload data of the HTTP request.
pub trait Payload {
    /// The payload name.
    const NAME: &'static str = "payload";

    /// The payload size limit; 1 MB by default.
    const LIMIT: u64 = 1024 * 1024;

    /// The payload media type.
    fn mime() -> mime::Mime;

    /// Returns whether the given media type is supported.
    fn detect(m: &mime::Mime) -> bool;

    /// Returns the effective limit.
    #[must_use]
    #[inline]
    fn limit(limit: Option<u64>) -> u64 {
        limit.unwrap_or(Self::LIMIT)
    }

    /// Checks the `Content-Type` header.
    ///
    /// # Errors
    ///
    /// Returns [`PayloadError::UnsupportedMediaType`] if the media type is unsupported.
    #[inline]
    fn check_type(m: Option<mime::Mime>) -> Result<mime::Mime, PayloadError> {
        m.filter(Self::detect)
            .ok_or_else(|| PayloadError::UnsupportedMediaType(Self::mime()))
    }

    /// Checks the `Content-Length` header.
    ///
    /// # Errors
    ///
    /// Returns [`PayloadError::TooLarge`] if the content length exceeds the limit.
    #[inline]
    fn check_length(len: Option<u64>, limit: Option<u64>) -> Result<(), PayloadError> {
        len.map_or_else(
            || Err(PayloadError::LengthRequired),
            |len| {
                (len <= Self::limit(limit))
                    .then_some(())
                    .ok_or_else(|| PayloadError::TooLarge)
            },
        )
    }

    /// Checks the `Content-Type` and `Content-Length` headers.
    ///
    /// # Errors
    ///
    /// 1. unsupported media type
    /// 2. content-length is required
    /// 3. payload is too large
    #[inline]
    fn check_header(
        m: Option<mime::Mime>,
        len: Option<u64>,
        limit: Option<u64>,
    ) -> Result<mime::Mime, PayloadError> {
        let m = Self::check_type(m)?;
        Self::check_length(len, limit)?;
        Ok(m)
    }
}
