use crate::{Env, Error, Request, Response, Result};
use async_trait::async_trait;
use edgeworker_sys::{Request as EdgeRequest, Response as EdgeResponse};
use js_sys::{Map, Object};
use serde::{Deserialize, Serialize};
use std::{future::Future, ops::Deref, result::Result as StdResult};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{future_to_promise, JsFuture};

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

impl ObjectStub {
    pub async fn fetch_with_request(&self, req: Request) -> Result<Response> {
        let promise = self.fetch_with_request_internal(req.inner());
        let response = JsFuture::from(promise).await?;
        Ok(response.dyn_into::<EdgeResponse>()?.into())
    }

    pub async fn fetch_with_str(&self, url: &str) -> Result<Response> {
        let promise = self.fetch_with_str_internal(url);
        let response = JsFuture::from(promise).await?;
        Ok(response.dyn_into::<EdgeResponse>()?.into())
    }
}

pub struct ObjectId<'a> {
    inner: JsObjectId,
    namespace: Option<&'a ObjectNamespace>,
}

impl ObjectId<'_> {
    pub fn get_stub(&self) -> Result<ObjectStub> {
        self.namespace
            .ok_or_else(|| JsValue::from("Cannot get stub from within a Durable Object"))
            .and_then(|n| n.get_internal(&self.inner))
            .map_err(Error::from)
    }
}

impl ObjectNamespace {
    // Get a Durable Object binding from the global namespace
    // if your build is configured with ES6 modules, use Env::get_binding instead
    // pub fn global(name: &str) -> Result<Self> {
    //     let global = js_sys::global();
    //     #[allow(unused_unsafe)]
    //     // Weird rust-analyzer bug is causing it to think Reflect::get is unsafe
    //     let class_binding = unsafe { js_sys::Reflect::get(&global, &JsValue::from(name))? };
    //     if class_binding.is_undefined() {
    //         Err(Error::JsError("namespace binding does not exist".into()))
    //     } else {
    //         Ok(class_binding.unchecked_into())
    //     }
    // }

    pub fn id_from_name(&self, name: &str) -> Result<ObjectId> {
        self.id_from_name_internal(name)
            .map_err(Error::from)
            .map(|id| ObjectId {
                inner: id,
                namespace: Some(self),
            })
    }

    pub fn id_from_string(&self, string: &str) -> Result<ObjectId> {
        self.id_from_string_internal(string)
            .map_err(Error::from)
            .map(|id| ObjectId {
                inner: id,
                namespace: Some(self),
            })
    }

    pub fn unique_id(&self) -> Result<ObjectId> {
        self.new_unique_id_internal()
            .map_err(Error::from)
            .map(|id| ObjectId {
                inner: id,
                namespace: Some(self),
            })
    }

    pub fn unique_id_with_jurisdiction(&self, jd: &str) -> Result<ObjectId> {
        let options = Object::new();
        #[allow(unused_unsafe)]
        // Weird rust-analyzer bug is causing it to think Reflect::set is unsafe
        unsafe {
            js_sys::Reflect::set(&options, &JsValue::from("jurisdiction"), &jd.into())?
        };
        self.new_unique_id_with_options_internal(&options)
            .map_err(Error::from)
            .map(|id| ObjectId {
                inner: id,
                namespace: Some(self),
            })
    }
}

impl State {
    pub fn id(&self) -> ObjectId<'_> {
        ObjectId {
            inner: self.id_internal(),
            namespace: None,
        }
    }

    // Just to improve visibility to code analysis tools
    pub fn storage(&self) -> Storage {
        self.storage_internal()
    }
}

impl Storage {
    pub async fn get<T: for<'a> Deserialize<'a>>(&self, key: &str) -> Result<T> {
        JsFuture::from(self.get_internal(key)?)
            .await
            .and_then(|val| {
                if val.is_undefined() {
                    Err(JsValue::from("No such value in storage."))
                } else {
                    val.into_serde().map_err(|e| JsValue::from(e.to_string()))
                }
            })
            .map_err(Error::from)
    }

