use crate::{env::EnvBinding, send::SendFuture};
use crate::{Error, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::Ai as AiSys;

#[derive(Debug)]
pub struct Ai(AiSys);

impl Ai {
    pub async fn run<T: Serialize, U: DeserializeOwned>(
        &self,
        model: impl AsRef<str>,
        input: T,
    ) -> Result<U> {
        let fut = SendFuture::new_unchecked(JsFuture::from(
            self.0
                .run(model.as_ref(), serde_wasm_bindgen::to_value(&input)?),
        ));
        match fut.await {
            Ok(output) => Ok(serde_wasm_bindgen::from_value(output)?),
            Err(err) => Err(Error::from(err)),
        }
    }
}

impl From<AiSys> for Ai {
    fn from(inner: AiSys) -> Self {
        Self(inner)
    }
}

impl AsRef<JsValue> for Ai {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}

impl From<Ai> for JsValue {
    fn from(database: Ai) -> Self {
        JsValue::from(database.0)
    }
}

impl JsCast for Ai {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<AiSys>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self(val.into())
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl EnvBinding for Ai {
    const TYPE_NAME: &'static str = "Ai";

    fn get(val: JsValue) -> Result<Self> {
        let obj = js_sys::Object::from(val);
        if obj.constructor().name() == Self::TYPE_NAME {
            Ok(obj.unchecked_into())
        } else {
            Err(format!(
                "Binding cannot be cast to the type {} from {}",
                Self::TYPE_NAME,
                obj.constructor().name()
            )
            .into())
        }
    }
}
