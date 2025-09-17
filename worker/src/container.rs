use js_sys::{Map, Object, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::{Fetcher, Result};

#[derive(Debug)]
pub struct Container {
    pub(super) inner: worker_sys::Container,
}

impl Container {
    pub fn running(&self) -> bool {
        self.inner.running()
    }

    pub fn start(&self, options: Option<ContainerStartupOptions>) -> Result<()> {
        let options = match options {
            Some(o) => o.into(),
            None => JsValue::undefined(),
        };
        self.inner.start(&options).map_err(|e| e.into())
    }

    pub async fn wait_for_exit(&self) -> Result<()> {
        let promise = self.inner.monitor();
        JsFuture::from(promise).await?;
        Ok(())
    }

    pub async fn destroy(&self, error: Option<&str>) -> Result<()> {
        let promise = self.inner.destroy(error);
        JsFuture::from(promise).await?;
        Ok(())
    }

    pub fn signal(&self, signo: i32) -> Result<()> {
        self.inner.signal(signo).map_err(|e| e.into())
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
    fn from(container: Container) -> Self {
        JsValue::from(container.inner)
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

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct ContainerStartupOptions {
    pub entrypoint: Vec<String>,
    #[wasm_bindgen(js_name = "enableInternet")]
    pub enable_internet: Option<bool>,
    pub env: Map,
}

impl ContainerStartupOptions {
    pub fn new() -> ContainerStartupOptions {
        ContainerStartupOptions {
            entrypoint: Vec::new(),
            enable_internet: None,
            env: Map::new(),
        }
    }

    pub fn set_entrypoint(&mut self, entrypoint: &[&str]) {
        self.entrypoint = entrypoint.iter().map(|s| s.to_string()).collect();
    }

    pub fn enable_internet(&mut self, enable_internet: bool) {
        self.enable_internet = Some(enable_internet);
    }

    pub fn add_env(&mut self, key: &str, value: &str) {
        self.env
            .set(&JsValue::from_str(key), &JsValue::from_str(value));
    }
}

impl From<ContainerStartupOptions> for Object {
    fn from(options: ContainerStartupOptions) -> Self {
        let obj = options.clone().into();
        if !options.entrypoint.is_empty() {
            Reflect::delete_property(&obj, &JsValue::from_str("entrypoint")).unwrap();
        }
        if options.enable_internet.is_some() {
            Reflect::delete_property(&obj, &JsValue::from_str("enableInternet")).unwrap();
        }
        if options.env.size() != 0 {
            Reflect::delete_property(&obj, &JsValue::from_str("env")).unwrap();
        }
        obj
    }
}
