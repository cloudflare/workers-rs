use std::marker::PhantomData;

use crate::{env::EnvBinding, Date, Error, Result};
use js_sys::Array;
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use worker_sys::{Message as MessageSys, MessageBatch as MessageBatchSys, Queue as EdgeQueue};

pub struct MessageBatch<T> {
    inner: MessageBatchSys,
    messages: Array,
    data: PhantomData<T>,
}

impl<T> MessageBatch<T> {
    pub fn new(message_batch_sys: MessageBatchSys) -> Self {
        Self {
            messages: message_batch_sys.messages(),
            inner: message_batch_sys,
            data: PhantomData,
        }
    }
}

pub struct Message<T> {
    pub body: T,
    pub timestamp: Date,
    pub id: String,
}

impl<T> MessageBatch<T> {
    /// The name of the Queue that belongs to this batch.
    pub fn queue(&self) -> String {
        self.inner.queue().into()
    }

    /// Marks every message to be retried in the next batch.
    pub fn retry_all(&self) {
        self.inner.retry_all();
    }

    /// Iterator that deserializes messages in the message batch. Ordering of messages is not guaranteed.
    pub fn iter(&self) -> MessageIter<'_, T>
    where
        T: DeserializeOwned,
    {
        MessageIter {
            range: 0..self.messages.length(),
            array: &self.messages,
            data: PhantomData,
        }
    }

    /// An array of messages in the batch. Ordering of messages is not guaranteed.
    pub fn messages(&self) -> Result<Vec<Message<T>>>
    where
        T: DeserializeOwned,
    {
        self.iter().collect()
    }
}

pub struct MessageIter<'a, T> {
    range: std::ops::Range<u32>,
    array: &'a Array,
    data: PhantomData<T>,
}

fn parse_message<T>(message: JsValue) -> Result<Message<T>>
where
    T: DeserializeOwned,
{
    let message = MessageSys::from(message);
    Ok(Message {
        id: message.id().into(),
        body: serde_wasm_bindgen::from_value(message.body())?,
        timestamp: Date::from(message.timestamp()),
    })
}

impl<T> std::iter::Iterator for MessageIter<'_, T>
where
    T: DeserializeOwned,
{
    type Item = Result<Message<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.range.next()?;

        let value = self.array.get(index);

        Some(parse_message(value))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl<T> std::iter::DoubleEndedIterator for MessageIter<'_, T>
where
    T: DeserializeOwned,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.range.next_back()?;
        let value = self.array.get(index);

        Some(parse_message(value))
    }
}

impl<'a, T> std::iter::FusedIterator for MessageIter<'a, T> where T: DeserializeOwned {}

impl<'a, T> std::iter::ExactSizeIterator for MessageIter<'a, T> where T: DeserializeOwned {}

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
