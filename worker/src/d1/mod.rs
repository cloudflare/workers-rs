use std::fmt::Display;
use std::fmt::Formatter;
use std::iter::{once, Once};
use std::ops::Deref;
use std::result::Result as StdResult;

use js_sys::Array;
use js_sys::ArrayBuffer;
use js_sys::JsString;
use js_sys::Uint8Array;
use serde::Deserialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::types::D1Database as D1DatabaseSys;
use worker_sys::types::D1ExecResult;
use worker_sys::types::D1PreparedStatement as D1PreparedStatementSys;
use worker_sys::types::D1Result as D1ResultSys;

use crate::env::EnvBinding;
use crate::Error;
use crate::Result;

pub use serde_wasm_bindgen;

pub mod macros;

// A D1 Database.
pub struct D1Database(D1DatabaseSys);

unsafe impl Sync for D1Database {}
unsafe impl Send for D1Database {}

impl D1Database {
    /// Prepare a query statement from a query string.
    pub fn prepare<T: Into<String>>(&self, query: T) -> D1PreparedStatement {
        self.0.prepare(&query.into()).unwrap().into()
    }

    /// Dump the data in the database to a `Vec`.
    pub async fn dump(&self) -> Result<Vec<u8>> {
        let result = JsFuture::from(self.0.dump()?).await;
        let array_buffer = cast_to_d1_error(result)?;
        let array_buffer = array_buffer.dyn_into::<ArrayBuffer>()?;
        let array = Uint8Array::new(&array_buffer);
        Ok(array.to_vec())
    }

    /// Batch execute one or more statements against the database.
    ///
    /// Returns the results in the same order as the provided statements.
    pub async fn batch(&self, statements: Vec<D1PreparedStatement>) -> Result<Vec<D1Result>> {
        let statements = statements.into_iter().map(|s| s.0).collect::<Array>();
        let results = JsFuture::from(self.0.batch(statements)?).await;
        let results = cast_to_d1_error(results)?;
        let results = results.dyn_into::<Array>()?;
        let mut vec = Vec::with_capacity(results.length() as usize);
        for result in results.iter() {
            let result = result.unchecked_into::<D1ResultSys>();
            vec.push(D1Result(result));
        }
        Ok(vec)
    }

    /// Execute one or more queries directly against the database.
    ///
    /// The input can be one or multiple queries separated by `\n`.
    ///
    /// # Considerations
    ///
    /// This method can have poorer performance (prepared statements can be reused
    /// in some cases) and, more importantly, is less safe. Only use this
    /// method for maintenance and one-shot tasks (example: migration jobs).
    ///
    /// If an error occurs, an exception is thrown with the query and error
    /// messages, execution stops and further statements are not executed.
    pub async fn exec(&self, query: &str) -> Result<D1ExecResult> {
        let result = JsFuture::from(self.0.exec(query)?).await;
        let result = cast_to_d1_error(result)?;
        Ok(result.into())
    }
}

impl EnvBinding for D1Database {
    const TYPE_NAME: &'static str = "D1Database";

    // Workaround for Miniflare D1 Beta
    fn get(val: JsValue) -> Result<Self> {
        let obj = js_sys::Object::from(val);
        if obj.constructor().name() == Self::TYPE_NAME || obj.constructor().name() == "BetaDatabase"
        {
            Ok(obj.unchecked_into())
        } else {
            Err(format!(
                "Binding cannot be cast to the type {} from {}",
                Self::TYPE_NAME,
                obj.constructor().name()
            )
            .into())
        }
    }
}

