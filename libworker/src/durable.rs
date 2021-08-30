// use std::future::Future;
use std::ops::Deref;

use crate::{
    env::{Env, EnvBinding},
    error::Error,
    request::Request,
    response::Response,
    Result,
};

use async_trait::async_trait;
use edgeworker_sys::{
    durable_object::{
        JsObjectId, ObjectNamespace as EdgeObjectNamespace, ObjectState, ObjectStorage, ObjectStub,
        ObjectTransaction,
    },
    Response as EdgeResponse,
};
use js_sys::{Map, Object};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, JsCast};
// use wasm_bindgen_futures::future_to_promise;
use wasm_bindgen_futures::JsFuture;

pub struct Stub {
    inner: ObjectStub,
}

impl Stub {
    pub async fn fetch_with_request(&self, req: Request) -> Result<Response> {
        let promise = self.inner.fetch_with_request_internal(req.inner());
        let response = JsFuture::from(promise).await?;
        Ok(response.dyn_into::<EdgeResponse>()?.into())
    }

    pub async fn fetch_with_str(&self, url: &str) -> Result<Response> {
        let promise = self.inner.fetch_with_str_internal(url);
        let response = JsFuture::from(promise).await?;
        Ok(response.dyn_into::<EdgeResponse>()?.into())
    }
}

pub struct ObjectNamespace {
    inner: EdgeObjectNamespace,
}

pub struct ObjectId<'a> {
    inner: JsObjectId,
    namespace: Option<&'a ObjectNamespace>,
}

impl ObjectId<'_> {
    pub fn get_stub(&self) -> Result<Stub> {
        self.namespace
            .ok_or_else(|| JsValue::from("Cannot get stub from within a Durable Object"))
            .and_then(|n| {
                Ok(Stub {
                    inner: n.inner.get_internal(&self.inner)?,
                })
            })
            .map_err(Error::from)
    }
}

impl ObjectNamespace {
    pub fn id_from_name(&self, name: &str) -> Result<ObjectId> {
        self.inner
            .id_from_name_internal(name)
            .map_err(Error::from)
            .map(|id| ObjectId {
                inner: id,
                namespace: Some(self),
            })
    }

    pub fn id_from_string(&self, string: &str) -> Result<ObjectId> {
        self.inner
            .id_from_string_internal(string)
            .map_err(Error::from)
            .map(|id| ObjectId {
                inner: id,
                namespace: Some(self),
            })
    }

    pub fn unique_id(&self) -> Result<ObjectId> {
        self.inner
            .new_unique_id_internal()
            .map_err(Error::from)
            .map(|id| ObjectId {
                inner: id,
                namespace: Some(self),
            })
    }

    pub fn unique_id_with_jurisdiction(&self, jd: &str) -> Result<ObjectId> {
        let options = Object::new();
        #[allow(unused_unsafe)]
        unsafe {
            js_sys::Reflect::set(&options, &JsValue::from("jurisdiction"), &jd.into())?
        };
        self.inner
            .new_unique_id_with_options_internal(&options)
            .map_err(Error::from)
            .map(|id| ObjectId {
                inner: id,
                namespace: Some(self),
            })
    }
}

pub struct State {
    inner: ObjectState,
}

impl State {
    pub fn id(&self) -> ObjectId<'_> {
        ObjectId {
            inner: self.inner.id_internal(),
            namespace: None,
        }
    }

    pub fn storage(&self) -> Storage {
        Storage {
            inner: self.inner.storage_internal(),
        }
    }

    // needs to be accessed by the `durable_object` macro in a conversion step
    pub fn _inner(self) -> ObjectState {
        self.inner
    }
}

impl From<ObjectState> for State {
    fn from(o: ObjectState) -> Self {
        Self { inner: o }
    }
}

pub struct Storage {
    inner: ObjectStorage,
}

impl Storage {
    pub async fn get<T: for<'a> Deserialize<'a>>(&self, key: &str) -> Result<T> {
        JsFuture::from(self.inner.get_internal(key)?)
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
        let keys = self.inner.get_multiple_internal(
            keys.into_iter()
                .map(|key| JsValue::from(key.deref()))
                .collect(),
        )?;
        let keys = JsFuture::from(keys).await?;
        keys.dyn_into::<Map>().map_err(Error::from)
    }

    pub async fn put<T: Serialize>(&mut self, key: &str, value: T) -> Result<()> {
        JsFuture::from(self.inner.put_internal(key, JsValue::from_serde(&value)?)?)
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
        JsFuture::from(self.inner.put_multiple_internal(values)?)
            .await
            .map_err(Error::from)
            .map(|_| ())
    }

    pub async fn delete(&mut self, key: &str) -> Result<bool> {
        let fut: JsFuture = self.inner.delete_internal(key)?.into();
        fut.await
            .and_then(|jsv| {
                jsv.as_bool()
                    .ok_or_else(|| JsValue::from("Promise did not return bool"))
            })
            .map_err(Error::from)
    }

