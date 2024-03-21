use std::collections::HashMap;

use crate::error::Error;
use crate::Date;
use crate::DateInit;
use crate::Result;

use js_sys::Array;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// Representing the options any FormData value can be, a field or a file.
pub enum FormEntry {
    Field(String),
    File(File),
}

/// A [FormData](https://developer.mozilla.org/en-US/docs/Web/API/FormData) representation of the
/// request body, providing access to form encoded fields and files.
#[derive(Debug)]
pub struct FormData(web_sys::FormData);

impl FormData {
    pub fn new() -> Self {
        Self(web_sys::FormData::new().unwrap())
    }

    /// Returns the first value associated with a given key from within a `FormData` object.
    pub fn get(&self, name: &str) -> Option<FormEntry> {
        let val = self.0.get(name);
        if val.is_undefined() {
            return None;
        }

        if val.is_instance_of::<web_sys::File>() {
            return Some(FormEntry::File(File(val.into())));
        }

        if let Some(field) = val.as_string() {
            return Some(FormEntry::Field(field));
        }

        None
    }

    /// Returns the first Field value associated with a given key from within a `FormData` object.
    pub fn get_field(&self, name: &str) -> Option<String> {
        let val = self.0.get(name);
        if val.is_undefined() {
            return None;
        }

        if val.is_instance_of::<web_sys::File>() {
            return None;
        }

        if let Some(field) = val.as_string() {
            return Some(field);
        }

        None
    }

    /// Returns a vec of all the values associated with a given key from within a `FormData` object.
    pub fn get_all(&self, name: &str) -> Option<Vec<FormEntry>> {
        let val = self.0.get_all(name);
        if val.is_undefined() {
            return None;
        }

        if Array::is_array(&val) {
            return Some(
                val.to_vec()
                    .into_iter()
                    .map(|val| {
                        if val.is_instance_of::<web_sys::File>() {
                            return FormEntry::File(File(val.into()));
                        }

                        FormEntry::Field(val.as_string().unwrap_or_default())
                    })
                    .collect(),
            );
        }

        None
    }

    /// Returns a boolean stating whether a `FormData` object contains a certain key.
    pub fn has(&self, name: &str) -> bool {
        self.0.has(name)
    }

    /// Appends a new value onto an existing key inside a `FormData` object, or adds the key if it
    /// does not already exist.
    pub fn append(&mut self, name: &str, value: &str) -> Result<()> {
        self.0.append_with_str(name, value).map_err(Error::from)
    }

    /// Sets a new value for an existing key inside a `FormData` object, or adds the key/value if it
    /// does not already exist.
    pub fn set(&mut self, name: &str, value: &str) -> Result<()> {
        self.0.set_with_str(name, value).map_err(Error::from)
    }

    /// Deletes a key/value pair from a `FormData` object.
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

/// A [File](https://developer.mozilla.org/en-US/docs/Web/API/File) representation used with
/// `FormData`.
pub struct File(web_sys::File);

impl File {
    /// Construct a new named file from a buffer.
    pub fn new(data: impl AsRef<[u8]>, name: &str) -> Self {
        let data = data.as_ref();
        let arr = Uint8Array::new_with_length(data.len() as u32);
        arr.copy_from(data);

        // The first parameter of File's constructor must be an ArrayBuffer or similar types
        // https://developer.mozilla.org/en-US/docs/Web/API/File/File
        let buffer = arr.buffer();
        let file = web_sys::File::new_with_u8_array_sequence(&Array::of1(&buffer), name).unwrap();

        Self(file)
    }

    /// Get the file name.
    pub fn name(&self) -> String {
        self.0.name()
    }

    /// Get the file size.
    pub fn size(&self) -> usize {
        self.0.size() as usize
    }

    /// Get the file type.
    pub fn type_(&self) -> String {
        self.0.type_()
    }

    /// Read the file from an internal buffer and get the resulting bytes.
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

    /// Get the last_modified metadata property of the file.
    pub fn last_modified(&self) -> Date {
        DateInit::Millis(self.0.last_modified() as u64).into()
    }
}

impl From<web_sys::File> for File {
    fn from(file: web_sys::File) -> Self {
        Self(file)
    }
}
