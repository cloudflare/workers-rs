use crate::streams::ByteStream;
use crate::{env::EnvBinding, send::SendFuture};
use crate::{Error, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::ReadableStream;
use worker_sys::Ai as AiSys;

/// Enables access to Workers AI functionality.
#[derive(Debug)]
pub struct Ai(AiSys);

impl Ai {
    /// Execute a Workers AI operation using the specified model.
    /// Various forms of the input are documented in the Workers
    /// AI documentation.
    pub async fn run<T: Serialize, U: DeserializeOwned>(
        &self,
        model: impl AsRef<str>,
        input: T,
    ) -> Result<U> {
        let fut = SendFuture::new(JsFuture::from(
            self.0
                .run(model.as_ref(), serde_wasm_bindgen::to_value(&input)?),
        ));
        match fut.await {
            Ok(output) => Ok(serde_wasm_bindgen::from_value(output)?),
            Err(err) => Err(Error::from(err)),
        }
    }

    /// Execute a Workers AI operation that returns binary data as a [`ByteStream`].
    ///
    /// This method is designed for AI models that return raw bytes, such as:
    /// - Image generation models (e.g., Stable Diffusion)
    /// - Text-to-speech models
    /// - Any other model that returns binary output
    ///
    /// The returned [`ByteStream`] implements [`Stream`](futures_util::Stream) and can be:
    /// - Streamed directly to a [`Response`] using [`Response::from_stream`]
    /// - Collected into a `Vec<u8>` by iterating over the chunks
    ///
    /// # Examples
    ///
    /// ## Streaming directly to a response (recommended)
    ///
    /// This approach is more memory-efficient as it doesn't buffer the entire
    /// response in memory:
    ///
    /// ```ignore
    /// use worker::*;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct ImageGenRequest {
    ///     prompt: String,
    /// }
    ///
    /// async fn generate_image(env: &Env) -> Result<Response> {
    ///     let ai = env.ai("AI")?;
    ///     let request = ImageGenRequest {
    ///         prompt: "a beautiful sunset".to_string(),
    ///     };
    ///     let stream = ai.run_bytes(
    ///         "@cf/stabilityai/stable-diffusion-xl-base-1.0",
    ///         &request
    ///     ).await?;
    ///
    ///     // Stream directly to the response
    ///     let mut response = Response::from_stream(stream)?;
    ///     response.headers_mut().set("Content-Type", "image/png")?;
    ///     Ok(response)
    /// }
    /// ```
    ///
    /// ## Collecting into bytes
    ///
    /// Use this approach if you need to inspect or modify the bytes before sending:
    ///
    /// ```ignore
    /// use worker::*;
    /// use serde::Serialize;
    /// use futures_util::StreamExt;
    ///
    /// #[derive(Serialize)]
    /// struct ImageGenRequest {
    ///     prompt: String,
    /// }
    ///
    /// async fn generate_image(env: &Env) -> Result<Response> {
    ///     let ai = env.ai("AI")?;
    ///     let request = ImageGenRequest {
    ///         prompt: "a beautiful sunset".to_string(),
    ///     };
    ///     let mut stream = ai.run_bytes(
    ///         "@cf/stabilityai/stable-diffusion-xl-base-1.0",
    ///         &request
    ///     ).await?;
    ///
    ///     // Collect all chunks into a Vec<u8>
    ///     let mut bytes = Vec::new();
    ///     while let Some(chunk) = stream.next().await {
    ///         bytes.extend_from_slice(&chunk?);
    ///     }
    ///
    ///     let mut response = Response::from_bytes(bytes)?;
    ///     response.headers_mut().set("Content-Type", "image/png")?;
    ///     Ok(response)
    /// }
    /// ```
    pub async fn run_bytes<T: Serialize>(
        &self,
        model: impl AsRef<str>,
        input: T,
    ) -> Result<ByteStream> {
        let fut = SendFuture::new(JsFuture::from(
            self.0
                .run(model.as_ref(), serde_wasm_bindgen::to_value(&input)?),
        ));
        match fut.await {
            Ok(output) => {
                if output.is_instance_of::<ReadableStream>() {
                    let stream = ReadableStream::unchecked_from_js(output);
                    Ok(ByteStream::from(stream))
                } else {
                    Err(Error::RustError(
                        "AI model did not return binary data. Use run() for non-binary responses."
                            .into(),
                    ))
                }
            }
            Err(err) => Err(Error::from(err)),
        }
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
