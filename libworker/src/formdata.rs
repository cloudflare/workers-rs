use std::collections::HashMap;

use crate::error::Error;
use crate::Result;

use edgeworker_ffi::FormData as EdgeFormData;

use js_sys::Array;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub struct FormData(EdgeFormData);

impl FormData {
    pub fn new() -> Self {
        Self(EdgeFormData::new().unwrap())
    }

    pub fn get(&self, name: &str) -> Option<String> {
        let val = self.0.get(name);
        if val.is_undefined() {
            return None;
        }

        val.as_string()
    }

    pub fn get_all(&self, name: &str) -> Option<Vec<String>> {
        let val = self.0.get_all(name);
        if val.is_undefined() {
            return None;
        }

        if Array::is_array(&val) {
            return Some(
                val.to_vec()
                    .iter()
                    .map(|val| val.as_string().unwrap_or_default())
                    .collect(),
            );
        }

        None
    }

    pub fn has(&self, name: &str) -> bool {
        self.0.has(name)
    }

    pub fn append(&mut self, name: &str, value: &str) -> Result<()> {
        self.0.append_with_str(name, value).map_err(Error::from)
    }

    pub fn set(&mut self, name: &str, value: &str) -> Result<()> {
        self.0.set_with_str(name, value).map_err(Error::from)
    }

    pub fn delete(&mut self, name: &str) {
        self.0.delete(name)
    }
}

impl From<JsValue> for FormData {
    fn from(val: JsValue) -> Self {
        FormData(val.into())
    }
}

impl From<HashMap<&dyn AsRef<&str>, &dyn AsRef<&str>>> for FormData {
    fn from(m: HashMap<&dyn AsRef<&str>, &dyn AsRef<&str>>) -> Self {
        let mut formdata = FormData::new();
        for (k, v) in m {
            // TODO: determine error case and consider how to handle
            formdata.set(k.as_ref(), v.as_ref()).unwrap();
        }
        formdata
    }
}