impl JsCast for D1Database {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<D1DatabaseSys>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self(val.into())
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl From<D1Database> for JsValue {
    fn from(database: D1Database) -> Self {
        JsValue::from(database.0)
    }
}

impl AsRef<JsValue> for D1Database {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}

impl From<D1DatabaseSys> for D1Database {
    fn from(inner: D1DatabaseSys) -> Self {
        Self(inner)
    }
}

/// Possible argument types that can be bound to [`D1PreparedStatement`]
/// See https://developers.cloudflare.com/d1/build-with-d1/d1-client-api/#type-conversion
pub enum D1Type<'a> {
    Null,
    Real(f64),
    // I believe JS always casts to float. Documentation states it can accept up to 53 bits of signed precision
    // so I went with i32 here. https://developer.mozilla.org/en-US/docs/Web/JavaScript/Data_structures#number_type
    // D1 does not support `BigInt`
    Integer(i32),
    Text(&'a str),
    Boolean(bool),
    Blob(&'a [u8]),
}

/// A pre-computed argument for `bind_refs`.
///
/// Arguments must be converted to `JsValue` when bound. If you plan to
/// re-use the same argument multiple times, consider using a `D1PreparedArgument`
/// which does this once on construction.
pub struct D1PreparedArgument<'a> {
    value: &'a D1Type<'a>,
    js_value: JsValue,
}

impl<'a> D1PreparedArgument<'a> {
    pub fn new(value: &'a D1Type) -> D1PreparedArgument<'a> {
        Self {
            value,
            js_value: value.into(),
        }
    }
}

impl<'a> From<&'a D1Type<'a>> for JsValue {
    fn from(value: &'a D1Type<'a>) -> Self {
        match *value {
            D1Type::Null => JsValue::null(),
            D1Type::Real(f) => JsValue::from_f64(f),
            D1Type::Integer(i) => JsValue::from_f64(i as f64),
            D1Type::Text(s) => JsValue::from_str(s),
            D1Type::Boolean(b) => JsValue::from_bool(b),
            D1Type::Blob(a) => serde_wasm_bindgen::to_value(a).unwrap(),
        }
    }
}

impl<'a> Deref for D1PreparedArgument<'a> {
    type Target = D1Type<'a>;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a> IntoIterator for &'a D1Type<'a> {
    type Item = &'a D1Type<'a>;
    type IntoIter = Once<&'a D1Type<'a>>;
    /// Allows a single &D1Type to be passed to `bind_refs`, without placing it in an array.
    fn into_iter(self) -> Self::IntoIter {
        once(self)
    }
}

impl<'a> IntoIterator for &'a D1PreparedArgument<'a> {
    type Item = &'a D1PreparedArgument<'a>;
    type IntoIter = Once<&'a D1PreparedArgument<'a>>;
    /// Allows a single &D1PreparedArgument to be passed to `bind_refs`, without placing it in an array.
    fn into_iter(self) -> Self::IntoIter {
        once(self)
    }
}

pub trait D1Argument {
    fn js_value(&self) -> impl AsRef<JsValue>;
}

impl<'a> D1Argument for D1Type<'a> {
    fn js_value(&self) -> impl AsRef<JsValue> {
        Into::<JsValue>::into(self)
    }
}

impl<'a> D1Argument for D1PreparedArgument<'a> {
    fn js_value(&self) -> impl AsRef<JsValue> {
        &self.js_value
    }
}

// A D1 prepared query statement.
#[derive(Clone)]
pub struct D1PreparedStatement(D1PreparedStatementSys);

impl D1PreparedStatement {
    /// Bind one or more parameters to the statement.
    /// Consumes the old statement and returns a new statement with the bound parameters.
    ///
    /// D1 follows the SQLite convention for prepared statements parameter binding.
    ///
    /// # Considerations
    ///
    /// Supports Ordered (?NNNN) and Anonymous (?) parameters - named parameters are currently not supported.
    ///
    pub fn bind(self, values: &[JsValue]) -> Result<Self> {
        let array: Array = values.iter().collect::<Array>();

        match self.0.bind(array) {
            Ok(stmt) => Ok(D1PreparedStatement(stmt)),
            Err(err) => Err(Error::from(err)),
        }
    }

    /// Bind one or more parameters to the statement.
    /// Returns a new statement with the bound parameters, leaving the old statement available for reuse.
    pub fn bind_refs<'a, T, U: 'a>(&self, values: T) -> Result<Self>
    where
        T: IntoIterator<Item = &'a U>,
        U: D1Argument,
    {
        let array: Array = values.into_iter().map(|t| t.js_value()).collect::<Array>();

        match self.0.bind(array) {
            Ok(stmt) => Ok(D1PreparedStatement(stmt)),
            Err(err) => Err(Error::from(err)),
        }
    }

    /// Bind a batch of parameter values, returning a batch of prepared statements.
    /// Result can be passed to [`D1Database::batch`] to execute the statements.
    pub fn batch_bind<'a, U: 'a, T: 'a, V: 'a>(&self, values: T) -> Result<Vec<Self>>
    where
        T: IntoIterator<Item = U>,
        U: IntoIterator<Item = &'a V>,
        V: D1Argument,
    {
        values
            .into_iter()
            .map(|batch| self.bind_refs(batch))
            .collect()
    }

