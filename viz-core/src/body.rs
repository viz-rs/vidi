use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{stream::Stream, TryStreamExt};
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody};
use hyper::body::{Body, Frame, Incoming, SizeHint};

use crate::{Bytes, Error, Result};

#[derive(Debug)]
/// Incoming Body from request.
pub enum IncomingBody {
    /// A empty body.
    Empty,
    /// A incoming body.
    Incoming(Option<Incoming>),
}

impl IncomingBody {
    /// Creates new Incoming Body
    #[must_use]
    pub fn new(inner: Option<Incoming>) -> Self {
        Self::Incoming(inner)
    }

    /// Incoming body has been used
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
            Self::Empty | Self::Incoming(None) => Poll::Ready(None),
            Self::Incoming(s) => {
                match Pin::new(s.as_mut().unwrap()).poll_frame(cx)? {
                    Poll::Ready(Some(f)) => Poll::Ready(Some(Ok(f))),
                    Poll::Ready(None) => {
                        // the body has been used.
                        *s = None;
                        Poll::Ready(None)
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            Self::Empty | Self::Incoming(None) => true,
            Self::Incoming(Some(inner)) => inner.is_end_stream(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            Self::Empty | Self::Incoming(None) => SizeHint::with_exact(0),
            Self::Incoming(Some(inner)) => inner.size_hint(),
        }
    }
}

impl Stream for IncomingBody {
    type Item = Result<Bytes, Box<dyn std::error::Error + Send + Sync + 'static>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            Self::Empty | Self::Incoming(None) => Poll::Ready(None),
            Self::Incoming(Some(inner)) => match Pin::new(inner).poll_frame(cx)? {
                Poll::Ready(Some(f)) => Poll::Ready(f.into_data().map(Ok).ok()),
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

#[derive(Debug)]
/// Outgoing Body to response.
pub enum OutgoingBody<D = Bytes> {
    /// A empty body.
    Empty,
    /// A body that consists of a single chunk.
    Full(Full<D>),
    /// A boxed [`Body`] trait object.
    Boxed(BoxBody<D, Error>),
}

impl OutgoingBody {
    /// A body created from a [`Stream`].
    pub fn streaming<S, D, E>(stream: S) -> Self
    where
        S: Stream<Item = Result<D, E>> + Send + Sync + 'static,
        D: Into<Bytes>,
        E: Into<Error> + 'static,
    {
        Self::Boxed(BodyExt::boxed(StreamBody::new(
            stream
                .map_ok(|data| Frame::<Bytes>::data(data.into()))
                .map_err(Into::into),
        )))
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
            Self::Full(f) => Pin::new(f).poll_frame(cx).map_err(Error::from),
            Self::Boxed(b) => Pin::new(b).poll_frame(cx),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Full(f) => f.is_end_stream(),
            Self::Boxed(b) => b.is_end_stream(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            Self::Empty => SizeHint::with_exact(0),
            Self::Full(f) => f.size_hint(),
            Self::Boxed(b) => b.size_hint(),
        }
    }
}

impl Stream for OutgoingBody {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match match self.get_mut() {
            Self::Empty => return Poll::Ready(None),
            Self::Full(f) => Pin::new(f)
                .poll_frame(cx)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
            Self::Boxed(b) => Pin::new(b)
                .poll_frame(cx)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
        } {
            Poll::Ready(Some(f)) => Poll::Ready(f.into_data().map(Ok).ok()),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let sh = match self {
            Self::Empty => return (0, Some(0)),
            Self::Full(f) => f.size_hint(),
            Self::Boxed(b) => b.size_hint(),
        };
        (
            usize::try_from(sh.lower()).unwrap_or(usize::MAX),
            sh.upper().map(|v| usize::try_from(v).unwrap_or(usize::MAX)),
        )
    }
}

impl<D> From<()> for OutgoingBody<D> {
    fn from(_: ()) -> Self {
        Self::Empty
    }
}

impl<D> From<Full<D>> for OutgoingBody<D> {
    fn from(value: Full<D>) -> Self {
        Self::Full(value)
    }
}

impl<D> From<BoxBody<D, Error>> for OutgoingBody<D> {
    fn from(value: BoxBody<D, Error>) -> Self {
        Self::Boxed(value)
    }
}
