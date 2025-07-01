//! Compression Middleware.

use std::str::FromStr;

use async_compression::tokio::bufread;
use tokio_util::io::{ReaderStream, StreamReader};

use crate::{
    Body, Handler, IntoResponse, Request, Response, Result, Transform,
    header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_LENGTH, HeaderValue},
};

/// Compress response body.
#[derive(Debug)]
pub struct Config;

impl<H> Transform<H> for Config
where
    H: Clone,
{
    type Output = CompressionMiddleware<H>;

    fn transform(&self, h: H) -> Self::Output {
        CompressionMiddleware { h }
    }
}

/// Compression middleware.
#[derive(Clone, Debug)]
pub struct CompressionMiddleware<H> {
    h: H,
}

#[crate::async_trait]
impl<H, O> Handler<Request> for CompressionMiddleware<H>
where
    H: Handler<Request, Output = Result<O>>,
    O: IntoResponse,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        let accept_encoding = req
            .headers()
            .get(ACCEPT_ENCODING)
            .map(HeaderValue::to_str)
            .and_then(Result::ok)
            .and_then(parse_accept_encoding);

        let raw = self.h.call(req).await?;

        Ok(match accept_encoding {
            Some(algo) => Compress::new(raw, algo).into_response(),
            None => raw.into_response(),
        })
    }
}

/// Compresses the response body with the specified algorithm
/// and sets the `Content-Encoding` header.
#[derive(Debug)]
pub struct Compress<T> {
    inner: T,
    algo: ContentCoding,
}

impl<T> Compress<T> {
    /// Creates a compressed response with the specified algorithm.
    pub const fn new(inner: T, algo: ContentCoding) -> Self {
        Self { inner, algo }
    }
}

impl<T: IntoResponse> IntoResponse for Compress<T> {
    fn into_response(self) -> Response {
        let mut res = self.inner.into_response();

        match self.algo {
            ContentCoding::Gzip | ContentCoding::Deflate | ContentCoding::Brotli => {
                res = res.map(|body| {
                    let body = StreamReader::new(body);
                    if self.algo == ContentCoding::Gzip {
                        Body::from_stream(ReaderStream::new(bufread::GzipEncoder::new(body)))
                    } else if self.algo == ContentCoding::Deflate {
                        Body::from_stream(ReaderStream::new(bufread::DeflateEncoder::new(body)))
                    } else {
                        Body::from_stream(ReaderStream::new(bufread::BrotliEncoder::new(body)))
                    }
                });
                res.headers_mut()
                    .append(CONTENT_ENCODING, HeaderValue::from_static(self.algo.into()));
                res.headers_mut().remove(CONTENT_LENGTH);
                res
            }
            ContentCoding::Any => res,
        }
    }
}

/// [`ContentCoding`]
///
/// [`ContentCoding`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding
#[derive(Debug, Eq, PartialEq)]
pub enum ContentCoding {
    /// gzip
    Gzip,
    /// deflate
    Deflate,
    /// brotli
    Brotli,
    /// *
    Any,
}

impl FromStr for ContentCoding {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("deflate") {
            Ok(Self::Deflate)
        } else if s.eq_ignore_ascii_case("gzip") {
            Ok(Self::Gzip)
        } else if s.eq_ignore_ascii_case("br") {
            Ok(Self::Brotli)
        } else if s == "*" {
            Ok(Self::Any)
        } else {
            Err(())
        }
    }
}

impl From<ContentCoding> for &'static str {
    fn from(cc: ContentCoding) -> Self {
        match cc {
            ContentCoding::Gzip => "gzip",
            ContentCoding::Deflate => "deflate",
            ContentCoding::Brotli => "br",
            ContentCoding::Any => "*",
        }
    }
}

#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
fn parse_accept_encoding(s: &str) -> Option<ContentCoding> {
    s.split(',')
        .map(str::trim)
        .filter_map(|v| match v.split_once(";q=") {
            None => v.parse::<ContentCoding>().ok().map(|c| (c, 100)),
            Some((c, q)) => Some((
                c.parse::<ContentCoding>().ok()?,
                q.parse::<f32>()
                    .ok()
                    .filter(|v| *v >= 0. && *v <= 1.)
                    .map(|v| (v * 100.) as u8)?,
            )),
        })
        .max_by_key(|(_, q)| *q)
        .map(|(c, _)| c)
}
