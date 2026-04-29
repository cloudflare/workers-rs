#[allow(unused_imports)]
use js_sys::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type SyncKvStorage;

    /// Retrieves the value associated with the given key.
    ///
    /// Returns `undefined` if the key does not exist.
    #[wasm_bindgen(method)]
    pub fn get(this: &SyncKvStorage, key: &str) -> JsValue;
    /// Retrieves the value associated with the given key.
    ///
    /// Returns `undefined` if the key does not exist.
    #[wasm_bindgen(method, catch, js_name = "get")]
    pub fn try_get(this: &SyncKvStorage, key: &str) -> Result<JsValue, JsValue>;

    /// Stores the value at the given key.
    #[wasm_bindgen(method)]
    pub fn put(this: &SyncKvStorage, key: &str, value: &JsValue);
    /// Stores the value at the given key.
    #[wasm_bindgen(method, catch, js_name = "put")]
    pub fn try_put(this: &SyncKvStorage, key: &str, value: &JsValue) -> Result<(), JsValue>;

    /// Deletes the key. Returns `true` if the key existed and was removed.
    #[wasm_bindgen(method)]
    pub fn delete(this: &SyncKvStorage, key: &str) -> bool;
    /// Deletes the key. Returns `true` if the key existed and was removed.
    #[wasm_bindgen(method, catch, js_name = "delete")]
    pub fn try_delete(this: &SyncKvStorage, key: &str) -> Result<bool, JsValue>;

    /// Returns an iterable of `[key, value]` pairs in ascending lexicographic order.
    #[wasm_bindgen(method)]
    pub fn list(this: &SyncKvStorage) -> JsValue;
    /// Returns an iterable of `[key, value]` pairs in ascending lexicographic order.
    #[wasm_bindgen(method, catch, js_name = "list")]
    pub fn try_list(this: &SyncKvStorage) -> Result<JsValue, JsValue>;
    /// Returns an iterable of `[key, value]` pairs filtered by the given options.
    #[wasm_bindgen(method, js_name = "list")]
    pub fn list_with_options(this: &SyncKvStorage, options: &SyncKvListOptions) -> JsValue;
    /// Returns an iterable of `[key, value]` pairs filtered by the given options.
    #[wasm_bindgen(method, catch, js_name = "list")]
    pub fn try_list_with_options(
        this: &SyncKvStorage,
        options: &SyncKvListOptions,
    ) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type SyncKvListOptions;

    /// Key at which the list results should start, inclusive.
    #[wasm_bindgen(method, getter)]
    pub fn start(this: &SyncKvListOptions) -> Option<String>;
    #[wasm_bindgen(method, setter)]
    pub fn set_start(this: &SyncKvListOptions, val: &str);

    /// Key at which the list results should start, exclusive. Cannot be used
    /// simultaneously with `start`.
    #[wasm_bindgen(method, getter, js_name = "startAfter")]
    pub fn start_after(this: &SyncKvListOptions) -> Option<String>;
    #[wasm_bindgen(method, setter, js_name = "startAfter")]
    pub fn set_start_after(this: &SyncKvListOptions, val: &str);

    /// Key at which the list results should end, exclusive.
    #[wasm_bindgen(method, getter)]
    pub fn end(this: &SyncKvListOptions) -> Option<String>;
    #[wasm_bindgen(method, setter)]
    pub fn set_end(this: &SyncKvListOptions, val: &str);

    /// Restricts results to only include key-value pairs whose keys begin with the prefix.
    #[wasm_bindgen(method, getter)]
    pub fn prefix(this: &SyncKvListOptions) -> Option<String>;
    #[wasm_bindgen(method, setter)]
    pub fn set_prefix(this: &SyncKvListOptions, val: &str);

    /// If true, return results in descending lexicographic order.
    #[wasm_bindgen(method, getter)]
    pub fn reverse(this: &SyncKvListOptions) -> Option<bool>;
    #[wasm_bindgen(method, setter)]
    pub fn set_reverse(this: &SyncKvListOptions, val: bool);

    /// Maximum number of key-value pairs to return.
    #[wasm_bindgen(method, getter)]
    pub fn limit(this: &SyncKvListOptions) -> Option<f64>;
    #[wasm_bindgen(method, setter)]
    pub fn set_limit(this: &SyncKvListOptions, val: f64);
}

impl SyncKvListOptions {
    /// Create an empty `SyncKvListOptions` JS object.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        use wasm_bindgen::JsCast;
        JsCast::unchecked_into(js_sys::Object::new())
    }
}
