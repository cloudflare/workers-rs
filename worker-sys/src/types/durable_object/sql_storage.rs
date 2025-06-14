use js_sys::{Array, Iterator as JsIterator, Object};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Object)]
    #[derive(Clone)]
    pub type SqlStorage;

    /// Returns the on-disk size of the SQLite database, in bytes.
    #[wasm_bindgen(method, getter, js_name = databaseSize)]
    pub fn database_size(this: &SqlStorage) -> f64;

    /// Execute a SQL statement (optionally with bindings) and return a cursor over the results.
    ///
    /// The JavaScript definition of `exec` is variadic, taking a SQL string followed by an
    /// arbitrary number of binding parameters.  In Rust we accept the bindings packed into an
    /// `Array` – callers can construct the array (or use the helper provided in the higher-level
    /// `worker` crate).
    #[wasm_bindgen(structural, method, catch, variadic, js_class = SqlStorage, js_name = exec)]
    pub fn exec(
        this: &SqlStorage,
        query: &str,
        bindings: Array,
    ) -> Result<SqlStorageCursor, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Object)]
    #[derive(Clone)]
    pub type SqlStorageCursor;

    /// JavaScript `Iterator.next()` implementation.  Returns an object with `{ done, value }`.
    #[wasm_bindgen(method, js_name = next)]
    pub fn next(this: &SqlStorageCursor) -> Object;

    /// Convert the remaining rows into an array of objects.
    #[wasm_bindgen(method, js_name = toArray)]
    pub fn to_array(this: &SqlStorageCursor) -> Array;

    /// Returns the single row if exactly one row exists, otherwise throws.
    #[wasm_bindgen(method)]
    pub fn one(this: &SqlStorageCursor) -> JsValue;

    /// Returns an iterator where each row is an array rather than an object.
    #[wasm_bindgen(method)]
    pub fn raw(this: &SqlStorageCursor) -> JsIterator;

    /// Column names in the order they appear in `raw()` row arrays.
    #[wasm_bindgen(method, getter, js_name = columnNames)]
    pub fn column_names(this: &SqlStorageCursor) -> Array;

    /// Rows read so far.
    #[wasm_bindgen(method, getter, js_name = rowsRead)]
    pub fn rows_read(this: &SqlStorageCursor) -> f64;

    /// Rows written so far.
    #[wasm_bindgen(method, getter, js_name = rowsWritten)]
    pub fn rows_written(this: &SqlStorageCursor) -> f64;
}
