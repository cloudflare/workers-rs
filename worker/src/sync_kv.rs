//! Cloudflare Durable Objects' Synchronous KV API exposed via
//! [`Storage::kv`](crate::Storage::kv).
//!
//! This is the `ctx.storage.kv` API available on SQLite-backed Durable Objects.
//! Entries are stored in the Durable Object's hidden `__cf_kv` SQLite table.
//!
//! [`SyncKvListOptions`] is re-exported directly from `worker-sys` as an
//! imported JS handle; build one with [`SyncKvListOptionsBuilder`].
//! [`SyncKvStorage`] is a thin Rust wrapper that adds `serde`-aware typed
//! `get`/`put`/`list` methods on top of the imported sys type.

use core::fmt;
use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Serialize};
use serde_wasm_bindgen as swb;
use wasm_bindgen::{JsCast as _, JsValue};
use worker_sys::types::SyncKvStorage as SyncKvStorageSys;

pub use worker_sys::types::SyncKvListOptions;

use crate::{Error, Result};

/// Cloudflare Durable Objects' Synchronous KV API exposed by [`Storage::kv`](crate::Storage::kv).
///
/// Wraps the imported [`worker_sys::types::SyncKvStorage`] handle and provides
/// typed `serde`-aware methods.
#[derive(Clone, Debug)]
pub struct SyncKvStorage {
    inner: SyncKvStorageSys,
}

// SAFETY: workers run single-threaded; matches the convention used by other
// Durable Object wrappers in this crate (e.g. `Storage`, `SqlStorage`).
unsafe impl Send for SyncKvStorage {}
unsafe impl Sync for SyncKvStorage {}

impl SyncKvStorage {
    pub(crate) fn new(inner: SyncKvStorageSys) -> Self {
        Self { inner }
    }

    /// Retrieves the value associated with the given key.
    ///
    /// Returns `Ok(None)` if the key does not exist.
    pub fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let val = self.inner.try_get(key).map_err(Error::from)?;
        if val.is_undefined() {
            Ok(None)
        } else {
            Ok(Some(swb::from_value(val)?))
        }
    }

    /// Stores the value and associates it with the given key.
    pub fn put<T>(&self, key: &str, value: T) -> Result<()>
    where
        T: Serialize,
    {
        let js = swb::to_value(&value)?;
        self.inner.try_put(key, &js).map_err(Error::from)
    }

    /// Deletes the key and associated value.
    ///
    /// Returns `Ok(true)` if the key existed and was removed.
    pub fn delete(&self, key: &str) -> Result<bool> {
        self.inner.try_delete(key).map_err(Error::from)
    }

    /// Returns an iterator over all key-value pairs in ascending lexicographic order.
    pub fn list<T>(&self) -> Result<SyncKvIterator<T>>
    where
        T: DeserializeOwned,
    {
        let iterable = self.inner.try_list().map_err(Error::from)?;
        SyncKvIterator::from_iterable(&iterable)
    }

    /// Returns an iterator over key-value pairs that match the provided [`SyncKvListOptions`].
    pub fn list_with_options<T>(&self, options: &SyncKvListOptions) -> Result<SyncKvIterator<T>>
    where
        T: DeserializeOwned,
    {
        let iterable = self
            .inner
            .try_list_with_options(options)
            .map_err(Error::from)?;
        SyncKvIterator::from_iterable(&iterable)
    }
}

/// Fluent builder for [`SyncKvListOptions`].
///
/// Each method mutates the underlying JS object in place and returns `self`.
/// [`build`](Self::build) returns the configured imported handle.
#[derive(Debug)]
pub struct SyncKvListOptionsBuilder {
    inner: SyncKvListOptions,
}

impl Default for SyncKvListOptionsBuilder {
    fn default() -> Self {
        Self {
            inner: SyncKvListOptions::new(),
        }
    }
}

impl SyncKvListOptionsBuilder {
    /// Create an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Key at which the list results should start, inclusive.
    pub fn start(self, val: &str) -> Self {
        self.inner.set_start(val);
        self
    }

    /// Key at which the list results should start, exclusive.
    /// Cannot be used simultaneously with [`start`](Self::start).
    pub fn start_after(self, val: &str) -> Self {
        self.inner.set_start_after(val);
        self
    }

    /// Key at which the list results should end, exclusive.
    pub fn end(self, val: &str) -> Self {
        self.inner.set_end(val);
        self
    }

    /// Restricts results to only include key-value pairs whose keys begin with the prefix.
    pub fn prefix(self, val: &str) -> Self {
        self.inner.set_prefix(val);
        self
    }

    /// If true, return results in descending lexicographic order.
    pub fn reverse(self, val: bool) -> Self {
        self.inner.set_reverse(val);
        self
    }

    /// Maximum number of key-value pairs to return.
    pub fn limit(self, val: u32) -> Self {
        self.inner.set_limit(val as f64);
        self
    }

    /// Consume the builder and return the configured [`SyncKvListOptions`].
    pub fn build(self) -> SyncKvListOptions {
        self.inner
    }
}

/// Iterator over typed entries returned by [`SyncKvStorage::list`] and
/// [`SyncKvStorage::list_with_options`].
///
/// Each item yields the key together with its `serde`-deserialized value.
pub struct SyncKvIterator<T> {
    inner: js_sys::IntoIter,
    _phantom: PhantomData<T>,
}

impl<T> SyncKvIterator<T> {
    fn from_iterable(iterable: &JsValue) -> Result<Self> {
        let inner = js_sys::try_iter(iterable)?.ok_or_else(|| {
            Error::JsError("SyncKvStorage.list() did not return an iterable".into())
        })?;
        Ok(Self {
            inner,
            _phantom: PhantomData,
        })
    }
}

impl<T> Iterator for SyncKvIterator<T>
where
    T: DeserializeOwned,
{
    type Item = Result<(String, T)>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = match self.inner.next()? {
            Ok(r) => r,
            Err(e) => return Some(Err(Error::from(e))),
        };

        // workerd guarantees `[key, value]` tuples; treat anything else as a JS error.
        let arr: js_sys::Array = match entry.dyn_into() {
            Ok(a) => a,
            Err(_) => {
                return Some(Err(Error::JsError(
                    "Expected SyncKvStorage list entry to be an array".into(),
                )));
            }
        };

        let key = match arr.get(0).as_string() {
            Some(k) => k,
            None => {
                return Some(Err(Error::JsError(
                    "Expected SyncKvStorage list entry key to be a string".into(),
                )));
            }
        };

        let val = match swb::from_value(arr.get(1)) {
            Ok(v) => v,
            Err(e) => return Some(Err(Error::from(e))),
        };

        Some(Ok((key, val)))
    }
}

impl<T> fmt::Debug for SyncKvIterator<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyncKvIterator").finish()
    }
}
