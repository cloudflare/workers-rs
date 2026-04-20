use core::fmt;
use std::marker::PhantomData;

use serde::{Serialize, de::DeserializeOwned};
use serde_wasm_bindgen as swb;
use wasm_bindgen::JsCast as _;
use worker_sys::types::SyncKvStorage as SyncKvStorageSys;

use crate::{Error, ListOptions, Result};

#[derive(Clone)]
pub struct SyncKvStorage {
    inner: SyncKvStorageSys,
}

unsafe impl Send for SyncKvStorage {}
unsafe impl Sync for SyncKvStorage {}

impl SyncKvStorage {
    pub(crate) fn new(inner: SyncKvStorageSys) -> Self {
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
    inner: js_sys::IntoIter,
    _phantom: PhantomData<T>,
}

impl<T> Iterator for SyncKvIterator<T>
where
    T: DeserializeOwned,
{
    type Item = Result<(String, T)>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.inner.next()? {
            Ok(r) => r,
            Err(e) => return Some(Err(Error::from(e))),
        };

        if !js_sys::Array::is_array(&result) {
            return Some(Err(Error::JsError("Expected result to be array".into())));
        }

        let arr: js_sys::Array = result.unchecked_into();

        if arr.length() < 2 {
            return Some(Err(Error::JsError(
                "Expected entry to have at least 2 elements".into(),
            )));
        }

        let key = match arr.get(0).as_string() {
            Some(k) => k,
            None => {
                return Some(Err(Error::JsError("Expected key to be string".into())));
            }
        };

        let val = match swb::from_value(arr.get(1)) {
            Ok(v) => v,
            Err(e) => return Some(Err(Error::from(e))),
        };

        Some(Ok((key, val)))
    }
}

impl SyncKvStorage {
    const ERR_NOT_AN_ITERABLE: &str = "SyncKvStorage.list() did not return an iterable";

    pub fn list<T>(&self) -> Result<SyncKvIterator<T>>
    where
        T: DeserializeOwned,
    {
        let inner = js_sys::try_iter(&self.inner.list())?
            .ok_or_else(|| Error::JsError(Self::ERR_NOT_AN_ITERABLE.into()))?;

        Ok(SyncKvIterator {
            inner,
            _phantom: PhantomData,
        })
    }

    pub fn list_with_options<T>(&self, options: ListOptions<'_>) -> Result<SyncKvIterator<T>> {
        let js_opts = swb::to_value(&options)?;

        let iter = self.inner.list_with_options(js_opts.into());

        let inner = js_sys::try_iter(&iter)?
            .ok_or_else(|| Error::JsError(Self::ERR_NOT_AN_ITERABLE.into()))?;

        Ok(SyncKvIterator {
            inner,
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