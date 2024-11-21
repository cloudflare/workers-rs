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
        self.inner.queue().unwrap().into()
    }

    /// Marks every message to be retried in the next batch.
    pub fn retry_all(&self) {
        self.inner.retry_all(JsValue::null()).unwrap();
    }

    /// Marks every message to be retried in the next batch with options.
    pub fn retry_all_with_options(&self, queue_retry_options: &QueueRetryOptions) {
        self.inner
            // SAFETY: QueueRetryOptions is controlled by this module and all data in it is serializable to a js value.
            .retry_all(serde_wasm_bindgen::to_value(&queue_retry_options).unwrap())
            .unwrap();
    }

    /// Marks every message acknowledged in the batch.
    pub fn ack_all(&self) {
        self.inner.ack_all().unwrap();
    }

    /// Iterator for raw messages in the message batch. Ordering of messages is not guaranteed.
    pub fn raw_iter(&self) -> RawMessageIter {
        let messages = self.inner.messages().unwrap();
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
        let messages = self.inner.messages().unwrap();
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

    /// The raw body of the message.
    pub fn raw_body(&self) -> JsValue {
        self.inner().body().unwrap()
    }
}

impl<T> TryFrom<RawMessage> for Message<T>
where
    T: DeserializeOwned,
{
    type Error = Error;

    fn try_from(value: RawMessage) -> std::result::Result<Self, Self::Error> {
        let body = serde_wasm_bindgen::from_value(value.body())?;
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
    pub fn body(&self) -> JsValue {
        self.inner.body().unwrap()
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
/// Optional configuration when marking a message or a batch of messages for retry.
pub struct QueueRetryOptions {
    delay_seconds: Option<u32>,
}

pub struct QueueRetryOptionsBuilder {
    delay_seconds: Option<u32>,
}

impl QueueRetryOptionsBuilder {
    /// Creates a new retry options builder.
    pub fn new() -> Self {
        Self {
            delay_seconds: None,
        }
    }

    #[must_use]
    /// The number of seconds to delay a message for within the queue, before it can be delivered to a consumer
    pub fn with_delay_seconds(mut self, delay_seconds: u32) -> Self {
        self.delay_seconds = Some(delay_seconds);
        self
    }

    /// Build the retry options.
    pub fn build(self) -> QueueRetryOptions {
        QueueRetryOptions {
            delay_seconds: self.delay_seconds,
        }
    }
}

pub trait MessageExt {
    /// A unique, system-generated ID for the message.
    fn id(&self) -> String;

    /// A timestamp when the message was sent.
    fn timestamp(&self) -> Date;

    /// Marks message to be retried.
    fn retry(&self);

    /// Marks message to be retried with options.
    fn retry_with_options(&self, queue_retry_options: &QueueRetryOptions);

    /// Marks message acknowledged.
    fn ack(&self);
}

impl<T: MessageSysInner> MessageExt for T {
    /// A unique, system-generated ID for the message.
    fn id(&self) -> String {
        self.inner().id().unwrap().into()
    }

    /// A timestamp when the message was sent.
    fn timestamp(&self) -> Date {
        Date::from(self.inner().timestamp().unwrap())
    }

    /// Marks message to be retried.
    fn retry(&self) {
        self.inner().retry(JsValue::null()).unwrap();
    }

    /// Marks message to be retried with options.
    fn retry_with_options(&self, queue_retry_options: &QueueRetryOptions) {
        self.inner()
            // SAFETY: QueueRetryOptions is controlled by this module and all data in it is serializable to a js value.
            .retry(serde_wasm_bindgen::to_value(&queue_retry_options).unwrap())
            .unwrap();
    }

    /// Marks message acknowledged.
    fn ack(&self) {
        self.inner().ack().unwrap();
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

#[derive(Clone, Copy, Debug)]
pub enum QueueContentType {
    /// Send a JavaScript object that can be JSON-serialized. This content type can be previewed from the Cloudflare dashboard.
    Json,
    /// Send a String. This content type can be previewed with the List messages from the dashboard feature.
    Text,
    /// Send a JavaScript object that cannot be JSON-serialized but is supported by structured clone (for example Date and Map). This content type cannot be previewed from the Cloudflare dashboard and will display as Base64-encoded.
    V8,
}

impl Serialize for QueueContentType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Self::Json => "json",
            Self::Text => "text",
            Self::V8 => "v8",
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueSendOptions {
    content_type: Option<QueueContentType>,
    delay_seconds: Option<u32>,
}

pub struct MessageBuilder<T> {
    message: T,
    delay_seconds: Option<u32>,
    content_type: QueueContentType,
}

impl<T: Serialize> MessageBuilder<T> {
    /// Creates a new message builder. The message must be `serializable`.
    pub fn new(message: T) -> Self {
        Self {
            message,
            delay_seconds: None,
            content_type: QueueContentType::Json,
        }
    }

    #[must_use]
    /// The number of seconds to delay a message for within the queue, before it can be delivered to a consumer
    pub fn delay_seconds(mut self, delay_seconds: u32) -> Self {
        self.delay_seconds = Some(delay_seconds);
        self
    }

    #[must_use]
    /// The content type of the message.
    /// Default is `QueueContentType::Json`.
    pub fn content_type(mut self, content_type: QueueContentType) -> Self {
        self.content_type = content_type;
        self
    }

    /// Build the message.
    pub fn build(self) -> SendMessage<T> {
        SendMessage {
            message: self.message,
            options: Some(QueueSendOptions {
                content_type: Some(self.content_type),
                delay_seconds: self.delay_seconds,
            }),
        }
    }
}

pub struct RawMessageBuilder {
    message: JsValue,
    delay_seconds: Option<u32>,
}

impl RawMessageBuilder {
    /// Creates a new raw message builder. The message must be a `JsValue`.
    pub fn new(message: JsValue) -> Self {
        Self {
            message,
            delay_seconds: None,
        }
    }

    #[must_use]
    /// The number of seconds to delay a message for within the queue, before it can be delivered to a consumer
    pub fn delay_seconds(mut self, delay_seconds: u32) -> Self {
        self.delay_seconds = Some(delay_seconds);
        self
    }

    /// Build the message with a content type.
    pub fn build_with_content_type(self, content_type: QueueContentType) -> SendMessage<JsValue> {
        SendMessage {
            message: self.message,
            options: Some(QueueSendOptions {
                content_type: Some(content_type),
                delay_seconds: self.delay_seconds,
            }),
        }
    }
}

/// A wrapper type used for sending message.
///
/// This type can't be constructed directly.
///
/// It should be constructed using the `MessageBuilder`, `RawMessageBuilder` or by calling `.into()` on a struct that is `serializable`.
pub struct SendMessage<T> {
    /// The body of the message.
    ///
    /// Can be either a serializable struct or a `JsValue`.
    message: T,

    /// Options to apply to the current message, including content type and message delay settings.
    options: Option<QueueSendOptions>,
}

impl<T: Serialize> SendMessage<T> {
    fn into_raw_send_message(self) -> Result<SendMessage<JsValue>> {
        Ok(SendMessage {
            message: serde_wasm_bindgen::to_value(&self.message)?,
            options: self.options,
        })
    }
}

impl<T: Serialize> From<T> for SendMessage<T> {
    fn from(message: T) -> Self {
        Self {
            message,
            options: Some(QueueSendOptions {
                content_type: Some(QueueContentType::Json),
                delay_seconds: None,
            }),
        }
    }
}

pub struct BatchSendMessage<T> {
    body: Vec<SendMessage<T>>,
    options: Option<QueueSendBatchOptions>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueSendBatchOptions {
    delay_seconds: Option<u32>,
}

pub struct BatchMessageBuilder<T> {
    messages: Vec<SendMessage<T>>,
    delay_seconds: Option<u32>,
}

impl<T> BatchMessageBuilder<T> {
    /// Creates a new batch message builder.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            delay_seconds: None,
        }
    }

    #[must_use]
    /// Adds a message to the batch.
    pub fn message<U: Into<SendMessage<T>>>(mut self, message: U) -> Self {
        self.messages.push(message.into());
        self
    }

    #[must_use]
    /// Adds messages to the batch.
    pub fn messages<U, V>(mut self, messages: U) -> Self
    where
        U: IntoIterator<Item = V>,
        V: Into<SendMessage<T>>,
    {
        self.messages
            .extend(messages.into_iter().map(std::convert::Into::into));
        self
    }

    #[must_use]
    /// The number of seconds to delay a message for within the queue, before it can be delivered to a consumer
    pub fn delay_seconds(mut self, delay_seconds: u32) -> Self {
        self.delay_seconds = Some(delay_seconds);
        self
    }

    pub fn build(self) -> BatchSendMessage<T> {
        BatchSendMessage {
            body: self.messages,
            options: self
                .delay_seconds
                .map(|delay_seconds| QueueSendBatchOptions {
                    delay_seconds: Some(delay_seconds),
                }),
        }
    }
}

impl<T, U, V> From<U> for BatchSendMessage<T>
where
    U: IntoIterator<Item = V>,
    V: Into<SendMessage<T>>,
{
    fn from(value: U) -> Self {
        Self {
            body: value.into_iter().map(std::convert::Into::into).collect(),
            options: None,
        }
    }
}

impl<T: Serialize> BatchSendMessage<T> {
    fn into_raw_batch_send_message(self) -> Result<BatchSendMessage<JsValue>> {
        Ok(BatchSendMessage {
            body: self
                .body
                .into_iter()
                .map(SendMessage::into_raw_send_message)
                .collect::<Result<_>>()?,
            options: self.options,
        })
    }
}

impl Queue {
    /// Sends a message to the Queue.
    ///
    /// Accepts a struct that is `serializable`.
    ///
    /// If message options are needed use the `MessageBuilder` to create the message.
    ///
    /// ## Example
    /// ```no_run
    /// # use serde::Serialize;
    /// # use js_sys::Object;
    /// # use wasm_bindgen::JsCast;
    /// #[derive(Serialize)]
    /// pub struct MyMessage {
    ///     my_data: u32,
    /// }
    /// # let env: worker::Env = Object::new().unchecked_into();
    /// # tokio_test::block_on(async {
    /// # let queue = env.queue("FOO")?;
    /// queue.send(MyMessage{ my_data: 1}).await?;
    /// # Ok::<(), worker::Error>(())
    /// # });
    /// ```
    pub async fn send<T, U: Into<SendMessage<T>>>(&self, message: U) -> Result<()>
    where
        T: Serialize,
    {
        let message: SendMessage<T> = message.into();
        let serialized_message = message.into_raw_send_message()?;
        self.send_raw(serialized_message).await
    }

    /// Sends a raw `JsValue` to the Queue.
    ///
    /// Use the `RawMessageBuilder` to create the message.
    pub async fn send_raw<T: Into<SendMessage<JsValue>>>(&self, message: T) -> Result<()> {
        let message: SendMessage<JsValue> = message.into();
        let options = match message.options {
            Some(options) => serde_wasm_bindgen::to_value(&options)?,
            None => JsValue::null(),
        };

        let fut: JsFuture = self.0.send(message.message, options)?.into();
        fut.await.map_err(Error::from)?;
        Ok(())
    }

    /// Sends a batch of messages to the Queue.
    ///
    /// Accepts an iterator that produces structs that are `serializable`.
    ///
    /// If message options are needed use the `BatchMessageBuilder` to create the batch.
    ///
    /// ## Example
    /// ```no_run
    /// # use serde::Serialize;
    /// # use wasm_bindgen::JsCast;
    /// # use js_sys::Object;
    /// #[derive(Serialize)]
    /// pub struct MyMessage {
    ///     my_data: u32,
    /// }
    /// # let env: worker::Env = Object::new().unchecked_into();
    /// # tokio_test::block_on(async {
    /// # let queue = env.queue("FOO")?;
    /// queue.send_batch(vec![MyMessage{ my_data: 1}]).await?;
    /// # Ok::<(), worker::Error>(())
    /// # });
    /// ```
    pub async fn send_batch<T: Serialize, U: Into<BatchSendMessage<T>>>(
        &self,
        messages: U,
    ) -> Result<()> {
        let messages: BatchSendMessage<T> = messages.into();
        let serialized_messages = messages.into_raw_batch_send_message()?;
        self.send_raw_batch(serialized_messages).await
    }

    /// Sends a batch of raw messages to the Queue.
    ///
    /// Accepts an iterator that produces structs that are `serializable`.
    ///
    /// If message options are needed use the `BatchMessageBuilder` to create the batch.
    pub async fn send_raw_batch<T: Into<BatchSendMessage<JsValue>>>(
        &self,
        messages: T,
    ) -> Result<()> {
        let messages: BatchSendMessage<JsValue> = messages.into();
        let batch_send_options = serde_wasm_bindgen::to_value(&messages.options)?;

        let messages = messages
            .body
            .into_iter()
            .map(|message: SendMessage<JsValue>| {
                let body = message.message;
                let message_send_request = js_sys::Object::new();

                js_sys::Reflect::set(&message_send_request, &"body".into(), &body)?;
                js_sys::Reflect::set(
                    &message_send_request,
                    &"contentType".into(),
                    &serde_wasm_bindgen::to_value(
                        &message.options.as_ref().map(|o| o.content_type),
                    )?,
                )?;
                js_sys::Reflect::set(
                    &message_send_request,
                    &"delaySeconds".into(),
                    &serde_wasm_bindgen::to_value(
                        &message.options.as_ref().map(|o| o.delay_seconds),
                    )?,
                )?;

                Ok::<JsValue, Error>(message_send_request.into())
            })
            .collect::<Result<js_sys::Array>>()?;

        let fut: JsFuture = self.0.send_batch(messages, batch_send_options)?.into();
        fut.await.map_err(Error::from)?;
        Ok(())
    }
}
