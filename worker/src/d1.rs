use js_sys::Array;
use js_sys::ArrayBuffer;
use js_sys::Uint8Array;
use serde::de::Deserialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::d1::D1Database as D1DatabaseSys;
use worker_sys::d1::D1ExecResult;
use worker_sys::d1::D1PreparedStatement as D1PreparedStatementSys;
use worker_sys::d1::D1Result as D1ResultSys;

use crate::env::EnvBinding;
use crate::Error;
use crate::Result;

// A D1 Database.
pub struct D1Database(D1DatabaseSys);

impl D1Database {
    /// Prepare a query statement from a query string.
    pub fn prepare<T: Into<String>>(&self, query: T) -> D1PreparedStatement {
        self.0.prepare(&query.into()).into()
    }

    /// Dump the data in the database to a `Vec`.
    pub async fn dump(&self) -> Result<Vec<u8>> {
        let array_buffer = JsFuture::from(self.0.dump()).await?;
        let array_buffer = array_buffer.dyn_into::<ArrayBuffer>()?;
        let array = Uint8Array::new(&array_buffer);
        let mut vec = Vec::with_capacity(array.length() as usize);
        array.copy_to(&mut vec);
        Ok(vec)
    }

    /// Batch execute one or more statements against the database.
    ///
    /// Returns the results in the same order as the provided statements.
    pub async fn batch(&self, statements: Vec<D1PreparedStatement>) -> Result<Vec<D1Result>> {
        let statements = statements.into_iter().map(|s| s.0).collect::<Array>();
        let results = JsFuture::from(self.0.batch(statements)).await?;
        let results = results.dyn_into::<Array>()?;
        let mut vec = Vec::with_capacity(results.length() as usize);
        for result in results.iter() {
            let result = result.dyn_into::<D1ResultSys>()?;
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
        let result = JsFuture::from(self.0.exec(query)).await?;
        Ok(result.into())
    }
}

impl EnvBinding for D1Database {
    const TYPE_NAME: &'static str = "D1Database";
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

// A D1 prepared query statement.
pub struct D1PreparedStatement(D1PreparedStatementSys);

impl D1PreparedStatement {
    /// Bind one or more parameters to the statement. Returns a new statement object.
    ///
    /// D1 follows the SQLite convention for prepared statements parameter binding.
    ///
    /// # Considerations
    ///
    /// Supports Ordered (?NNNN) and Anonymous (?) parameters - named parameters are currently not supported.
    ///
    pub fn bind<T>(&self, values: &[&T]) -> Result<Self>
    where
        T: serde::ser::Serialize + ?Sized,
    {
        let mut params = Vec::new();
        for value in values.iter() {
            let res = serde_wasm_bindgen::to_value(value)?;
            params.push(res);
        }

        let array: Array = params.into_iter().collect::<Array>();

        match self.0.bind(array) {
            Ok(stmt) => Ok(D1PreparedStatement(stmt)),
            Err(err) => Err(Error::from(err)),
        }
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
        let js_value = JsFuture::from(self.0.first(col_name)).await?;
        let value = serde_wasm_bindgen::from_value(js_value)?;
        Ok(value)
    }

    /// Executes a query against the database but only return metadata.
    pub async fn run(&self) -> Result<D1Result> {
        let result = JsFuture::from(self.0.run()).await?;
        Ok(D1Result(result.into()))
    }

    /// Executes a query against the database and returns all rows and metadata.
    pub async fn all(&self) -> Result<D1Result> {
        let result = JsFuture::from(self.0.all()).await?;
        Ok(D1Result(result.into()))
    }

    /// Executes a query against the database and returns a `Vec` of rows instead of objects.
    pub async fn raw<T>(&self) -> Result<Vec<Vec<T>>>
    where
        T: for<'a> Deserialize<'a>,
    {
        let result = JsFuture::from(self.0.raw()).await?;
        let result = result.dyn_into::<Array>()?;
        let mut vec = Vec::with_capacity(result.length() as usize);
        for value in result.iter() {
            let value = serde_wasm_bindgen::from_value(value)?;
            vec.push(value);
        }
        Ok(vec)
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
        self.0.success()
    }

    /// Return the error contained in this result.
    ///
    /// Returns `None` if the result indicates a success.
    pub fn error(&self) -> Option<String> {
        self.0.error()
    }

    /// Retrieve the collection of result objects, or an `Err` if an error occurred.
    pub fn results<T>(&self) -> Result<Vec<T>>
    where
        T: for<'a> Deserialize<'a>,
    {
        if let Some(results) = self.0.results() {
            let mut vec = Vec::with_capacity(results.length() as usize);
            for result in results.iter() {
                let result = serde_wasm_bindgen::from_value(result)?;
                vec.push(result);
            }
            Ok(vec)
        } else {
            Ok(Vec::new())
        }
    }
}
