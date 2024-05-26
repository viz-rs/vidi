use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_util::{Stream, TryStream, TryStreamExt};
use http_body_util::{combinators::UnsyncBoxBody, BodyExt, BodyStream, Full, StreamBody};
use hyper::body::{Frame, Incoming, SizeHint};
use sync_wrapper::SyncWrapper;

use crate::{BoxError, Error, HttpBody, Result};

/// A body state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BodyState {
    /// The body is inited.
    Normal,
    /// The body is empty.
    Empty,
    /// The body has ben used.
    Used,
}

/// A body for HTTP [`Request`] and HTTP [`Response`].
///
/// [`Request`]: crate::Request
/// [`Response`]: crate::Response
#[derive(Debug)]
pub enum Body<D = Bytes> {
    /// An empty body.
    Empty,
    /// A body that consists of a single chunk.
    Full(Full<D>),
    /// An incoming body.
    Incoming(Incoming),
    /// A boxed [`Body`] trait object.
    Boxed(SyncWrapper<UnsyncBoxBody<D, Error>>),
}

impl Body {
    /// Creates an empty body.
    #[must_use]
    pub const fn empty() -> Self {
        Self::Empty
    }

    /// Wraps a body into box.
    #[allow(clippy::missing_panics_doc)]
    pub fn wrap<B>(body: B) -> Self
    where
        B: HttpBody + Send + 'static,
        B::Data: Into<Bytes>,
        B::Error: Into<BoxError>,
    {
        // Copied from Axum, thanks.
        let mut body = Some(body);
        <dyn std::any::Any>::downcast_mut::<Option<UnsyncBoxBody<Bytes, Error>>>(&mut body)
            .and_then(Option::take)
            .unwrap_or_else(|| {
                body.unwrap()
                    .map_frame(|frame| frame.map_data(Into::into))
                    .map_err(Error::boxed)
                    .boxed_unsync()
            })
            .into()
    }

    /// A body created from a [`Stream`].
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: TryStream + Send + 'static,
        S::Ok: Into<Bytes>,
        S::Error: Into<BoxError>,
    {
        StreamBody::new(
            stream
                .map_ok(Into::into)
                .map_ok(Frame::data)
                .map_err(Error::boxed),
        )
        .boxed_unsync()
        .into()
    }

    /// A stream created from a [`http_body::Body`].
    pub fn into_stream(self) -> BodyStream<Self> {
        BodyStream::new(self)
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::Empty
    }
}

impl HttpBody for Body {
    type Data = Bytes;
    type Error = Error;

    #[inline]
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.get_mut() {
            Self::Empty => Poll::Ready(None),
            Self::Full(inner) => Pin::new(inner).poll_frame(cx).map_err(Into::into),
            Self::Incoming(inner) => Pin::new(inner).poll_frame(cx).map_err(Into::into),
            Self::Boxed(inner) => Pin::new(inner).get_pin_mut().poll_frame(cx),
        }
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Boxed(_) => false,
            Self::Full(inner) => inner.is_end_stream(),
            Self::Incoming(inner) => inner.is_end_stream(),
        }
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self {
            Self::Boxed(_) => SizeHint::default(),
            Self::Empty => SizeHint::with_exact(0),
            Self::Full(inner) => inner.size_hint(),
            Self::Incoming(inner) => inner.size_hint(),
        }
    }
}

impl Stream for Body {
    type Item = Result<Bytes, std::io::Error>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match match self.get_mut() {
            Self::Empty => return Poll::Ready(None),
            Self::Full(inner) => Pin::new(inner)
                .poll_frame(cx)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
            Self::Incoming(inner) => Pin::new(inner)
                .poll_frame(cx)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
            Self::Boxed(inner) => Pin::new(inner)
                .get_pin_mut()
                .poll_frame(cx)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
        } {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(frame)) => Poll::Ready(frame.into_data().map(Ok).ok()),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let sh = match self {
            Self::Empty => return (0, Some(0)),
            Self::Boxed(_) => return (0, None),
            Self::Full(inner) => inner.size_hint(),
            Self::Incoming(inner) => inner.size_hint(),
        };
        (
            usize::try_from(sh.lower()).unwrap_or(usize::MAX),
            sh.upper().map(|v| usize::try_from(v).unwrap_or(usize::MAX)),
        )
    }
}

impl From<()> for Body {
    fn from((): ()) -> Self {
        Self::Empty
    }
}

impl<D> From<Full<D>> for Body<D> {
    fn from(value: Full<D>) -> Self {
        Self::Full(value)
    }
}

impl<D> From<UnsyncBoxBody<D, Error>> for Body<D> {
    fn from(value: UnsyncBoxBody<D, Error>) -> Self {
        Self::Boxed(SyncWrapper::new(value))
    }
}
