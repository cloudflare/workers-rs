use crate::{send::SendFuture, EnvBinding, Result};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::RateLimiter as RateLimiterSys;

pub struct RateLimiter(RateLimiterSys);

#[derive(Serialize, Deserialize)]
struct RateLimitOptions {
    key: String,
}

#[derive(Serialize, Deserialize)]
pub struct RateLimitOutcome {
    success: bool,
}

unsafe impl Send for RateLimiter {}
unsafe impl Sync for RateLimiter {}

impl EnvBinding for RateLimiter {
    const TYPE_NAME: &'static str = "RateLimiter";
}
impl RateLimiter {
    pub async fn limit(&self, key: String) -> Result<RateLimitOutcome> {
        let arg = serde_wasm_bindgen::to_value(&RateLimitOptions { key })?;
        let promise = self.0.limit(arg.into())?;
        let fut = SendFuture::new(JsFuture::from(promise));
        let result = fut.await?;
        let outcome = serde_wasm_bindgen::from_value(result)?;
        Ok(outcome)
    }
}

impl JsCast for RateLimiter {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<RateLimiterSys>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self(val.into())
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl From<RateLimiter> for JsValue {
    fn from(limiter: RateLimiter) -> Self {
        JsValue::from(limiter.0)
    }
}

impl AsRef<JsValue> for RateLimiter {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}

impl From<RateLimiterSys> for RateLimiter {
    fn from(inner: RateLimiterSys) -> Self {
        Self(inner)
    }
}
