//! Session extractor.

use std::{
    convert::Infallible,
    fmt,
    sync::{
        Arc, RwLock,
        atomic::{AtomicU8, Ordering},
    },
};

use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, from_value, to_value};

use sessions_core::{CHANGED, Data, PURGED, RENEWED, State, UNCHANGED};

use crate::{Error, FromRequest, IntoResponse, Request, RequestExt, StatusCode};

/// A session for the current request.
#[derive(Clone)]
pub struct Session {
    state: Arc<State>,
}

impl Session {
    /// Creates a [`Session`] from `Data`.
    #[must_use]
    pub fn new(data: Data) -> Self {
        Self {
            state: Arc::new(State {
                status: AtomicU8::new(UNCHANGED),
                data: RwLock::new(data),
            }),
        }
    }

    /// Returns the session status.
    #[must_use]
    pub fn status(&self) -> &AtomicU8 {
        &self.state.status
    }

    /// Returns the session data lock.
    #[must_use]
    pub fn lock_data(&self) -> &RwLock<Data> {
        &self.state.data
    }

    /// Returns the value for the given key.
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be read or deserialized.
    pub fn get<T>(&self, key: &str) -> Result<Option<T>, Error>
    where
        T: DeserializeOwned,
    {
        self.lock_data()
            .read()
            .map_err(|e| responder_error((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())))?
            .get(key)
            .cloned()
            .map_or_else(
                || Ok(None),
                |t| from_value(t).map(Some).map_err(report_error),
            )
    }

    /// Sets the value for the given key.
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be serialized.
    pub fn set<T>(&self, key: &str, val: T) -> Result<(), Error>
    where
        T: Serialize,
    {
        self.with_data_mut(|d| {
            d.insert(key.into(), to_value(val).map_err(report_error)?);
            Ok(())
        })
        .transpose()
        .map(|_| ())
    }

    /// Removes a key from the session, returning the value at the key if the key was previously in
    /// the session.
    #[allow(clippy::must_use_candidate)]
    pub fn remove(&self, key: &str) -> Option<Value> {
        self.with_data_mut(|d| d.remove(key)).flatten()
    }

    /// Removes and deserializes the value for the key.
    #[allow(clippy::must_use_candidate)]
    pub fn remove_as<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        self.remove(key).and_then(|t| from_value(t).ok())
    }

    /// Clears the session data.
    pub fn clear(&self) {
        self.with_data_mut(Data::clear);
    }

    /// Marks the session for renewal.
    pub fn renew(&self) {
        let status = self.status().load(Ordering::Acquire);
        // not allowed `PURGED & RENEWED`
        if status != PURGED && status != RENEWED {
            self.status().store(RENEWED, Ordering::SeqCst);
        }
    }

    /// Marks the session for purging.
    pub fn purge(&self) {
        let status = self.status().load(Ordering::Acquire);
        // not allowed `PURGED`
        if status != PURGED {
            self.status().store(PURGED, Ordering::SeqCst);
            if let Ok(mut d) = self.lock_data().write() {
                d.clear();
            }
        }
    }

    /// Returns the raw session data.
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be read.
    #[allow(clippy::must_use_candidate)]
    pub fn data(&self) -> Result<Data, Error> {
        self.lock_data()
            .read()
            .map_err(|e| responder_error((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())))
            .map(|d| d.clone())
    }

    /// Applies `f` to the session data if it is not purged, marking it changed.
    fn with_data_mut<T>(&self, f: impl FnOnce(&mut Data) -> T) -> Option<T> {
        let status = self.status().load(Ordering::Acquire);
        if status == PURGED {
            return None;
        }
        if let Ok(mut d) = self.lock_data().write() {
            if status == UNCHANGED {
                self.status().store(CHANGED, Ordering::SeqCst);
            }
            Some(f(&mut d))
        } else {
            None
        }
    }
}

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.state.fmt(f)
    }
}

impl FromRequest for Session {
    type Error = Infallible;

    #[allow(clippy::unused_async_trait_impl)]
    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(req.session().clone())
    }
}

fn responder_error(e: (StatusCode, String)) -> Error {
    Error::Response(Box::new(e.into_response()))
}

fn report_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> Error {
    Error::WithResponse(
        Box::new(e),
        Box::new(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    )
}
