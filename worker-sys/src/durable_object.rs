use std::result::Result as StdResult;

use crate::Request as EdgeRequest;

use js_sys::JsString;
use wasm_bindgen::{closure::Closure, prelude::*};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends = ::js_sys::Object, js_name = DurableObjectId)]
    pub type JsObjectId;

    #[wasm_bindgen(method, js_class = "JsObjectId", js_name = toString)]
    pub fn to_string(this: &JsObjectId) -> JsString;

    #[wasm_bindgen (extends = ::js_sys::Object, js_name = DurableObject)]
    pub type ObjectStub;

    #[wasm_bindgen (extends = ::js_sys::Object, js_name = DurableObjectNamespace)]
    pub type ObjectNamespace;

    #[wasm_bindgen (extends = ::js_sys::Object, js_name = DurableObjectState)]
    pub type ObjectState;

    #[wasm_bindgen(method, getter, js_class = "DurableObjectState", js_name = id)]
    pub fn id_internal(this: &ObjectState) -> JsObjectId;

    #[wasm_bindgen(method, getter, js_class = "DurableObjectState", js_name = storage)]
    pub fn storage_internal(this: &ObjectState) -> ObjectStorage;

    #[wasm_bindgen (catch, method, js_class = "DurableObjectNamespace", js_name = idFromName)]
    pub fn id_from_name_internal(
        this: &ObjectNamespace,
        name: &str,
    ) -> StdResult<JsObjectId, JsValue>;

    #[wasm_bindgen (catch, method, js_class = "ObjectNamespace", js_name = idFromString)]
    pub fn id_from_string_internal(
        this: &ObjectNamespace,
        string: &str,
    ) -> StdResult<JsObjectId, JsValue>;

    #[wasm_bindgen (catch, method, js_class = "DurableObjectNamespace", js_name = newUniqueId)]
    pub fn new_unique_id_internal(this: &ObjectNamespace) -> StdResult<JsObjectId, JsValue>;

    #[wasm_bindgen (catch, method, js_class = "DurableObjectNamespace", js_name = newUniqueId)]
    pub fn new_unique_id_with_options_internal(
        this: &ObjectNamespace,
        options: &JsValue,
    ) -> StdResult<JsObjectId, JsValue>;

    #[wasm_bindgen (catch, method, js_class = "DurableObjectNamespace", js_name = get)]
    pub fn get_internal(this: &ObjectNamespace, id: &JsObjectId) -> StdResult<ObjectStub, JsValue>;

    #[wasm_bindgen (method, js_class = "DurableObject", js_name = fetch)]
    pub fn fetch_with_request_internal(this: &ObjectStub, req: &EdgeRequest) -> ::js_sys::Promise;

    #[wasm_bindgen (method, js_class = "DurableObject", js_name = fetch)]
    pub fn fetch_with_str_internal(this: &ObjectStub, url: &str) -> ::js_sys::Promise;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends = ::js_sys::Object, js_name = DurableObjectStorage)]
    pub type ObjectStorage;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = get)]
    pub fn get_internal(this: &ObjectStorage, key: &str) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = get)]
    pub fn get_multiple_internal(
        this: &ObjectStorage,
        keys: Vec<JsValue>,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = put)]
    pub fn put_internal(
        this: &ObjectStorage,
        key: &str,
        value: JsValue,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = put)]
    pub fn put_multiple_internal(
        this: &ObjectStorage,
        value: JsValue,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = delete)]
    pub fn delete_internal(
        this: &ObjectStorage,
        key: &str,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = delete)]
    pub fn delete_multiple_internal(
        this: &ObjectStorage,
        keys: Vec<JsValue>,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = deleteAll)]
    pub fn delete_all_internal(this: &ObjectStorage) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = list)]
    pub fn list_internal(this: &ObjectStorage) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = list)]
    pub fn list_with_options_internal(
        this: &ObjectStorage,
        options: ::js_sys::Object,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = transaction)]
    pub fn transaction_internal(
        this: &ObjectStorage,
        closure: &Closure<dyn FnMut(ObjectTransaction)>,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = getAlarm)]
    pub fn get_alarm_internal(
        this: &ObjectStorage,
        options: ::js_sys::Object,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = setAlarm)]
    pub fn set_alarm_internal(
        this: &ObjectStorage,
        scheduled_time: ::js_sys::Date,
        options: ::js_sys::Object,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectStorage", js_name = deleteAlarm)]
    pub fn delete_alarm_internal(
        this: &ObjectStorage,
        options: ::js_sys::Object,
    ) -> StdResult<::js_sys::Promise, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = ::js_sys::Object, js_name = DurableObjectTransaction)]
    pub type ObjectTransaction;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = get)]
    pub fn get_internal(
        this: &ObjectTransaction,
        key: &str,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = get)]
    pub fn get_multiple_internal(
        this: &ObjectTransaction,
        keys: Vec<JsValue>,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = put)]
    pub fn put_internal(
        this: &ObjectTransaction,
        key: &str,
        value: JsValue,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = put)]
    pub fn put_multiple_internal(
        this: &ObjectTransaction,
        value: JsValue,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = delete)]
    pub fn delete_internal(
        this: &ObjectTransaction,
        key: &str,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = delete)]
    pub fn delete_multiple_internal(
        this: &ObjectTransaction,
        keys: Vec<JsValue>,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = deleteAll)]
    pub fn delete_all_internal(this: &ObjectTransaction) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = list)]
    pub fn list_internal(this: &ObjectTransaction) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = list)]
    pub fn list_with_options_internal(
        this: &ObjectTransaction,
        options: ::js_sys::Object,
    ) -> StdResult<::js_sys::Promise, JsValue>;

    #[wasm_bindgen(catch, method, js_class = "DurableObjectTransaction", js_name = rollback)]
    pub fn rollback_internal(this: &ObjectTransaction) -> StdResult<(), JsValue>;
}
