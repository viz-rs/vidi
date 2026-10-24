use std::fmt::{self, Write};

use bytes::Bytes;

/// An SSE event.
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

macro_rules! impl_event_string_setter {
    ($name:ident, $desc:literal) => {
        #[doc = concat!("Sets the ", $desc, ".")]
        #[must_use]
        pub fn $name(mut self, value: impl Into<String>) -> Self {
            self.$name.replace(value.into());
            self
        }
    };
}

impl Event {
    impl_event_string_setter!(id, "event ID");
    impl_event_string_setter!(data, "data field");
    impl_event_string_setter!(event, "event type");

    /// Sets the reconnection time.
    #[must_use]
    pub fn retry(mut self, retry: u64) -> Self {
        self.retry.replace(retry);
        self
    }

    impl_event_string_setter!(comment, "comment field");
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
