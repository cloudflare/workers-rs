use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::env::EnvBinding;
use crate::{Response, Result};
use worker_sys::images::Images as SysImages;

pub struct Images {
    inner: SysImages,
}

impl Images {
    pub(crate) fn new(inner: SysImages) -> Self {
        Self { inner }
    }

    pub async fn fetch(
        &self,
        input: impl Into<JsValue>,
        options: Option<JsValue>,
    ) -> Result<Response> {
        let input = input.into();
        let options = options.unwrap_or(JsValue::UNDEFINED);

        let promise = self.inner.fetch(input, options);
        let js_value = JsFuture::from(promise).await?;
        let response: web_sys::Response = js_value.dyn_into()?;
        Ok(Response::from(response))
    }
}

impl JsCast for Images {
    fn instanceof(val: &JsValue) -> bool {
        SysImages::instanceof(val)
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self {
            inner: SysImages::unchecked_from_js(val),
        }
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Images) }
    }
}

impl AsRef<JsValue> for Images {
    fn as_ref(&self) -> &JsValue {
        self.inner.as_ref()
    }
}

impl From<Images> for JsValue {
    fn from(images: Images) -> JsValue {
        images.inner.into()
    }
}

impl EnvBinding for Images {
    const TYPE_NAME: &'static str = "Images";
}