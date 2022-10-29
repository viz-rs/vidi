use http_body_util::Full;

use crate::{header::CONTENT_TYPE, Error, Response, Result, StatusCode};

/// Trait implemented by types that can be converted to an HTTP [`Response`].
pub trait IntoResponse: Sized {
    /// Convert self to HTTP [`Response`].
    #[must_use]
    fn into_response(self) -> Response;

    /// Convert self to the [`Error`].
    fn into_error(self) -> Error {
        Error::Responder(self.into_response())
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Normal(error) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::from(error.to_string()).into())
                .unwrap(),
            Error::Responder(resp) => resp,
            Error::Report(_, resp) => resp,
        }
    }
}

impl IntoResponse for std::io::Error {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Full::from(self.to_string()).into())
            .unwrap()
    }
}

impl IntoResponse for std::convert::Infallible {
    fn into_response(self) -> Response {
        Response::new(().into())
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response::builder()
            .header(CONTENT_TYPE, mime::TEXT_PLAIN_UTF_8.as_ref())
            .body(Full::from(self).into())
            .unwrap()
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Response::builder()
            .header(CONTENT_TYPE, mime::TEXT_PLAIN_UTF_8.as_ref())
            .body(Full::from(self).into())
            .unwrap()
    }
}

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> Response {
        Response::builder()
            .header(CONTENT_TYPE, mime::APPLICATION_OCTET_STREAM.as_ref())
            .body(Full::from(self).into())
            .unwrap()
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        Response::builder()
            .header(CONTENT_TYPE, mime::APPLICATION_OCTET_STREAM.as_ref())
            .body(Full::from(self).into())
            .unwrap()
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
        match self {
            Some(r) => r.into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        }
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
        let mut res = self.1.into_response();
        *res.status_mut() = self.0;
        res
    }
}
