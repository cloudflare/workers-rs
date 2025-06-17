use js_sys::Array;
use std::convert::TryFrom;
use wasm_bindgen::{JsCast, JsValue};
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

impl TryFrom<JsValue> for SqlStorageValue {
    type Error = crate::Error;

    fn try_from(js_val: JsValue) -> Result<Self> {
        if js_val.is_null() || js_val.is_undefined() {
            Ok(SqlStorageValue::Null)
        } else if let Some(bool_val) = js_val.as_bool() {
            Ok(SqlStorageValue::Boolean(bool_val))
        } else if let Some(str_val) = js_val.as_string() {
            Ok(SqlStorageValue::String(str_val))
        } else if let Some(num_val) = js_val.as_f64() {
            if num_val.fract() == 0.0 && num_val >= i32::MIN as f64 && num_val <= i32::MAX as f64 {
                Ok(SqlStorageValue::Integer(num_val as i32))
            } else {
                Ok(SqlStorageValue::Float(num_val))
            }
        } else if js_val.is_instance_of::<js_sys::Uint8Array>() {
            let uint8_array = js_sys::Uint8Array::from(js_val);
            let mut bytes = vec![0u8; uint8_array.length() as usize];
            uint8_array.copy_to(&mut bytes);
            Ok(SqlStorageValue::Blob(bytes))
        } else {
            Err(Error::from("Unsupported JavaScript value type"))
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

/// Iterator wrapper for typed cursor results.
///
/// This iterator yields deserialized rows of type `T`, providing a type-safe
/// way to iterate over SQL query results with automatic conversion to Rust types.
pub struct SqlCursorIterator<T> {
    cursor: SqlCursor,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Iterator for SqlCursorIterator<T>
where
    T: DeserializeOwned,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.cursor.inner.next();

        let done = js_sys::Reflect::get(&result, &JsValue::from("done"))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if done {
            None
        } else {
            let value = js_sys::Reflect::get(&result, &JsValue::from("value"))
                .map_err(Error::from)
                .and_then(|js_val| swb::from_value(js_val).map_err(Error::from));
            Some(value)
        }
    }
}

/// Iterator wrapper for raw cursor results as Vec<SqlStorageValue>.
///
/// This iterator yields raw values as `Vec<SqlStorageValue>`, providing efficient
/// access to SQL data without deserialization overhead. Useful when you need
/// direct access to the underlying SQL values without column names.
pub struct SqlCursorRawIterator {
    inner: js_sys::Iterator,
}

impl Iterator for SqlCursorRawIterator {
    type Item = Result<Vec<SqlStorageValue>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Ok(iterator_next) => {
                if iterator_next.done() {
                    None
                } else {
                    let js_val = iterator_next.value();
                    let array_result = js_array_to_sql_storage_values(js_val);
                    Some(array_result)
                }
            }
            Err(e) => Some(Err(Error::from(e))),
        }
    }
}

fn js_array_to_sql_storage_values(js_val: JsValue) -> Result<Vec<SqlStorageValue>> {
    let array = js_sys::Array::from(&js_val);
    let mut values = Vec::with_capacity(array.length() as usize);

    for i in 0..array.length() {
        let item = array.get(i);
        let sql_value = SqlStorageValue::try_from(item)?;
        values.push(sql_value);
    }

    Ok(values)
}

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

    /// Returns a Rust iterator that yields deserialized rows of type T.
    ///
    /// This provides a first-class Rust API for iterating over query results
    /// with proper type safety and error handling.
    pub fn next<T>(&self) -> SqlCursorIterator<T>
    where
        T: DeserializeOwned,
    {
        SqlCursorIterator {
            cursor: self.clone(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Returns a Rust iterator where each row is a Vec<SqlStorageValue>.
    ///
    /// This method provides a more efficient way to iterate over results when you
    /// only need the raw values without column names, using proper Rust types.
    pub fn raw(&self) -> SqlCursorRawIterator {
        SqlCursorRawIterator {
            inner: self.inner.raw(),
        }
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
            let value = js_sys::Reflect::get(&result, &JsValue::from("value")).map_err(Error::from);
            Some(value)
        }
    }
}
