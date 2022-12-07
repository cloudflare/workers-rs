use crate::{env::EnvBinding, Date, Error, Result};
use js_sys::Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use worker_sys::{MessageBatch as EdgeMessageBatch, Queue as EdgeQueue};

static BODY_KEY_STR: &str = "body";
static ID_KEY_STR: &str = "id";
static TIMESTAMP_KEY_STR: &str = "timestamp";

#[derive(Debug)]
pub struct MessageBatch {
    queue: String,
    messages: Array,
}

pub struct Message<T> {
    pub body: T,
    pub timestamp: Date,
    pub id: String,
}

impl From<EdgeMessageBatch> for MessageBatch {
    fn from(value: EdgeMessageBatch) -> Self {
        Self {
            messages: value.messages(),
            queue: value.queue(),
        }
    }
}

impl MessageBatch {
    pub fn queue(&self) -> String {
        self.queue.clone()
    }

    pub fn messages<T>(&self) -> Result<Vec<Message<T>>>
    where
        T: for<'a> Deserialize<'a>,
    {
        let timestamp_key = JsValue::from_str(TIMESTAMP_KEY_STR);
        let body_key = JsValue::from_str(BODY_KEY_STR);
        let id_key = JsValue::from_str(ID_KEY_STR);

        let mut vec: Vec<Message<T>> = Vec::with_capacity(self.messages.length() as usize);

        for result in self.messages.iter() {
            let js_date = js_sys::Date::from(js_sys::Reflect::get(&result, &timestamp_key)?);
            let body = serde_wasm_bindgen::from_value(js_sys::Reflect::get(&result, &body_key)?)?;
            let id = js_sys::Reflect::get(&result, &id_key)?
                .as_string()
                .ok_or(Error::JsError(
                    "Invalid message batch. Failed to get id from message.".to_string(),
                ))?;

            let message = Message {
                id,
                body,
                timestamp: Date::from(js_date),
            };
            vec.push(message);
        }
        Ok(vec)
    }
}

pub struct Queue(EdgeQueue);

impl EnvBinding for Queue {
    const TYPE_NAME: &'static str = "WorkerQueue";
}

impl JsCast for Queue {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<Queue>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self(val.into())
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl From<Queue> for JsValue {
    fn from(queue: Queue) -> Self {
        JsValue::from(queue.0)
    }
}

impl AsRef<JsValue> for Queue {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}

impl Queue {
    pub async fn send<T>(&self, message: &T) -> Result<()>
    where
        T: Serialize,
    {
        let js_value = serde_wasm_bindgen::to_value(message)?;
        let fut: JsFuture = self.0.send(js_value).into();

        fut.await.map_err(Error::from)?;
        Ok(())
    }
}
