use std::borrow::Cow;

use http::header::CONTENT_LENGTH;

use crate::{Body, Error, Response, ResponseExt, Result, StatusCode};

/// Trait implemented by types that can be converted to an HTTP [`Response`].
pub trait IntoResponse: Sized {
    /// Convert self to HTTP [`Response`].
    #[must_use]
    fn into_response(self) -> Response;

    /// Convert self to the [`Error`].
    fn into_error(self) -> Error {
        Error::Responder(Box::new(self.into_response()))
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for Body {
    fn into_response(self) -> Response {
        Response::new(self)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::Boxed(error) => {
                let mut resp = error.to_string().into_response();
                *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                resp
            }
            Self::Responder(resp) | Self::Report(_, resp) => *resp,
        }
    }
}

impl IntoResponse for std::io::Error {
    fn into_response(self) -> Response {
        let mut resp = self.to_string().into_response();
        *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        resp
    }
}

impl IntoResponse for std::convert::Infallible {
    fn into_response(self) -> Response {
        Response::new(().into())
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        let size = self.len();
        let mut resp = Response::text(self);
        resp.headers_mut().insert(CONTENT_LENGTH, size.into());
        resp
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        let size = self.len();
        let mut resp = Response::text(self);
        resp.headers_mut().insert(CONTENT_LENGTH, size.into());
        resp
    }
}

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> Response {
        bytes::Bytes::from(self).into_response()
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        bytes::Bytes::from(self).into_response()
    }
}

impl<B> IntoResponse for Cow<'static, B>
where
    bytes::Bytes: From<&'static B> + From<B::Owned>,
    B: ToOwned + ?Sized,
{
    fn into_response(self) -> Response {
        match self {
            Cow::Borrowed(b) => bytes::Bytes::from(b),
            Cow::Owned(o) => bytes::Bytes::from(o),
        }
        .into_response()
    }
}

impl IntoResponse for bytes::Bytes {
    fn into_response(self) -> Response {
        let size = self.len();
        let mut resp = Response::binary(self);
        resp.headers_mut().insert(CONTENT_LENGTH, size.into());
        resp
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        Response::builder().status(self).body(().into()).unwrap()
    }
}

impl<T> IntoResponse for Option<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        self.map_or_else(
            || StatusCode::NOT_FOUND.into_response(),
            IntoResponse::into_response,
        )
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Ok(r) => r.into_response(),
            Err(e) => e.into_response(),
        }
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::new(self.into())
    }
}

impl<T> IntoResponse for (StatusCode, T)
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        let mut resp = self.1.into_response();
        *resp.status_mut() = self.0;
        resp
    }
}
