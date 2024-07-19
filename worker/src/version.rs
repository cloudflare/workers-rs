use crate::EnvBinding;
use wasm_bindgen::{JsCast, JsValue};
use worker_sys::types::CfVersionMetadata;

pub struct WorkerVersionMetadata(CfVersionMetadata);

unsafe impl Send for WorkerVersionMetadata {}
unsafe impl Sync for WorkerVersionMetadata {}

impl EnvBinding for WorkerVersionMetadata {
    const TYPE_NAME: &'static str = "Object";
}

impl WorkerVersionMetadata {
    pub fn id(&self) -> String {
        self.0.id()
    }

    pub fn tag(&self) -> String {
        self.0.tag()
    }
}

impl JsCast for WorkerVersionMetadata {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<CfVersionMetadata>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self(val.into())
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl From<WorkerVersionMetadata> for JsValue {
    fn from(cf_version: WorkerVersionMetadata) -> Self {
        JsValue::from(cf_version.0)
    }
}

impl AsRef<JsValue> for WorkerVersionMetadata {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}
