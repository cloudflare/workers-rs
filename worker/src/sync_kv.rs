use core::fmt;
use std::marker::PhantomData;

use serde::{Serialize, de::DeserializeOwned};
use serde_wasm_bindgen as swb;
use wasm_bindgen::JsCast as _;

use crate::{Error, ListOptions, Result};

#[derive(Clone)]
pub struct SyncKvStorage {
    inner: worker_sys::types::SyncKvStorage,
}

unsafe impl Send for SyncKvStorage {}
unsafe impl Sync for SyncKvStorage {}

impl SyncKvStorage {
    pub(crate) fn new(inner: worker_sys::types::SyncKvStorage) -> Self {
        Self { inner }
    }
}

impl SyncKvStorage {
    pub fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let val = self.inner.get(key);

        if val.is_undefined() {
            Ok(None)
        } else {
            Ok(Some(swb::from_value(val)?))
        }
    }

    pub fn put<T>(&self, key: &str, value: T) -> Result<()>
    where
        T: Serialize,
    {
        let js = swb::to_value(&value)?;
        self.inner.put(key, js);
        Ok(())
    }

    pub fn delete(&self, key: &str) -> bool {
        self.inner.delete(key)
    }
}

pub struct SyncKvIterator<T> {
    inner: js_sys::Object,
    _phantom: PhantomData<T>,
}

impl<T> Iterator for SyncKvIterator<T>
where
    T: DeserializeOwned,
{
    type Item = Result<(String, T)>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = js_sys::Reflect::get(&self.inner, &"next".into())
            .ok()?
            .dyn_into::<js_sys::Function>()
            .ok()?;

        let result = next.call0(&self.inner).ok()?;

        let done = js_sys::Reflect::get(&result, &"done".into())
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if done {
            return None;
        }

        let value = js_sys::Reflect::get(&result, &"value".into())
            .map_err(Error::from)
            .and_then(|v| {
                let arr = js_sys::Array::from(&v);

                let key = arr.get(0).as_string().unwrap_or_default();
                let val = swb::from_value(arr.get(1))?;

                Ok((key, val))
            });

        Some(value)
    }
}

impl SyncKvStorage {
    pub fn list<T>(&self) -> SyncKvIterator<T>
    where
        T: DeserializeOwned,
    {
        SyncKvIterator {
            inner: self.inner.list(),
            _phantom: PhantomData,
        }
    }

    pub fn list_with_options<T>(&self, options: ListOptions<'_>) -> Result<SyncKvIterator<T>> {
        let js_opts = swb::to_value(&options)?;

        let iter = self.inner.list_with_options(js_opts.into());

        Ok(SyncKvIterator {
            inner: iter,
            _phantom: PhantomData,
        })
    }
}

impl fmt::Debug for SyncKvStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncKvStorage").finish()
    }
}

impl<T> fmt::Debug for SyncKvIterator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncKvIterator").finish()
    }
}
