use std::{convert::Infallible, sync::Arc};

use crate::{async_trait, FromRequest, Request, RequestExt};

#[cfg(feature = "form")]
use super::Form;

#[cfg(feature = "json")]
use super::Json;

#[cfg(any(feature = "form", feature = "json"))]
use super::Payload;

/// A extractor for the limits settings.
#[derive(Debug, Clone)]
pub struct Limits {
    inner: Arc<Vec<(&'static str, u64)>>,
}

impl Default for Limits {
    fn default() -> Self {
        let limits = Limits::new()
            .insert("payload", Limits::NORMAL)
            .insert("text", Limits::NORMAL)
            .insert("bytes", Limits::NORMAL);

        #[cfg(feature = "json")]
        let limits = limits.insert("json", <Json as Payload>::LIMIT);

        #[cfg(feature = "form")]
        let limits = limits.insert("form", <Form as Payload>::LIMIT);

        limits
    }
}

impl Limits {
    /// By default 1024 * 8.
    pub const NORMAL: u64 = 1024 * 8;

    /// Creates a new Limits.
    pub fn new() -> Self {
        Limits {
            inner: Arc::new(Vec::new()),
        }
    }

    /// Inserts a name-limit pair into the Limits.
    pub fn insert(mut self, name: &'static str, limit: u64) -> Self {
        Arc::make_mut(&mut self.inner).push((name, limit));
        self
    }

    /// Returns a limit value by the name.
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

#[async_trait]
impl FromRequest for Limits {
    type Error = Infallible;

    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(req.limits())
    }
}
