use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{self, Poll};

use crate::{env::EnvBinding, send::SendFuture};
use crate::{Error, Result};
use futures_util::io::{BufReader, Lines};
use futures_util::{ready, AsyncBufReadExt as _, Stream, StreamExt as _};
use js_sys::Reflect;
use pin_project::pin_project;
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_streams::readable::IntoAsyncRead;
use worker_sys::Ai as AiSys;

/// Enables access to Workers AI functionality.
#[derive(Debug)]
pub struct Ai(AiSys);

impl Ai {
    /// Execute a Workers AI operation using the specified model.
    /// Various forms of the input are documented in the Workers
    /// AI documentation.
    pub async fn run<M: Model>(&self, input: M::Input) -> Result<M::Output> {
        let fut = SendFuture::new(JsFuture::from(
            self.0
                .run(M::MODEL_NAME, serde_wasm_bindgen::to_value(&input)?),
        ));
        match fut.await {
            Ok(output) => Ok(serde_wasm_bindgen::from_value(output)?),
            Err(err) => Err(Error::from(err)),
        }
    }

    pub async fn run_streaming<M: StreamableModel>(&self, input: M::Input) -> Result<AiStream<M>> {
        let input = serde_wasm_bindgen::to_value(&input)?;
        Reflect::set(&input, &JsValue::from_str("stream"), &JsValue::TRUE)?;

        let fut = SendFuture::new(JsFuture::from(self.0.run(M::MODEL_NAME, input)));
        let raw_stream = fut.await?.dyn_into::<web_sys::ReadableStream>()?;
        let stream = wasm_streams::ReadableStream::from_raw(raw_stream).into_async_read();

        Ok(AiStream::new(stream))
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

pub trait Model: 'static {
    const MODEL_NAME: &str;
    type Input: Serialize;
    type Output: DeserializeOwned;
}

pub trait StreamableModel: Model {}

#[derive(Debug)]
#[pin_project]
pub struct AiStream<T: StreamableModel> {
    #[pin]
    inner: Lines<BufReader<IntoAsyncRead<'static>>>,
    phantom: PhantomData<T>,
}

impl<T: StreamableModel> AiStream<T> {
    pub fn new(stream: IntoAsyncRead<'static>) -> Self {
        Self {
            inner: BufReader::new(stream).lines(),
            phantom: PhantomData,
        }
    }
}

impl<T: StreamableModel> Stream for AiStream<T> {
    type Item = Result<T::Output>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let string = match ready!(this.inner.poll_next_unpin(cx)) {
            Some(item) => match item {
                Ok(item) => {
                    if item.is_empty() {
                        match ready!(this.inner.poll_next_unpin(cx)) {
                            Some(item) => match item {
                                Ok(item) => item,
                                Err(err) => {
                                    return Poll::Ready(Some(Err(err.into())));
                                }
                            },
                            None => {
                                return Poll::Ready(None);
                            }
                        }
                    } else {
                        item
                    }
                }
                Err(err) => {
                    return Poll::Ready(Some(Err(err.into())));
                }
            },
            None => {
                return Poll::Ready(None);
            }
        };

        let string = if let Some(string) = string.strip_prefix("data: ") {
            string
        } else {
            string.as_str()
        };

        if string == "[DONE]" {
            return Poll::Ready(None);
        }

        Poll::Ready(Some(Ok(serde_json::from_str(string)?)))
    }
}
