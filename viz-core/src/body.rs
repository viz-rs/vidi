use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{Stream, TryStreamExt};
use http_body_util::{combinators::UnsyncBoxBody, BodyExt, Full, StreamBody};
use hyper::body::{Body, Frame, Incoming, SizeHint};
use sync_wrapper::SyncWrapper;

use crate::{BoxError, Bytes, Error, Result};

/// The incoming body from HTTP [`Request`].
///
/// [`Request`]: crate::Request
#[derive(Debug)]
pub enum IncomingBody {
    /// An empty body.
    Empty,
    /// An incoming body.
    Incoming(Option<Incoming>),
}

impl IncomingBody {
    /// Creates new incoming body.
    #[must_use]
    pub fn new(inner: Option<Incoming>) -> Self {
        Self::Incoming(inner)
    }

    /// The incoming body has been used.
    #[must_use]
    pub fn used() -> Self {
        Self::Incoming(None)
    }
}

impl Default for IncomingBody {
    fn default() -> Self {
        Self::Empty
    }
}

impl Body for IncomingBody {
    type Data = Bytes;
    type Error = Error;

    #[inline]
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.get_mut() {
            Self::Empty => Poll::Ready(None),
            Self::Incoming(i) => match i {
                None => Poll::Ready(None),
                Some(inner) => match Pin::new(inner).poll_frame(cx)? {
                    Poll::Ready(Some(frame)) => Poll::Ready(Some(Ok(frame))),
                    Poll::Ready(None) => {
                        // the body has been used.
                        *i = None;
                        Poll::Ready(None)
                    }
                    Poll::Pending => Poll::Pending,
                },
            },
        }
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        match self {
            Self::Empty | Self::Incoming(None) => true,
            Self::Incoming(Some(inner)) => inner.is_end_stream(),
        }
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self {
            Self::Empty | Self::Incoming(None) => SizeHint::with_exact(0),
            Self::Incoming(Some(inner)) => inner.size_hint(),
        }
    }
}

impl Stream for IncomingBody {
    type Item = Result<Bytes, BoxError>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            Self::Empty | Self::Incoming(None) => Poll::Ready(None),
            Self::Incoming(Some(inner)) => match Pin::new(inner).poll_frame(cx)? {
                Poll::Ready(Some(frame)) => Poll::Ready(frame.into_data().map(Ok).ok()),
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            },
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Empty | Self::Incoming(None) => (0, Some(0)),
            Self::Incoming(Some(inner)) => {
                let sh = inner.size_hint();
                (
                    usize::try_from(sh.lower()).unwrap_or(usize::MAX),
                    sh.upper().map(|v| usize::try_from(v).unwrap_or(usize::MAX)),
                )
            }
        }
    }
}

/// The outgoing body to HTTP [`Response`].
///
/// [`Response`]: crate::Response
#[derive(Debug)]
pub enum OutgoingBody<D = Bytes> {
    /// An empty body.
    Empty,
    /// A body that consists of a single chunk.
    Full(Full<D>),
    /// A boxed [`Body`] trait object.
    Boxed(SyncWrapper<UnsyncBoxBody<D, Error>>),
}

impl OutgoingBody {
    /// A body created from a [`Stream`].
    pub fn stream<S, D, E>(stream: S) -> Self
    where
        S: Stream<Item = Result<D, E>> + Send + 'static,
        D: Into<Bytes> + 'static,
        E: Into<Error> + 'static,
    {
        Self::Boxed(SyncWrapper::new(
            StreamBody::new(
                stream
                    .map_ok(Into::into)
                    .map_ok(Frame::data)
                    .map_err(Into::into),
            )
            .boxed_unsync(),
        ))
    }
}

impl Default for OutgoingBody {
    fn default() -> Self {
        Self::Empty
    }
}

impl Body for OutgoingBody {
    type Data = Bytes;
    type Error = Error;

    #[inline]
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.get_mut() {
            Self::Empty => Poll::Ready(None),
            Self::Full(full) => Pin::new(full).poll_frame(cx).map_err(Error::from),
            Self::Boxed(body) => Pin::new(body).get_pin_mut().poll_frame(cx),
        }
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Boxed(_) => false,
            Self::Full(full) => full.is_end_stream(),
        }
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self {
            Self::Empty => SizeHint::with_exact(0),
            Self::Boxed(_) => SizeHint::default(),
            Self::Full(full) => full.size_hint(),
        }
    }
}

impl Stream for OutgoingBody {
    type Item = Result<Bytes, std::io::Error>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match match self.get_mut() {
            Self::Empty => return Poll::Ready(None),
            Self::Full(full) => Pin::new(full)
                .poll_frame(cx)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
            Self::Boxed(wrapper) => Pin::new(wrapper)
                .get_pin_mut()
                .poll_frame(cx)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
        } {
            Poll::Ready(Some(frame)) => Poll::Ready(frame.into_data().map(Ok).ok()),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let sh = match self {
            Self::Empty => return (0, Some(0)),
            Self::Boxed(_) => return (0, None),
            Self::Full(full) => full.size_hint(),
        };
        (
            usize::try_from(sh.lower()).unwrap_or(usize::MAX),
            sh.upper().map(|v| usize::try_from(v).unwrap_or(usize::MAX)),
        )
    }
}

impl<D> From<()> for OutgoingBody<D> {
    fn from((): ()) -> Self {
        Self::Empty
    }
}

impl<D> From<Full<D>> for OutgoingBody<D> {
    fn from(value: Full<D>) -> Self {
        Self::Full(value)
    }
}

impl<D> From<UnsyncBoxBody<D, Error>> for OutgoingBody<D> {
    fn from(value: UnsyncBoxBody<D, Error>) -> Self {
        Self::Boxed(SyncWrapper::new(value))
    }
}
