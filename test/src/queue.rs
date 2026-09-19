use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    SomeSharedData, GLOBAL_MAP_QUEUE_STATE, GLOBAL_QUEUE_METADATA_STATE, GLOBAL_QUEUE_STATE,
};
use worker::{event, Context, Env, Message, MessageBatch, MessageExt, Request, Response, Result};
#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct QueueBody {
    pub id: Uuid,
    pub id_string: String,
}

/// A message whose `id_string` starts with this prefix makes the consumer return `Err` on its
/// first delivery, so the runtime retries the batch.
const FAIL_ONCE_PREFIX: &str = "fail-once:";

/// Ids of the fail-once messages that have already failed. Like `GLOBAL_QUEUE_STATE`, this is
/// lost if the Wasm instance is reinitialized.
static FAILED_ONCE: Mutex<Vec<Uuid>> = Mutex::new(Vec::new());

/// A message with map-shaped fields, sent with the default (`json`) content type.
#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct MapQueueBody {
    pub id: Uuid,
    pub map: HashMap<String, String>,
    pub value: serde_json::Value,
}

#[derive(Serialize, Debug, Clone)]
pub struct QueueMessageMetadata {
    pub message_id: String,
    pub body_id: String,
    pub timestamp: String,
    pub attempts: u32,
}

// Not `worker::Error`, so the `Err` rejects through the `std::error::Error` fallback.
#[event(queue)]
pub async fn queue(
    message_batch: MessageBatch<QueueBody>,
    _env: Env,
    _ctx: Context,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if message_batch.queue() == "map_queue" {
        let mut guard = GLOBAL_MAP_QUEUE_STATE.lock().unwrap();
        for raw in message_batch.raw_iter() {
            let message = Message::<MapQueueBody>::try_from(raw)?;
            guard.push(message.into_body());
        }
        return Ok(());
    }
    let messages = message_batch.messages()?;
    {
        let mut failed = FAILED_ONCE.lock().unwrap();
        for message in &messages {
            let body = message.body();
            if body.id_string.starts_with(FAIL_ONCE_PREFIX) && !failed.contains(&body.id) {
                failed.push(body.id);
                return Err("fail-once message, retry the batch".into());
            }
        }
    }
    let mut guard = GLOBAL_QUEUE_STATE.lock().unwrap();
    let mut metadata_guard = GLOBAL_QUEUE_METADATA_STATE.lock().unwrap();
    for message in messages {
        let body = message.body();
        metadata_guard.push(QueueMessageMetadata {
            message_id: message.id().to_string(),
            body_id: body.id.to_string(),
            timestamp: message.timestamp().to_string(),
            attempts: message.attempts(),
        });
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

#[worker::send]
pub async fn handle_queue_metrics_send(
    _req: Request,
    env: Env,
    _: SomeSharedData,
) -> Result<Response> {
    let id = Uuid::new_v4();
    env.queue("metrics_queue")?
        .send(&QueueBody {
            id,
            id_string: id.to_string(),
        })
        .await?;
    Response::ok("Message sent")
}

#[worker::send]
pub async fn handle_queue_metrics(_req: Request, env: Env, _: SomeSharedData) -> Result<Response> {
    let metrics = env.queue("metrics_queue")?.metrics().await?;
    Response::from_json(&serde_json::json!({
        "backlog_count": metrics.backlog_count(),
        "backlog_bytes": metrics.backlog_bytes(),
        "oldest_message_timestamp": metrics.oldest_message_timestamp().map(|d| d.as_millis()),
    }))
}

#[worker::send]
pub async fn handle_map_queue_send(
    mut req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let message: MapQueueBody = match req.json().await {
        Ok(message) => message,
        Err(err) => {
            return Response::error(format!("Failed to parse request body: {err:?}"), 400);
        }
    };

    let map_queue = match env.queue("map_queue") {
        Ok(queue) => queue,
        Err(err) => return Response::error(format!("Failed to get queue: {err:?}"), 500),
    };

    match map_queue.send(message).await {
        Ok(()) => Response::ok("Message sent"),
        Err(err) => Response::error(format!("Failed to send message to queue: {err:?}"), 500),
    }
}

#[worker::send]
pub async fn handle_map_queue_send_batch(
    mut req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let messages: Vec<MapQueueBody> = match req.json().await {
        Ok(messages) => messages,
        Err(err) => {
            return Response::error(format!("Failed to parse request body: {err:?}"), 400);
        }
    };

    let map_queue = match env.queue("map_queue") {
        Ok(queue) => queue,
        Err(err) => return Response::error(format!("Failed to get queue: {err:?}"), 500),
    };

    match map_queue.send_batch(messages).await {
        Ok(()) => Response::ok("Message sent"),
        Err(err) => Response::error(
            format!("Failed to batch send message to queue: {err:?}"),
            500,
        ),
    }
}

pub async fn handle_map_queue(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let guard = GLOBAL_MAP_QUEUE_STATE.lock().unwrap();
    let messages: Vec<MapQueueBody> = guard.clone();
    Response::from_json(&messages)
}
