use crate::user::UsersMap;

use serde_json::json;
use std::sync::{PoisonError, RwLockReadGuard, RwLockWriteGuard};

/// A wrapper for [`worker::Error`].
///
/// This struct implements [`From<MutexError>`](MutexError) for [`Self`](Error).
#[derive(Debug)]
pub struct Error(worker::Error);

impl Error {
    pub fn take(self) -> (String, u16) {
        let (msg, status) = match self.0 {
            worker::Error::Json(err) => (err.0, err.1),
            _ => (self.0.to_string(), 500),
        };

        let json_string = json!({
            "error": msg,
            "status": status
        });

        (json_string.to_string(), status)
    }
}

type ReadErr<'a> = PoisonError<RwLockReadGuard<'a, UsersMap>>;
type WriteErr<'a> = PoisonError<RwLockWriteGuard<'a, UsersMap>>;

impl From<ReadErr<'_>> for Error {
    fn from(err: ReadErr<'_>) -> Self {
        Self(worker::Error::RustError(err.to_string()))
    }
}

impl From<WriteErr<'_>> for Error {
    fn from(err: WriteErr<'_>) -> Self {
        Self(worker::Error::RustError(err.to_string()))
    }
}

impl From<worker::Error> for Error {
    fn from(err: worker::Error) -> Self {
        Self(err)
    }
}

impl From<Error> for worker::Error {
    fn from(err: Error) -> Self {
        err.0
    }
}

impl From<(String, u16)> for Error {
    fn from(err: (String, u16)) -> Self {
        Self(worker::Error::Json((err.0, err.1)))
    }
}
