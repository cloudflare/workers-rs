use wasm_bindgen::{JsCast, JsValue};

use crate::{Fetcher, Result};

#[derive(Debug)]
pub struct Container {
    pub(super) inner: worker_sys::Container,
}

impl Container {
    pub fn running(&self) -> bool {
        self.inner.running()
    }

    pub fn start(&self, options: Option<ContainerStartupOptions>) {
        self.inner.start(options.as_ref().map(|o| &o.inner))
    }

    pub fn get_tcp_port(&self, port: u16) -> Result<Fetcher> {
        self.inner
            .get_tcp_port(port)
            .map(|f| f.into())
            .map_err(|e| e.into())
    }
}

unsafe impl Sync for Container {}
unsafe impl Send for Container {}

impl From<worker_sys::Container> for Container {
    fn from(inner: worker_sys::Container) -> Self {
        Self { inner }
    }
}

impl AsRef<JsValue> for Container {
    fn as_ref(&self) -> &JsValue {
        &self.inner
    }
}

impl From<Container> for JsValue {
    fn from(database: Container) -> Self {
        JsValue::from(database.inner)
    }
}

impl JsCast for Container {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<worker_sys::Container>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self { inner: val.into() }
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

#[derive(Debug)]
pub struct ContainerStartupOptions {
    inner: worker_sys::ContainerStartupOptions,
}

impl ContainerStartupOptions {
    pub fn entrypoint(&self) -> Option<Vec<String>> {
        self.inner.entrypoint()
    }

    pub fn enable_internet(&self) -> bool {
        self.inner.enable_internet()
    }

    pub fn env(&self) -> Result<js_sys::Object> {
        Ok(self.inner.env()?)
    }
}

unsafe impl Sync for ContainerStartupOptions {}
unsafe impl Send for ContainerStartupOptions {}

impl From<worker_sys::ContainerStartupOptions> for ContainerStartupOptions {
    fn from(inner: worker_sys::ContainerStartupOptions) -> Self {
        Self { inner }
    }
}

impl AsRef<JsValue> for ContainerStartupOptions {
    fn as_ref(&self) -> &JsValue {
        &self.inner
    }
}

impl From<ContainerStartupOptions> for JsValue {
    fn from(database: ContainerStartupOptions) -> Self {
        JsValue::from(database.inner)
    }
}

impl JsCast for ContainerStartupOptions {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<worker_sys::ContainerStartupOptions>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self { inner: val.into() }
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}
