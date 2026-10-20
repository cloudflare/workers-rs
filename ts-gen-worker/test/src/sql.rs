//! `worker::SqlStorageValue` and cursor iteration over the generated
//! `SqlStorage` / `SqlStorageCursor`, whose row values are erased to `JsValue`.

use js_sys::Uint8Array;
use serde::de::DeserializeOwned;
use ts_gen_worker::*;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SqlStorageValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Blob(Vec<u8>),
}

impl SqlStorageValue {
    pub fn try_from_i64(value: i64) -> Result<Self> {
        if value >= js_sys::Number::MIN_SAFE_INTEGER as i64
            && value <= js_sys::Number::MAX_SAFE_INTEGER as i64
        {
            Ok(SqlStorageValue::Integer(value))
        } else {
            Err("Value outside JavaScript safe integer range".into())
        }
    }
}

impl From<&str> for SqlStorageValue {
    fn from(value: &str) -> Self {
        SqlStorageValue::String(value.to_owned())
    }
}

impl From<i32> for SqlStorageValue {
    fn from(value: i32) -> Self {
        SqlStorageValue::Integer(i64::from(value))
    }
}

impl From<f64> for SqlStorageValue {
    fn from(value: f64) -> Self {
        SqlStorageValue::Float(value)
    }
}

impl From<SqlStorageValue> for JsValue {
    fn from(val: SqlStorageValue) -> Self {
        match val {
            SqlStorageValue::Null => JsValue::NULL,
            SqlStorageValue::Boolean(b) => JsValue::from(b),
            SqlStorageValue::Integer(i) => JsValue::from(i as f64),
            SqlStorageValue::Float(f) => JsValue::from(f),
            SqlStorageValue::String(s) => JsValue::from(s),
            SqlStorageValue::Blob(bytes) => Uint8Array::from(&bytes[..]).into(),
        }
    }
}

impl TryFrom<JsValue> for SqlStorageValue {
    type Error = Error;

    fn try_from(js_val: JsValue) -> Result<Self> {
        if js_val.is_null() || js_val.is_undefined() {
            Ok(SqlStorageValue::Null)
        } else if let Some(bool_val) = js_val.as_bool() {
            Ok(SqlStorageValue::Boolean(bool_val))
        } else if let Some(str_val) = js_val.as_string() {
            Ok(SqlStorageValue::String(str_val))
        } else if let Some(num_val) = js_val.as_f64() {
            if js_sys::Number::is_safe_integer(&js_val) {
                Ok(SqlStorageValue::Integer(num_val as i64))
            } else {
                Ok(SqlStorageValue::Float(num_val))
            }
        } else if let Some(bytes) = js_val.dyn_ref::<Uint8Array>() {
            Ok(SqlStorageValue::Blob(bytes.to_vec()))
        } else if let Some(buffer) = js_val.dyn_ref::<js_sys::ArrayBuffer>() {
            Ok(SqlStorageValue::Blob(buffer.to_vec()))
        } else {
            Err(format!("Unsupported SQL value: {js_val:?}").into())
        }
    }
}

pub fn exec(
    sql: &SqlStorage,
    query: &str,
    bindings: impl Into<Option<Vec<SqlStorageValue>>>,
) -> Result<SqlStorageCursor> {
    let bindings: Vec<JsValue> = bindings
        .into()
        .unwrap_or_default()
        .into_iter()
        .map(JsValue::from)
        .collect();
    sql.exec(query, &bindings)
}

pub fn to_array<T: DeserializeOwned>(cursor: &SqlStorageCursor) -> Result<Vec<T>> {
    cursor
        .to_array()?
        .into_iter()
        .map(|row| Ok(serde_wasm_bindgen::from_value(row)?))
        .collect()
}

/// `try_iter` keeps a throwing cursor as an `Err` item rather than rethrowing.
pub fn rows<T: DeserializeOwned>(cursor: &SqlStorageCursor) -> impl Iterator<Item = Result<T>> {
    cursor
        .iterator()
        .into_try_iter()
        .map(|row| Ok(serde_wasm_bindgen::from_value(row?)?))
}

pub fn raw(
    cursor: &SqlStorageCursor,
) -> Result<impl Iterator<Item = Result<Vec<SqlStorageValue>>>> {
    Ok(cursor.raw_typed::<js_sys::Array>().into_iter().map(|row| {
        row.iter()
            .map(SqlStorageValue::try_from)
            .collect::<Result<Vec<_>>>()
    }))
}

pub fn raw_row(row: &[SqlStorageValue]) -> Vec<String> {
    row.iter().map(|v| format!("{v:?}")).collect()
}
