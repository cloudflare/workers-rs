use crate::{env::EnvBinding, Date, Error, Result};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use worker_sys::{MessageBatch as MessageBatchSys, Queue as EdgeQueue};

static BODY_KEY_STR: &str = "body";
static ID_KEY_STR: &str = "id";
static TIMESTAMP_KEY_STR: &str = "timestamp";

#[derive(Debug)]
pub struct MessageBatch {
    inner: MessageBatchSys,
}

pub struct Message<T> {
    pub body: T,
    pub timestamp: Date,
    pub id: String,
}

impl From<MessageBatchSys> for MessageBatch {
    fn from(message_batch_sys: MessageBatchSys) -> Self {
        Self {
            inner: message_batch_sys,
        }
    }
}

impl MessageBatch {
    /// The name of the Queue that belongs to this batch.
    pub fn queue(&self) -> String {
        self.inner.queue()
    }

    /// Marks every message to be retried in the next batch.
    pub fn retry_all(&self) {
        self.inner.retry_all();
    }

    /// An array of messages in the batch. Ordering of messages is not guaranteed.
    pub fn messages<T>(&self) -> Result<Vec<Message<T>>>
    where
        T: for<'a> Deserialize<'a>,
    {
        let timestamp_key = JsValue::from_str(TIMESTAMP_KEY_STR);
        let body_key = JsValue::from_str(BODY_KEY_STR);
        let id_key = JsValue::from_str(ID_KEY_STR);

        let mut vec: Vec<Message<T>> = Vec::with_capacity(self.inner.messages().length() as usize);

        for result in self.inner.messages().iter() {
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
    /// Sends a message to the Queue.
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
