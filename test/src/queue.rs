use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{SomeSharedData, GLOBAL_QUEUE_METADATA_STATE, GLOBAL_QUEUE_STATE};
use worker::{event, Context, Env, MessageBatch, MessageExt, Request, Response, Result};
#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct QueueBody {
    pub id: Uuid,
    pub id_string: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct QueueMessageMetadata {
    pub message_id: String,
    pub body_id: String,
    pub timestamp: String,
    pub attempts: u32,
}

#[event(queue)]
pub async fn queue(message_batch: MessageBatch<QueueBody>, _env: Env, _ctx: Context) -> Result<()> {
    let mut guard = GLOBAL_QUEUE_STATE.lock().unwrap();
    let mut metadata_guard = GLOBAL_QUEUE_METADATA_STATE.lock().unwrap();
    for message in message_batch.messages()? {
        let body = message.body();
        let metadata = QueueMessageMetadata {
            message_id: message.id().to_string(),
            body_id: body.id.to_string(),
            timestamp: message.timestamp().to_string(),
            attempts: message.attempts(),
        };
        metadata_guard.push(metadata);
        guard.push(message.into_body());
    }
    Ok(())
}

pub async fn handle_queue_metadata(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let guard = GLOBAL_QUEUE_METADATA_STATE.lock().unwrap();
    let metadata = guard.clone();

    Response::from_json(&metadata)
}

#[worker::send]
pub async fn handle_queue_send(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let uri = req.url()?;
    let mut segments = uri.path_segments().unwrap();
    let Some(id) = segments
        .nth(2)
        .map(|id| Uuid::try_parse(id).ok())
        .and_then(|u| u)
    else {
        return Response::error("Failed to parse id, expected a UUID", 400);
    };

    let my_queue = match env.queue("my_queue") {
        Ok(queue) => queue,
        Err(err) => return Response::error(format!("Failed to get queue: {err:?}"), 500),
    };
    match my_queue
        .send(&QueueBody {
            id,
            id_string: id.to_string(),
        })
        .await
    {
        Ok(()) => Response::ok("Message sent"),
        Err(err) => Response::error(format!("Failed to send message to queue: {err:?}"), 500),
    }
}

#[worker::send]
pub async fn handle_batch_send(mut req: Request, env: Env, _: SomeSharedData) -> Result<Response> {
    let messages: Vec<QueueBody> = match req.json().await {
        Ok(messages) => messages,
        Err(err) => {
            return Response::error(format!("Failed to parse request body: {err:?}"), 400);
        }
    };

    let my_queue = match env.queue("my_queue") {
        Ok(queue) => queue,
        Err(err) => return Response::error(format!("Failed to get queue: {err:?}"), 500),
    };

    match my_queue.send_batch(messages).await {
        Ok(()) => Response::ok("Message sent"),
        Err(err) => Response::error(
            format!("Failed to batch send message to queue: {err:?}"),
            500,
        ),
    }
}

pub async fn handle_queue(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let guard = GLOBAL_QUEUE_STATE.lock().unwrap();
    let messages: Vec<QueueBody> = guard.clone();
    Response::from_json(&messages)
}
