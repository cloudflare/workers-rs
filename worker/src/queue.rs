use std::{
    convert::{TryFrom, TryInto},
    marker::PhantomData,
};

use crate::{env::EnvBinding, Date, Error, Result};
use js_sys::Array;
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use worker_sys::{Message as MessageSys, MessageBatch as MessageBatchSys, Queue as EdgeQueue};

/// A batch of messages that are sent to a consumer Worker.
pub struct MessageBatch<T> {
    inner: MessageBatchSys,
    phantom: PhantomData<T>,
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

    /// Marks every message acknowledged in the batch.
    pub fn ack_all(&self) {
        self.inner.ack_all();
    }

    /// Iterator for raw messages in the message batch. Ordering of messages is not guaranteed.
    pub fn raw_iter(&self) -> RawMessageIter {
        let messages = self.inner.messages();
        RawMessageIter {
            range: 0..messages.length(),
            array: messages,
        }
    }
}

impl<T: DeserializeOwned> MessageBatch<T> {
    /// An array of messages in the batch. Ordering of messages is not guaranteed.
    pub fn messages(&self) -> Result<Vec<Message<T>>> {
        self.iter().collect()
    }

    /// Iterator for messages in the message batch. Ordering of messages is not guaranteed.
    pub fn iter(&self) -> MessageIter<T> {
        let messages = self.inner.messages();
        MessageIter {
            range: 0..messages.length(),
            array: messages,
            marker: PhantomData,
        }
    }
}

impl<T> From<MessageBatchSys> for MessageBatch<T> {
    fn from(value: MessageBatchSys) -> Self {
        Self {
            inner: value,
            phantom: PhantomData,
        }
    }
}

/// A message that is sent to a consumer Worker.
pub struct Message<T> {
    inner: MessageSys,
    body: T,
}

impl<T> Message<T> {
    /// The body of the message.
    pub fn body(&self) -> &T {
        &self.body
    }

    /// The body of the message.
    pub fn into_body(self) -> T {
        self.body
    }
}

impl<T> TryFrom<RawMessage> for Message<T>
where
    T: DeserializeOwned,
{
    type Error = Error;

    fn try_from(value: RawMessage) -> std::result::Result<Self, Self::Error> {
        let body = value.body()?;
        Ok(Self {
            inner: value.inner,
            body,
        })
    }
}

/// A message that is sent to a consumer Worker.
pub struct RawMessage {
    inner: MessageSys,
}

impl RawMessage {
    /// The body of the message.
    pub fn body<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        Ok(serde_wasm_bindgen::from_value(self.raw_body())?)
    }
}

impl From<MessageSys> for RawMessage {
    fn from(value: MessageSys) -> Self {
        Self { inner: value }
    }
}

trait MessageSysInner {
    fn inner(&self) -> &MessageSys;
}

impl MessageSysInner for RawMessage {
    fn inner(&self) -> &MessageSys {
        &self.inner
    }
}

impl<T> MessageSysInner for Message<T> {
    fn inner(&self) -> &MessageSys {
        &self.inner
    }
}

pub trait MessageExt {
    /// The raw body of the message.
    fn raw_body(&self) -> JsValue;

    /// A unique, system-generated ID for the message.
    fn id(&self) -> String;

    /// A timestamp when the message was sent.
    fn timestamp(&self) -> Date;

    /// Marks message to be retried.
    fn retry(&self);

    /// Marks message acknowledged.
    fn ack(&self);
}

impl<T: MessageSysInner> MessageExt for T {
    /// The raw body of the message.
    fn raw_body(&self) -> JsValue {
        self.inner().body()
    }

    /// A unique, system-generated ID for the message.
    fn id(&self) -> String {
        self.inner().id().into()
    }

    /// A timestamp when the message was sent.
    fn timestamp(&self) -> Date {
        Date::from(self.inner().timestamp())
    }

    /// Marks message to be retried.
    fn retry(&self) {
        self.inner().retry();
    }

    /// Marks message acknowledged.
    fn ack(&self) {
        self.inner().ack();
    }
}

pub struct MessageIter<T> {
    range: std::ops::Range<u32>,
    array: Array,
    marker: PhantomData<T>,
}

impl<T> std::iter::Iterator for MessageIter<T>
where
    T: DeserializeOwned,
{
    type Item = Result<Message<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.range.next()?;
        let value = self.array.get(index);
        let raw_message = RawMessage::from(MessageSys::from(value));
        Some(raw_message.try_into())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl<T> std::iter::DoubleEndedIterator for MessageIter<T>
where
    T: DeserializeOwned,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.range.next_back()?;
        let value = self.array.get(index);
        let raw_message = RawMessage::from(MessageSys::from(value));
        Some(raw_message.try_into())
    }
}

impl<T> std::iter::FusedIterator for MessageIter<T> where T: DeserializeOwned {}

impl<T> std::iter::ExactSizeIterator for MessageIter<T> where T: DeserializeOwned {}

pub struct RawMessageIter {
    range: std::ops::Range<u32>,
    array: Array,
}

impl std::iter::Iterator for RawMessageIter {
    type Item = RawMessage;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.range.next()?;
        let value = self.array.get(index);
        Some(MessageSys::from(value).into())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl std::iter::DoubleEndedIterator for RawMessageIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.range.next_back()?;
        let value = self.array.get(index);
        Some(MessageSys::from(value).into())
    }
}

impl std::iter::FusedIterator for RawMessageIter {}

impl std::iter::ExactSizeIterator for RawMessageIter {}

#[derive(Clone)]
pub struct Queue(EdgeQueue);

unsafe impl Send for Queue {}
unsafe impl Sync for Queue {}

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
        self.send_raw(js_value).await
    }

    /// Sends a raw message to the Queue.
    pub async fn send_raw(&self, message: JsValue) -> Result<()> {
        let fut: JsFuture = self.0.send(message).into();
        fut.await.map_err(Error::from)?;
        Ok(())
    }
}