    pub async fn delete_multiple(&mut self, keys: Vec<impl Deref<Target = str>>) -> Result<usize> {
        let fut: JsFuture = self
            .inner
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
        let fut: JsFuture = self.inner.delete_all_internal()?.into();
        fut.await.map(|_| ()).map_err(Error::from)
    }

    pub async fn list(&self) -> Result<Map> {
        let fut: JsFuture = self.inner.list_internal()?.into();
        fut.await
            .and_then(|jsv| jsv.dyn_into())
            .map_err(Error::from)
    }

    pub async fn list_with_options(&self, opts: ListOptions<'_>) -> Result<Map> {
        let fut: JsFuture = self
            .inner
            .list_with_options_internal(JsValue::from_serde(&opts)?.into())?
            .into();
        fut.await
            .and_then(|jsv| jsv.dyn_into())
            .map_err(Error::from)
    }

    // TODO(nilslice): follow up with runtime team on transaction API in general
    // This function doesn't work on stable yet because the wasm_bindgen `Closure` type is still nightly-gated
    // #[allow(dead_code)]
    // async fn transaction<F>(&mut self, closure: fn(Transaction) -> F) -> Result<()>
    // where
    //     F: Future<Output = Result<()>> + 'static,
    // {
    //     let mut clos = |t: Transaction| {
    //         future_to_promise(async move {
    //             closure(t)
    //                 .await
    //                 .map_err(JsValue::from)
    //                 .map(|_| JsValue::NULL)
    //         })
    //     };
    //     JsFuture::from(self.inner.transaction_internal(&mut clos)?)
    //         .await
    //         .map_err(Error::from)
    //         .map(|_| ())
    // }
}

struct Transaction {
    inner: ObjectTransaction,
}

impl Transaction {
    async fn get<T: for<'a> Deserialize<'a>>(&self, key: &str) -> Result<T> {
        JsFuture::from(self.inner.get_internal(key)?)
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

    async fn get_multiple(&self, keys: Vec<impl Deref<Target = str>>) -> Result<Map> {
        let keys = self.inner.get_multiple_internal(
            keys.into_iter()
                .map(|key| JsValue::from(key.deref()))
                .collect(),
        )?;
        let keys = JsFuture::from(keys).await?;
        keys.dyn_into::<Map>().map_err(Error::from)
    }

    async fn put<T: Serialize>(&mut self, key: &str, value: T) -> Result<()> {
        JsFuture::from(self.inner.put_internal(key, JsValue::from_serde(&value)?)?)
            .await
            .map_err(Error::from)
            .map(|_| ())
    }

    // Each key-value pair in the serialized object will be added to the storage
    async fn put_multiple<T: Serialize>(&mut self, values: T) -> Result<()> {
        let values = JsValue::from_serde(&values)?;
        if !values.is_object() {
            return Err("Must pass in a struct type".to_string().into());
        }
        JsFuture::from(self.inner.put_multiple_internal(values)?)
            .await
            .map_err(Error::from)
            .map(|_| ())
    }

    async fn delete(&mut self, key: &str) -> Result<bool> {
        let fut: JsFuture = self.inner.delete_internal(key)?.into();
        fut.await
            .and_then(|jsv| {
                jsv.as_bool()
                    .ok_or_else(|| JsValue::from("Promise did not return bool"))
            })
            .map_err(Error::from)
    }

    async fn delete_multiple(&mut self, keys: Vec<impl Deref<Target = str>>) -> Result<usize> {
        let fut: JsFuture = self
            .inner
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

    async fn delete_all(&mut self) -> Result<()> {
        let fut: JsFuture = self.inner.delete_all_internal()?.into();
        fut.await.map(|_| ()).map_err(Error::from)
    }

    async fn list(&self) -> Result<Map> {
        let fut: JsFuture = self.inner.list_internal()?.into();
        fut.await
            .and_then(|jsv| jsv.dyn_into())
            .map_err(Error::from)
    }

    async fn list_with_options(&self, opts: ListOptions<'_>) -> Result<Map> {
        let fut: JsFuture = self
            .inner
            .list_with_options_internal(JsValue::from_serde(&opts)?.into())?
            .into();
        fut.await
            .and_then(|jsv| jsv.dyn_into())
            .map_err(Error::from)
    }

    fn rollback(&mut self) -> Result<()> {
        self.inner.rollback_internal().map_err(Error::from)
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

impl EnvBinding for ObjectNamespace {
    const TYPE_NAME: &'static str = "DurableObjectNamespace";
}

impl JsCast for ObjectNamespace {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<EdgeObjectNamespace>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self { inner: val.into() }
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl From<ObjectNamespace> for JsValue {
    fn from(ns: ObjectNamespace) -> Self {
        JsValue::from(ns.inner)
    }
}

impl AsRef<JsValue> for ObjectNamespace {
    fn as_ref(&self) -> &JsValue {
        &self.inner
    }
}

#[async_trait(?Send)]
pub trait DurableObject {
    fn new(state: State, env: Env) -> Self;
    async fn fetch(&mut self, req: Request) -> Result<Response>;
}
