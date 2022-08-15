use js_sys::{global, Function, Object, Promise, Reflect, Uint8Array};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

#[derive(Clone)]
pub struct R2Storage {
    pub(crate) this: Object,
    pub(crate) head_function: Function,
    pub(crate) get_function: Function,
    pub(crate) put_function: Function,
    pub(crate) list_function: Function,
    pub(crate) delete_function: Function,
}

impl R2Storage {
    pub fn create(binding: &str) -> Result<Self, R2Error> {
        let this = get(&global(), binding)?;

        if this.is_undefined() {
            Err(R2Error::InvalidR2Storage(binding.into()))
        } else {
            Ok(Self {
                head_function: get(&this, "head")?.into(),
                get_function: get(&this, "get")?.into(),
                put_function: get(&this, "put")?.into(),
                list_function: get(&this, "list")?.into(),
                delete_function: get(&this, "delete")?.into(),
                this: this.into(),
            })
        }
    }

    pub fn from_this(this: &JsValue, binding: &str) -> Result<Self, R2Error> {
        let this = get(this, binding)?;

        if this.is_undefined() {
            Err(R2Error::InvalidR2Storage(binding.into()))
        } else {
            Ok(Self {
                head_function: get(&this, "head")?.into(),
                get_function: get(&this, "get")?.into(),
                put_function: get(&this, "put")?.into(),
                list_function: get(&this, "list")?.into(),
                delete_function: get(&this, "delete")?.into(),
                this: this.into(),
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum R2Error {
    #[error("js error: {0:?}")]
    JavaScript(JsValue),
    #[error("unable to serialize/deserialize: {0}")]
    Serialization(serde_json::Error),
    #[error("invalid r2 storage: {0}")]
    InvalidR2Storage(String),
}

impl From<JsValue> for R2Error {
    fn from(value: JsValue) -> Self {
        Self::JavaScript(value)
    }
}

impl From<serde_json::Error> for R2Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}

fn get(target: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    Reflect::get(target, &JsValue::from(name))
}