    /// Return the first row of results.
    ///
    /// If `col_name` is `Some`, returns that single value, otherwise returns the entire object.
    ///
    /// If the query returns no rows, then this will return `None`.
    ///
    /// If the query returns rows, but column does not exist, then this will return an `Err`.
    pub async fn first<T>(&self, col_name: Option<&str>) -> Result<Option<T>>
    where
        T: for<'a> Deserialize<'a>,
    {
        let result = JsFuture::from(self.0.first(col_name)?).await;
        let js_value = cast_to_d1_error(result)?;
        let value = serde_wasm_bindgen::from_value(js_value)?;
        Ok(value)
    }

    /// Executes a query against the database but only return metadata.
    pub async fn run(&self) -> Result<D1Result> {
        let result = JsFuture::from(self.0.run()?).await;
        let result = cast_to_d1_error(result)?;
        Ok(D1Result(result.into()))
    }

    /// Executes a query against the database and returns all rows and metadata.
    pub async fn all(&self) -> Result<D1Result> {
        let result = JsFuture::from(self.0.all()?).await?;
        Ok(D1Result(result.into()))
    }

    /// Executes a query against the database and returns a `Vec` of rows instead of objects.
    pub async fn raw<T>(&self) -> Result<Vec<Vec<T>>>
    where
        T: for<'a> Deserialize<'a>,
    {
        let result = JsFuture::from(self.0.raw()?).await;
        let result = cast_to_d1_error(result)?;
        let result = result.dyn_into::<Array>()?;
        let mut vec = Vec::with_capacity(result.length() as usize);
        for value in result.iter() {
            let value = serde_wasm_bindgen::from_value(value)?;
            vec.push(value);
        }
        Ok(vec)
    }

    /// Executes a query against the database and returns a `Vec` of JsValues.
    pub async fn raw_js_value(&self) -> Result<Vec<JsValue>> {
        let result = JsFuture::from(self.0.raw()?).await;
        let result = cast_to_d1_error(result)?;
        let array = result.dyn_into::<Array>()?;

        Ok(array.to_vec())
    }

    /// Returns the inner JsValue bindings object.
    pub fn inner(&self) -> &D1PreparedStatementSys {
        &self.0
    }
}

impl From<D1PreparedStatementSys> for D1PreparedStatement {
    fn from(inner: D1PreparedStatementSys) -> Self {
        Self(inner)
    }
}

// The result of a D1 query execution.
pub struct D1Result(D1ResultSys);

impl D1Result {
    /// Returns `true` if the result indicates a success, otherwise `false`.
    pub fn success(&self) -> bool {
        self.0.success().unwrap()
    }

    /// Return the error contained in this result.
    ///
    /// Returns `None` if the result indicates a success.
    pub fn error(&self) -> Option<String> {
        self.0.error().unwrap()
    }

    /// Retrieve the collection of result objects, or an `Err` if an error occurred.
    pub fn results<T>(&self) -> Result<Vec<T>>
    where
        T: for<'a> Deserialize<'a>,
    {
        if let Some(results) = self.0.results()? {
            let mut vec = Vec::with_capacity(results.length() as usize);
            for result in results.iter() {
                let result = serde_wasm_bindgen::from_value(result).unwrap();
                vec.push(result);
            }
            Ok(vec)
        } else {
            Ok(Vec::new())
        }
    }
}

#[derive(Clone)]
pub struct D1Error {
    inner: js_sys::Error,
}

impl D1Error {
    /// Gets the cause of the error specific to D1.
    pub fn cause(&self) -> String {
        if let Ok(cause) = self.inner.cause().dyn_into::<js_sys::Error>() {
            cause.message().into()
        } else {
            "unknown error".into()
        }
    }
}

impl std::fmt::Debug for D1Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cause = self.inner.cause();

        f.debug_struct("D1Error").field("cause", &cause).finish()
    }
}

impl Display for D1Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cause = self.inner.cause();
        let cause = JsString::from(cause);
        write!(f, "{}", cause)
    }
}

impl AsRef<js_sys::Error> for D1Error {
    fn as_ref(&self) -> &js_sys::Error {
        &self.inner
    }
}

impl AsRef<JsValue> for D1Error {
    fn as_ref(&self) -> &JsValue {
        &self.inner
    }
}

fn cast_to_d1_error<T>(result: StdResult<T, JsValue>) -> StdResult<T, crate::Error> {
    let err = match result {
        Ok(value) => return Ok(value),
        Err(err) => err,
    };

    let err: JsValue = match err.dyn_into::<js_sys::Error>() {
        Ok(err) => {
            let message: String = err.message().into();

            if message.starts_with("D1") {
                return Err(D1Error { inner: err }.into());
            };
            err.into()
        }
        Err(err) => err,
    };

    Err(err.into())
}
