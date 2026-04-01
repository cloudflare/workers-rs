use wasm_bindgen::{JsCast, JsValue};
use worker_sys::types::Hyperdrive as HyperdriveSys;

use crate::{EnvBinding, Result, Socket};

#[derive(Debug)]
pub struct Hyperdrive(HyperdriveSys);

unsafe impl Send for Hyperdrive {}
unsafe impl Sync for Hyperdrive {}

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

impl Hyperdrive {
    pub fn connect(&self) -> Result<Socket> {
        Ok(Socket::new(self.0.connect()?))
    }

    pub fn connection_string(&self) -> String {
        self.0.connection_string()
    }

    pub fn host(&self) -> String {
        self.0.host()
    }

    pub fn port(&self) -> u16 {
        self.0.port()
    }

    pub fn user(&self) -> String {
        self.0.user()
    }

    pub fn password(&self) -> String {
        self.0.password()
    }

    pub fn database(&self) -> String {
        self.0.database()
    }
}
