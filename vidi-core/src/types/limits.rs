use std::{convert::Infallible, sync::Arc};

use crate::{FromRequest, Request, RequestLimitsExt};

#[cfg(feature = "form")]
use super::Form;

#[cfg(feature = "json")]
use super::Json;

#[cfg(any(feature = "form", feature = "json"))]
use super::Payload;

/// Extracts the limits configuration.
#[derive(Clone, Debug)]
pub struct Limits {
    inner: Arc<Vec<(&'static str, u64)>>,
}

impl Default for Limits {
    fn default() -> Self {
        let limits = Self::new()
            .set("bytes", Self::NORMAL)
            .set("payload", Self::NORMAL)
            .set("text", Self::NORMAL);

        #[cfg(feature = "json")]
        let limits = limits.set(<Json as Payload>::NAME, <Json as Payload>::LIMIT);

        #[cfg(feature = "form")]
        let limits = limits.set(<Form as Payload>::NAME, <Form as Payload>::LIMIT);

        limits.sort()
    }
}

impl Limits {
    /// The default limit: 8 KB.
    pub const NORMAL: u64 = 1024 * 8;

    /// Creates [`Limits`] with no entries.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Vec::new()),
        }
    }

    /// Sorts the limits by name.
    #[must_use]
    pub fn sort(mut self) -> Self {
        Arc::make_mut(&mut self.inner).sort_by_key(|a| a.0);
        self
    }

    /// Sets the limit for the given name.
    #[must_use]
    pub fn set(mut self, name: &'static str, limit: u64) -> Self {
        if let Some(val) = self
            .inner
            .binary_search_by_key(&name, |&(a, _)| a)
            .ok()
            .and_then(|i| Arc::make_mut(&mut self.inner).get_mut(i))
        {
            val.1 = limit;
        } else {
            Arc::make_mut(&mut self.inner).push((name, limit));
        }
        self
    }

    /// Returns the limit for the given name.
    pub fn get<S>(&self, name: S) -> Option<u64>
    where
        S: AsRef<str>,
    {
        self.inner
            .binary_search_by_key(&name.as_ref(), |&(a, _)| a)
            .map(|i| self.inner[i].1)
            .ok()
    }
}

impl FromRequest for Limits {
    type Error = Infallible;

    #[allow(clippy::unused_async_trait_impl)]
    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(req.limits().clone())
    }
}
