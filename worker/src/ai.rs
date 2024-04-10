use crate::{env::EnvBinding, send::SendFuture};
use crate::{Error, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::{console_log, Ai as AiSys};

pub struct Ai(AiSys);

#[derive(Deserialize)]
struct TextEmbeddingOutput {
    data: Vec<Vec<f64>>,
}

#[derive(Serialize)]
pub struct TextEmbeddingInput<'a> {
    text: Vec<&'a str>,
}

impl Ai {
    pub async fn run<T: Serialize, U: DeserializeOwned>(&self, model: &str, input: T) -> Result<U> {
        let fut = SendFuture::new(JsFuture::from(
            self.0.run(model, serde_wasm_bindgen::to_value(&input)?),
        ));
        match fut.await {
            Ok(output) => Ok(serde_wasm_bindgen::from_value(output)?),
            Err(err) => Err(Error::from(err)),
        }
    }

    pub async fn embed<'a, S: AsRef<str> + 'a, T: IntoIterator<Item = S>>(
        &self,
        model: &str,
        input: T,
    ) -> Result<Vec<Vec<f64>>> {
        let iter = input.into_iter();
        let items: Vec<S> = iter.collect();
        let text = items.iter().map(|s| s.as_ref()).collect();
        let arg = TextEmbeddingInput { text };
        self.run(model, arg)
            .await
            .map(|out: TextEmbeddingOutput| out.data)
    }
}

unsafe impl Sync for Ai {}
unsafe impl Send for Ai {}

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

    // Workaround for Miniflare D1 Beta
    fn get(val: JsValue) -> Result<Self> {
        let obj = js_sys::Object::from(val);
        console_log!("{}", obj.constructor().name());
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

#[cfg(test)]
mod test {
    use super::Ai;
    use wasm_bindgen::JsCast;

    #[test]
    #[allow(unused_must_use)]
    fn text_embedding_input_from() {
        let ai: Ai = js_sys::Object::new().unchecked_into();

        let s: &str = "foo";

        ai.embed("foo-model", [s]);
        ai.embed("foo-model", [s.to_owned()]);
        ai.embed("foo-model", [&(s.to_owned())]);
        ai.embed("foo-model", &[s]);
        ai.embed("foo-model", vec![s]);
        ai.embed("foo-model", &vec![s]);
    }
}
