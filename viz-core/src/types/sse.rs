//! Server-Sent Event Extractor
//!
//! [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events>

use futures_util::stream::{select, Stream, StreamExt};
use tokio::time::{interval_at, Duration, Instant};
use tokio_stream::wrappers::IntervalStream;

use crate::{
    header::{CACHE_CONTROL, CONTENT_TYPE},
    headers::{Connection, HeaderMapExt, HeaderValue},
    Bytes, IntoResponse, Response, ResponseExt,
};

mod event;

pub use event::Event;

/// Server-Sent Event
#[derive(Debug)]
pub struct Sse<S> {
    stream: S,
    interval: Option<Duration>,
}

impl<S> Sse<S>
where
    S: Stream<Item = Event> + Send + 'static,
{
    /// Creates a new Server-Sent Event.
    #[must_use]
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            interval: None,
        }
    }

    /// Sets a interval for Server-Sent Event.
    #[must_use]
    pub fn interval(mut self, duration: Duration) -> Self {
        self.interval.replace(duration);
        self
    }
}

impl<S> IntoResponse for Sse<S>
where
    S: Stream<Item = Event> + Send + 'static,
{
    fn into_response(self) -> Response {
        let stream = self.stream.map(|e| Ok::<Bytes, std::io::Error>(e.into()));
        let mut res = if let Some(duration) = self.interval {
            Response::stream(select(
                stream,
                IntervalStream::new(interval_at(Instant::now(), duration))
                    .map(|_| Ok(Event::default().comment(":\n\n").into())),
            ))
        } else {
            Response::stream(stream)
        };

        res.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_EVENT_STREAM.as_ref()),
        );
        res.headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        res.headers_mut().typed_insert(Connection::keep_alive());

        res
    }
}
