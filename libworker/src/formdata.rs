use std::collections::HashMap;

use crate::error::Error;
use crate::Date;
use crate::DateInit;
use crate::Result;

use edgeworker_sys::{File as EdgeFile, FormData as EdgeFormData};

use js_sys::Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

pub enum FormDataEntryValue {
    Field(String),
    File(File),
}

#[derive(Debug)]
pub struct FormData(EdgeFormData);

impl FormData {
    pub fn new() -> Self {
        Self(EdgeFormData::new().unwrap())
    }

    pub fn get(&self, name: &str) -> Option<FormDataEntryValue> {
        let val = self.0.get(name);
        if val.is_undefined() {
            return None;
        }

        if val.is_instance_of::<EdgeFile>() {
            return Some(FormDataEntryValue::File(File(val.into())));
        }

        if let Some(field) = val.as_string() {
            return Some(FormDataEntryValue::Field(field));
        }

        return None;
    }

    pub fn get_all(&self, name: &str) -> Option<Vec<FormDataEntryValue>> {
        let val = self.0.get_all(name);
        if val.is_undefined() {
            return None;
        }

        if Array::is_array(&val) {
            return Some(
                val.to_vec()
                    .into_iter()
                    .map(|val| {
                        if val.is_instance_of::<EdgeFile>() {
                            return FormDataEntryValue::File(File(val.into()));
                        }

                        return FormDataEntryValue::Field(val.as_string().unwrap_or_default());
                    })
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

pub struct File(EdgeFile);

impl File {
    pub fn new(data: Vec<u8>, name: &str) -> Self {
        let arr = Array::new();
        for byte in data.into_iter() {
            arr.push(&byte.into());
        }

        let file = EdgeFile::new_with_u8_array_sequence(&JsValue::from(arr), name).unwrap();
        Self(file)
    }

    pub fn name(&self) -> String {
        self.0.name()
    }

    pub async fn bytes(&self) -> Result<Vec<u8>> {
        JsFuture::from(self.0.array_buffer())
            .await
            .map(|val| js_sys::Uint8Array::new(&val).to_vec())
            .map_err(|e| {
                Error::JsError(
                    e.as_string()
                        .unwrap_or_else(|| "failed to read array buffer from file".into()),
                )
            })
    }

    pub fn last_modified(&self) -> Date {
        DateInit::Millis(self.0.last_modified() as u64).into()
    }
}

impl From<EdgeFile> for File {
    fn from(file: EdgeFile) -> Self {
        Self(file)
    }
}