    pub async fn get_multiple(&self, keys: Vec<impl Deref<Target = str>>) -> Result<Map> {
        let keys = self.get_multiple_internal(
            keys.into_iter()
                .map(|key| JsValue::from(key.deref()))
                .collect(),
        )?;
        let keys = JsFuture::from(keys).await?;
        keys.dyn_into::<Map>().map_err(Error::from)
    }

    pub async fn put<T: Serialize>(&mut self, key: &str, value: T) -> Result<()> {
        JsFuture::from(self.put_internal(key, JsValue::from_serde(&value)?)?)
            .await
            .map_err(Error::from)
            .map(|_| ())
    }

    // Each key-value pair in the serialized object will be added to the storage
    pub async fn put_multiple<T: Serialize>(&mut self, values: T) -> Result<()> {
        let values = JsValue::from_serde(&values)?;
        if !values.is_object() {
            return Err("Must pass in a struct type".to_string().into());
        }
        JsFuture::from(self.put_multiple_internal(values)?)
            .await
            .map_err(Error::from)
            .map(|_| ())
    }

    pub async fn delete(&mut self, key: &str) -> Result<bool> {
        let fut: JsFuture = self.delete_internal(key)?.into();
        fut.await
            .and_then(|jsv| {
                jsv.as_bool()
                    .ok_or_else(|| JsValue::from("Promise did not return bool"))
            })
            .map_err(Error::from)
    }

    pub async fn delete_multiple(&mut self, keys: Vec<impl Deref<Target = str>>) -> Result<usize> {
        let fut: JsFuture = self
            .delete_multiple_internal(
                keys.into_iter()
                    .map(|key| JsValue::from(key.deref()))
                    .collect(),
            )?
            .into();
        fut.await
            .and_then(|jsv| {
                jsv.as_f64()
                    .map(|f| f as usize)
                    .ok_or_else(|| JsValue::from("Promise did not return number"))
            })
            .map_err(Error::from)
    }

    pub async fn delete_all(&mut self) -> Result<()> {
        let fut: JsFuture = self.delete_all_internal()?.into();
        fut.await.map(|_| ()).map_err(Error::from)
    }

    pub async fn list(&self) -> Result<Map> {
        let fut: JsFuture = self.list_internal()?.into();
        fut.await
            .and_then(|jsv| jsv.dyn_into())
            .map_err(Error::from)
    }

    pub async fn list_with_options(&self, opts: ListOptions<'_>) -> Result<Map> {
        let fut: JsFuture = self
            .list_with_options_internal(JsValue::from_serde(&opts)?.into())?
            .into();
        fut.await
            .and_then(|jsv| jsv.dyn_into())
            .map_err(Error::from)
    }

    //This function doesn't work on stable yet because the wasm_bindgen `Closure` type is still nightly-gated
    #[allow(dead_code)]
    async fn transaction<F>(&mut self, closure: fn(Transaction) -> F) -> Result<()>
    where
        F: Future<Output = Result<()>> + 'static,
    {
        let mut clos = |t: Transaction| {
            future_to_promise(async move {
                closure(t)
                    .await
                    .map_err(JsValue::from)
                    .map(|_| JsValue::NULL)
            })
        };
        JsFuture::from(self.transaction_internal(&mut clos)?)
            .await
            .map_err(Error::from)
            .map(|_| ())
    }
}

impl Transaction {
    pub async fn get<T: for<'a> Deserialize<'a>>(&self, key: &str) -> Result<T> {
        JsFuture::from(self.get_internal(key)?)
            .await
            .and_then(|val| {
                if val.is_undefined() {
                    Err(JsValue::from("No such value in storage."))
                } else {
                    val.into_serde().map_err(|e| JsValue::from(e.to_string()))
                }
            })
            .map_err(Error::from)
    }

