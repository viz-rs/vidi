use std::fmt::{self, Write};

use bytes::Bytes;

/// Event Message
///
/// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#event_stream_format>
#[allow(clippy::struct_field_names)]
#[derive(Debug, Default)]
pub struct Event {
    id: Option<String>,
    data: Option<String>,
    event: Option<String>,
    retry: Option<u64>,
    comment: Option<String>,
}

impl Event {
    /// The event ID to set the `EventSource` object's last event ID value.
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id.replace(id.into());
        self
    }

    /// The data field for the message.
    #[must_use]
    pub fn data(mut self, data: impl Into<String>) -> Self {
        self.data.replace(data.into());
        self
    }

    /// A string identifying the type of event described.
    #[must_use]
    pub fn event(mut self, event: impl Into<String>) -> Self {
        self.event.replace(event.into());
        self
    }

    /// The reconnection time.
    #[must_use]
    pub fn retry(mut self, retry: u64) -> Self {
        self.retry.replace(retry);
        self
    }

    /// The comment field for the message.
    #[must_use]
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment.replace(comment.into());
        self
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(comment) = &self.comment {
            ":".fmt(f)?;
            comment.fmt(f)?;
            f.write_char('\n')?;
        }
        if let Some(event) = &self.event {
            "event:".fmt(f)?;
            event.fmt(f)?;
            f.write_char('\n')?;
        }
        if let Some(data) = &self.data {
            for line in data.lines() {
                "data: ".fmt(f)?;
                line.fmt(f)?;
                f.write_char('\n')?;
            }
        }
        if let Some(id) = &self.id {
            "id:".fmt(f)?;
            id.fmt(f)?;
            f.write_char('\n')?;
        }
        if let Some(millis) = self.retry {
            "retry:".fmt(f)?;
            millis.fmt(f)?;
            f.write_char('\n')?;
        }
        f.write_char('\n')
    }
}

impl From<Event> for Bytes {
    fn from(e: Event) -> Self {
        Self::from(e.to_string())
    }
}
