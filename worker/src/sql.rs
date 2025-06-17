use js_sys::Array;
use wasm_bindgen::JsValue;
use worker_sys::types::{SqlStorage as SqlStorageSys, SqlStorageCursor as SqlStorageCursorSys};

use serde::de::DeserializeOwned;
use serde_wasm_bindgen as swb;

use crate::Error;
use crate::Result;

/// A value that can be stored in Durable Object SQL storage.
///
/// This enum represents the types that can be safely passed to SQL queries
/// while maintaining type safety and proper conversion to JavaScript values.
#[derive(Debug, Clone, PartialEq)]
pub enum SqlStorageValue {
    /// SQL NULL value
    Null,
    /// Boolean value
    Boolean(bool),
    /// 32-bit signed integer (safe for JavaScript Number conversion)
    Integer(i32),
    /// 64-bit floating point number
    Float(f64),
    /// UTF-8 string
    String(String),
    /// Binary data
    Blob(Vec<u8>),
}

// From implementations for convenient conversion from Rust types
impl From<bool> for SqlStorageValue {
    fn from(value: bool) -> Self {
        SqlStorageValue::Boolean(value)
    }
}

impl From<i32> for SqlStorageValue {
    fn from(value: i32) -> Self {
        SqlStorageValue::Integer(value)
    }
}

impl From<f64> for SqlStorageValue {
    fn from(value: f64) -> Self {
        SqlStorageValue::Float(value)
    }
}

impl From<String> for SqlStorageValue {
    fn from(value: String) -> Self {
        SqlStorageValue::String(value)
    }
}

impl From<&str> for SqlStorageValue {
    fn from(value: &str) -> Self {
        SqlStorageValue::String(value.to_string())
    }
}

impl From<Vec<u8>> for SqlStorageValue {
    fn from(value: Vec<u8>) -> Self {
        SqlStorageValue::Blob(value)
    }
}

impl<T> From<Option<T>> for SqlStorageValue
where
    T: Into<SqlStorageValue>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => SqlStorageValue::Null,
        }
    }
}

// Convert SqlStorageValue to JsValue for passing to JavaScript
impl From<SqlStorageValue> for JsValue {
    fn from(val: SqlStorageValue) -> Self {
        match val {
            SqlStorageValue::Null => JsValue::NULL,
            SqlStorageValue::Boolean(b) => JsValue::from(b),
            SqlStorageValue::Integer(i) => JsValue::from(i),
            SqlStorageValue::Float(f) => JsValue::from(f),
            SqlStorageValue::String(s) => JsValue::from(s),
            SqlStorageValue::Blob(bytes) => {
                // Convert Vec<u8> to Uint8Array for JavaScript
                let array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
                array.copy_from(&bytes);
                array.into()
            }
        }
    }
}

/// Wrapper around the Workers `SqlStorage` interface exposed at `ctx.storage.sql`.
///
/// The API is intentionally minimal for now – additional helper utilities can be built on top
/// as needed.
#[derive(Clone, Debug)]
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
    /// Accepts `SqlStorageValue` for type-safe parameter binding.
    pub fn exec(
        &self,
        query: &str,
        bindings: impl Into<Option<Vec<SqlStorageValue>>>,
    ) -> Result<SqlCursor> {
        let array = Array::new();
        if let Some(bindings) = bindings.into() {
            for v in bindings {
                array.push(&v.into());
            }
        }
        let cursor = self.inner.exec(query, array).map_err(Error::from)?;
        Ok(SqlCursor { inner: cursor })
    }

    /// Execute a SQL query with raw JavaScript values.
    ///
    /// This method provides direct access to `JsValue` parameters for advanced use cases
    /// where you need more control over the JavaScript conversion.
    pub fn exec_raw(
        &self,
        query: &str,
        bindings: impl Into<Option<Vec<JsValue>>>,
    ) -> Result<SqlCursor> {
        let array = Array::new();
        if let Some(bindings) = bindings.into() {
            for v in bindings {
                array.push(&v);
            }
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
#[derive(Clone, Debug)]
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

    /// JavaScript Iterator.next() implementation.
    ///
    /// Returns an object with `{ done, value }` properties. This allows the cursor
    /// to be used as a JavaScript iterator.
    pub fn next(&self) -> js_sys::Object {
        self.inner.next()
    }

    /// Returns an iterator where each row is an array rather than an object.
    ///
    /// This method provides a more efficient way to iterate over results when you
    /// only need the raw values without column names.
    pub fn raw(&self) -> js_sys::Iterator {
        self.inner.raw()
    }
}

impl Iterator for SqlCursor {
    type Item = Result<JsValue>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.inner.next();
        
        // Extract 'done' property from iterator result
        let done = js_sys::Reflect::get(&result, &JsValue::from("done"))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        if done {
            None
        } else {
            // Extract 'value' property from iterator result
            let value = js_sys::Reflect::get(&result, &JsValue::from("value"))
                .map_err(Error::from);
            Some(value)
        }
    }
}