    pub async fn get_multiple(&self, keys: Vec<impl Deref<Target = str>>) -> Result<Map> {
        let keys = self.get_multiple_internal(
            keys.into_iter()
                .map(|key| JsValue::from(key.deref()))
                .collect(),
        )?;
        let keys = JsFuture::from(keys).await?;
        keys.dyn_into::<Map>().map_err(Error::from)
    }

    pub async fn put<T: Serialize>(&mut self, key: &str, value: T) -> Result<()> {
        JsFuture::from(self.put_internal(key, JsValue::from_serde(&value)?)?)
            .await
            .map_err(Error::from)
            .map(|_| ())
    }

    // Each key-value pair in the serialized object will be added to the storage
    pub async fn put_multiple<T: Serialize>(&mut self, values: T) -> Result<()> {
        let values = JsValue::from_serde(&values)?;
        if !values.is_object() {
            return Err("Must pass in a struct type".to_string().into());
        }
        JsFuture::from(self.put_multiple_internal(values)?)
            .await
            .map_err(Error::from)
            .map(|_| ())
    }

    pub async fn delete(&mut self, key: &str) -> Result<bool> {
        let fut: JsFuture = self.delete_internal(key)?.into();
        fut.await
            .and_then(|jsv| {
                jsv.as_bool()
                    .ok_or_else(|| JsValue::from("Promise did not return bool"))
            })
            .map_err(Error::from)
    }

    pub async fn delete_multiple(&mut self, keys: Vec<impl Deref<Target = str>>) -> Result<usize> {
        let fut: JsFuture = self
            .delete_multiple_internal(
                keys.into_iter()
                    .map(|key| JsValue::from(key.deref()))
                    .collect(),
            )?
            .into();
        fut.await
            .and_then(|jsv| {
                jsv.as_f64()
                    .map(|f| f as usize)
                    .ok_or_else(|| JsValue::from("Promise did not return number"))
            })
            .map_err(Error::from)
    }

    pub async fn delete_all(&mut self) -> Result<()> {
        let fut: JsFuture = self.delete_all_internal()?.into();
        fut.await.map(|_| ()).map_err(Error::from)
    }

    pub async fn list(&self) -> Result<Map> {
        let fut: JsFuture = self.list_internal()?.into();
        fut.await
            .and_then(|jsv| jsv.dyn_into())
            .map_err(Error::from)
    }

    pub async fn list_with_options(&self, opts: ListOptions<'_>) -> Result<Map> {
        let fut: JsFuture = self
            .list_with_options_internal(JsValue::from_serde(&opts)?.into())?
            .into();
        fut.await
            .and_then(|jsv| jsv.dyn_into())
            .map_err(Error::from)
    }

    pub fn rollback(&mut self) -> Result<()> {
        self.rollback_internal().map_err(Error::from)
    }
}

#[derive(Serialize)]
pub struct ListOptions<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prefix: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reverse: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
}

impl<'a> ListOptions<'a> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
            prefix: None,
            reverse: None,
            limit: None,
        }
    }

    pub fn start(mut self, val: &'a str) -> Self {
        self.start = Some(val);
        self
    }

    pub fn end(mut self, val: &'a str) -> Self {
        self.end = Some(val);
        self
    }

    pub fn prefix(mut self, val: &'a str) -> Self {
        self.prefix = Some(val);
        self
    }

    pub fn reverse(mut self, val: bool) -> Self {
        self.reverse = Some(val);
        self
    }

    pub fn limit(mut self, val: usize) -> Self {
        self.limit = Some(val);
        self
    }
}

impl crate::EnvBinding for ObjectNamespace {
    const TYPE_NAME: &'static str = "DurableObjectNamespace";
}

#[async_trait(?Send)]
pub trait DurableObject {
    fn constructor(state: State, env: Env) -> Self;
    async fn fetch(&mut self, req: Request) -> Result<Response>;
}
