use wasm_bindgen::{JsCast, JsValue};
use worker_sys::types::Hyperdrive as HyperdriveSys;

use crate::EnvBinding;

pub struct Hyperdrive(HyperdriveSys);

impl EnvBinding for Hyperdrive {
    const TYPE_NAME: &'static str = "Hyperdrive";
}

impl JsCast for Hyperdrive {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<HyperdriveSys>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self(val.into())
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl AsRef<JsValue> for Hyperdrive {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}

impl From<Hyperdrive> for JsValue {
    fn from(hyperdrive: Hyperdrive) -> Self {
        JsValue::from(hyperdrive.0)
    }
}
