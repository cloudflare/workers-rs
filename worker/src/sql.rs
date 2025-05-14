use js_sys::Array;
use wasm_bindgen::JsValue;
use worker_sys::types::{SqlStorage as SqlStorageSys, SqlStorageCursor as SqlStorageCursorSys};

use serde::de::DeserializeOwned;
use serde_wasm_bindgen as swb;

use crate::Error;
use crate::Result;

/// Wrapper around the Workers `SqlStorage` interface exposed at `ctx.storage.sql`.
///
/// The API is intentionally minimal for now – additional helper utilities can be built on top
/// as needed.
#[derive(Clone)]
pub struct SqlStorage {
    inner: SqlStorageSys,
}

unsafe impl Send for SqlStorage {}
unsafe impl Sync for SqlStorage {}

impl SqlStorage {
    pub(crate) fn new(inner: SqlStorageSys) -> Self {
        Self { inner }
    }

    /// Size of the underlying SQLite database in bytes.
    pub fn database_size(&self) -> usize {
        self.inner.database_size() as usize
    }

    /// Execute a SQL query and return a cursor over the results.
    ///
    /// `bindings` correspond to positional `?` placeholders in the query.
    pub fn exec(&self, query: &str, bindings: Vec<JsValue>) -> Result<SqlCursor> {
        let array = Array::new();
        for v in bindings {
            array.push(&v);
        }
        let cursor = self.inner.exec(query, array).map_err(Error::from)?;
        Ok(SqlCursor { inner: cursor })
    }
}

impl AsRef<JsValue> for SqlStorage {
    fn as_ref(&self) -> &JsValue {
        &self.inner
    }
}

/// A cursor returned from `SqlStorage::exec`.
#[derive(Clone)]
pub struct SqlCursor {
    inner: SqlStorageCursorSys,
}

unsafe impl Send for SqlCursor {}
unsafe impl Sync for SqlCursor {}

impl SqlCursor {
    /// Consume the remaining rows of the cursor into a `Vec` of deserialised objects.
    pub fn to_array<T>(&self) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let arr = self.inner.to_array();
        let mut out = Vec::with_capacity(arr.length() as usize);
        for val in arr.iter() {
            out.push(swb::from_value(val)?);
        }
        Ok(out)
    }

    /// Return the first (and only) row of the query result.
    pub fn one<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let val = self.inner.one();
        Ok(swb::from_value(val)?)
    }

    /// Column names returned by the query.
    pub fn column_names(&self) -> Vec<String> {
        self.inner
            .column_names()
            .iter()
            .map(|v| v.as_string().unwrap_or_default())
            .collect()
    }

    /// Number of rows read so far by the cursor.
    pub fn rows_read(&self) -> usize {
        self.inner.rows_read() as usize
    }

    /// Number of rows written by the query so far.
    pub fn rows_written(&self) -> usize {
        self.inner.rows_written() as usize
    }
}
