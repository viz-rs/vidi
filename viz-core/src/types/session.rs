//! Represents a session extractor.

use std::{
    convert::Infallible,
    fmt,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, RwLock,
    },
};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value, Value};

use sessions_core::{Data, State, CHANGED, PURGED, RENEWED, UNCHANGED};

use crate::{async_trait, Error, FromRequest, IntoResponse, Request, RequestExt, StatusCode};

/// A session for the current request.
#[derive(Clone)]
pub struct Session {
    state: Arc<State>,
}

impl Session {
    /// Creates new `Session` with `Data`
    #[must_use]
    pub fn new(data: Data) -> Self {
        Self {
            state: Arc::new(State {
                status: AtomicU8::new(UNCHANGED),
                data: RwLock::new(data),
            }),
        }
    }

    /// Gets status of the session
    #[must_use]
    pub fn status(&self) -> &AtomicU8 {
        &self.state.status
    }

    /// Gets lock data of the session
    #[must_use]
    pub fn lock_data(&self) -> &RwLock<Data> {
        &self.state.data
    }

    /// Gets a value by the key
    ///
    /// # Errors
    ///
    /// TODO
    pub fn get<T>(&self, key: &str) -> Result<Option<T>, Error>
    where
        T: DeserializeOwned,
    {
        let read = self
            .lock_data()
            .read()
            .map_err(|e| responder_error((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())))?;

        let val = read.get(key).cloned();

        match val {
            Some(t) => from_value(t).map(Some).map_err(report_error),
            None => Ok(None),
        }
    }

    /// Sets a value by the key
    ///
    /// # Errors
    ///
    /// TODO
    pub fn set<T>(&self, key: &str, val: T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let status = self.status().load(Ordering::Acquire);
        // not allowed `PURGED`
        if status != PURGED {
            if let Ok(mut d) = self.lock_data().write() {
                // not allowed `RENEWED & CHANGED`
                if status == UNCHANGED {
                    self.status().store(CHANGED, Ordering::SeqCst);
                }
                d.insert(key.into(), to_value(val).map_err(report_error)?);
            }
        }
        Ok(())
    }

    /// Removes a value
    #[must_use]
    pub fn remove(&self, key: &str) -> Option<Value> {
        let status = self.status().load(Ordering::Acquire);
        // not allowed `PURGED`
        if status != PURGED {
            if let Ok(mut d) = self.lock_data().write() {
                // not allowed `RENEWED & CHANGED`
                if status == UNCHANGED {
                    self.status().store(CHANGED, Ordering::SeqCst);
                }
                return d.remove(key);
            }
        }
        None
    }

    /// Removes a value and deserialize
    #[must_use]
    pub fn remove_as<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        self.remove(key).and_then(|t| from_value(t).ok())
    }

    /// Clears the state
    pub fn clear(&self) {
        let status = self.status().load(Ordering::Acquire);
        // not allowed `PURGED`
        if status != PURGED {
            if let Ok(mut d) = self.lock_data().write() {
                // not allowed `RENEWED & CHANGED`
                if status == UNCHANGED {
                    self.status().store(CHANGED, Ordering::SeqCst);
                }
                d.clear();
            }
        }
    }

    /// Renews the new state
    pub fn renew(&self) {
        let status = self.status().load(Ordering::Acquire);
        // not allowed `PURGED & RENEWED`
        if status != PURGED && status != RENEWED {
            self.status().store(RENEWED, Ordering::SeqCst);
        }
    }

    /// Destroys the current state from store
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

    /// Gets all raw key-value data from the session
    ///
    /// # Errors
    ///
    /// TODO
    pub fn data(&self) -> Result<Data, Error> {
        self.lock_data()
            .read()
            .map_err(|e| responder_error((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())))
            .map(|d| d.clone())
    }
}

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.state.fmt(f)
    }
}

#[async_trait]
impl FromRequest for Session {
    type Error = Infallible;

    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(req.session().clone())
    }
}

fn responder_error(e: (StatusCode, String)) -> Error {
    Error::Responder(e.into_response())
}

fn report_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> Error {
    Error::Report(
        Box::new(e),
        StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    )
}
