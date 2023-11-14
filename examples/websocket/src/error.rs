use crate::counter::Users;
use std::sync::{MutexGuard, PoisonError};

/// A wrapper for [worker::Error].
/// This struct implements [`From<MutexError>`](MutexError) for [`Self`](Error).
pub struct Error(worker::Error);

impl Error {
    pub fn new(msg: String) -> Self {
        Self(worker::Error::RustError(msg))
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

type MutexError<'a> = PoisonError<MutexGuard<'a, Users>>;

impl From<MutexError<'_>> for Error {
    fn from(err: MutexError<'_>) -> Self {
        Self(worker::Error::RustError(err.to_string()))
    }
}
