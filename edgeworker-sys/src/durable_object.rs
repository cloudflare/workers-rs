use std::result::Result as StdResult;

use crate::Request as EdgeRequest;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends = ::js_sys::Object, js_name = DurableObjectId)]
    type JsObjectId;

    #[wasm_bindgen (extends = ::js_sys::Object, js_name = DurableObject)]
    pub type ObjectStub;

    #[wasm_bindgen (extends = ::js_sys::Object, js_name = DurableObjectNamespace)]
    pub type ObjectNamespace;

    #[wasm_bindgen (extends = ::js_sys::Object, js_name = DurableObjectState)]
    pub type State;

    #[wasm_bindgen(method, getter, js_class = "DurableObjectState", js_name = id)]
    fn id_internal(this: &State) -> JsObjectId;

    #[wasm_bindgen(method, getter, js_class = "DurableObjectState", js_name = storage)]
    fn storage_internal(this: &State) -> Storage;

    #[wasm_bindgen (catch, method, js_class = "DurableObjectNamespace", js_name = idFromName)]
    fn id_from_name_internal(this: &ObjectNamespace, name: &str) -> StdResult<JsObjectId, JsValue>;

    #[wasm_bindgen (catch, method, js_class = "ObjectNamespace", js_name = idFromString)]
    fn id_from_string_internal(
        this: &ObjectNamespace,
        string: &str,
    ) -> StdResult<JsObjectId, JsValue>;

    #[wasm_bindgen (catch, method, js_class = "DurableObjectNamespace", js_name = newUniqueId)]
    fn new_unique_id_internal(this: &ObjectNamespace) -> StdResult<JsObjectId, JsValue>;

    #[wasm_bindgen (catch, method, js_class = "DurableObjectNamespace", js_name = newUniqueId)]
    fn new_unique_id_with_options_internal(
        this: &ObjectNamespace,
        options: &JsValue,
    ) -> StdResult<JsObjectId, JsValue>;

    #[wasm_bindgen (catch, method, js_class = "DurableObjectNamespace", js_name = get)]
    fn get_internal(this: &ObjectNamespace, id: &JsObjectId) -> StdResult<ObjectStub, JsValue>;

    #[wasm_bindgen (method, js_class = "DurableObject", js_name = fetch)]
    fn fetch_with_request_internal(this: &ObjectStub, req: &EdgeRequest) -> ::js_sys::Promise;

    #[wasm_bindgen (method, js_class = "DurableObject", js_name = fetch)]
    fn fetch_with_str_internal(this: &ObjectStub, url: &str) -> ::js_sys::Promise;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends = ::js_sys::Object, js_name = DurableObjectStorage)]
    pub type Storage;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = get)]
    fn get_internal(this: &Storage, key: &str) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = get)]
    fn get_multiple_internal(
        this: &Storage,
        keys: Vec<JsValue>,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = put)]
    fn put_internal(
        this: &Storage,
        key: &str,
        value: JsValue,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = put)]
    fn put_multiple_internal(
        this: &Storage,
        value: JsValue,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = delete)]
    fn delete_internal(this: &Storage, key: &str) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = delete)]
    fn delete_multiple_internal(
        this: &Storage,
        keys: Vec<JsValue>,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = deleteAll)]
    fn delete_all_internal(this: &Storage) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = list)]
    fn list_internal(this: &Storage) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = list)]
    fn list_with_options_internal(
        this: &Storage,
        options: ::js_sys::Object,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = transaction)]
    fn transaction_internal(
        this: &Storage,
        closure: &mut dyn FnMut(Transaction) -> ::js_sys::Promise,
    ) -> StdResult<::js_sys::Promise, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = ::js_sys::Object, js_name = DurableObjectTransaction)]
    pub type Transaction;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = get)]
    fn get_internal(this: &Transaction, key: &str) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = get)]
    fn get_multiple_internal(
        this: &Transaction,
        keys: Vec<JsValue>,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = put)]
    fn put_internal(
        this: &Transaction,
        key: &str,
        value: JsValue,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = put)]
    fn put_multiple_internal(
        this: &Transaction,
        value: JsValue,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = delete)]
    fn delete_internal(this: &Transaction, key: &str) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = delete)]
    fn delete_multiple_internal(
        this: &Transaction,
        keys: Vec<JsValue>,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = deleteAll)]
    fn delete_all_internal(this: &Transaction) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = list)]
    fn list_internal(this: &Transaction) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = list)]
    fn list_with_options_internal(
        this: &Transaction,
        options: ::js_sys::Object,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = rollback)]
    fn rollback_internal(this: &Transaction) -> StdResult<(), JsValue>;
}
