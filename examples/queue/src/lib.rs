use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use worker::*;

const MY_MESSAGES_BINDING_NAME: &str = "my_messages";
const MY_MESSAGES_QUEUE_NAME: &str = "mymessages";

const RAW_MESSAGES_BINDING_NAME: &str = "raw_messages";
const RAW_MESSAGES_QUEUE_NAME: &str = "rawmessages";

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct MyType {
    foo: String,
    bar: u32,
}

#[event(fetch)]
async fn main(_req: Request, env: Env, _: worker::Context) -> Result<Response> {
    let my_messages_queue = env.queue(MY_MESSAGES_BINDING_NAME)?;
    let raw_messages_queue = env.queue(RAW_MESSAGES_BINDING_NAME)?;

    // Send a message with using a serializable struct
    my_messages_queue
        .send(MyType {
            foo: "Hello world".into(),
            bar: 1,
        })
        .await?;

    // Send a batch of messages using some sort of iterator
    my_messages_queue
        .send_batch([
            // Use the MessageBuilder to set additional options
            MessageBuilder::new(MyType {
                foo: "Hello world".into(),
                bar: 2,
            })
            .delay_seconds(20)
            .build(),
            // Send a message with using a serializable struct
            MyType {
                foo: "Hello world".into(),
                bar: 4,
            }
            .into(),
        ])
        .await?;

    // Send a batch of messages using the BatchMessageBuilder
    my_messages_queue
        .send_batch(
            BatchMessageBuilder::new()
                .message(MyType {
                    foo: "Hello world".into(),
                    bar: 4,
                })
                .messages(vec![
                    MyType {
                        foo: "Hello world".into(),
                        bar: 5,
                    },
                    MyType {
                        foo: "Hello world".into(),
                        bar: 6,
                    },
                ])
                .delay_seconds(10)
                .build(),
        )
        .await?;

    // Send a raw JSValue
    raw_messages_queue
        .send_raw(
            // RawMessageBuilder has to be used as we should set content type of these raw messages
            RawMessageBuilder::new(JsValue::from_str("7"))
                .delay_seconds(30)
                .build_with_content_type(QueueContentType::Json),
        )
        .await?;

    // Send a batch of raw JSValues using the BatchMessageBuilder
    raw_messages_queue
        .send_raw_batch(
            BatchMessageBuilder::new()
                .message(
                    RawMessageBuilder::new(js_sys::Date::new_0().into())
                        .build_with_content_type(QueueContentType::V8),
                )
                .message(
                    RawMessageBuilder::new(JsValue::from_str("8"))
                        .build_with_content_type(QueueContentType::Json),
                )
                .delay_seconds(10)
                .build(),
        )
        .await?;

    // Send a batch of raw JsValues using some sort of iterator
    raw_messages_queue
        .send_raw_batch(vec![RawMessageBuilder::new(JsValue::from_str("9"))
            .delay_seconds(20)
            .build_with_content_type(QueueContentType::Text)])
        .await?;

    Response::empty()
}

// Consumes messages from `my_messages` queue and `raw_messages` queue
#[event(queue)]
pub async fn main(message_batch: MessageBatch<MyType>, _: Env, _: Context) -> Result<()> {
    match message_batch.queue().as_str() {
        MY_MESSAGES_QUEUE_NAME => {
            for message in message_batch.messages()? {
                console_log!(
                    "Got message {:?}, with id {} and timestamp: {}",
                    message.body(),
                    message.id(),
                    message.timestamp().to_string(),
                );
                if message.body().bar == 1 {
                    message.retry_with_options(
                        &QueueRetryOptionsBuilder::new()
                            .with_delay_seconds(10)
                            .build(),
                    );
                } else {
                    message.ack();
                }
            }
        }
        RAW_MESSAGES_QUEUE_NAME => {
            for message in message_batch.raw_iter() {
                console_log!(
                    "Got raw message {:?}, with id {} and timestamp: {}",
                    message.body(),
                    message.id(),
                    message.timestamp().to_string(),
                );
            }
            message_batch.ack_all();
        }
        _ => {
            console_error!("Unknown queue: {}", message_batch.queue());
        }
    }

    Ok(())
}
