use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{SomeSharedData, GLOBAL_MAP_QUEUE_STATE, GLOBAL_QUEUE_STATE};
use worker::{
    console_log, event, Context, Env, Message, MessageBatch, MessageExt, Request, Response, Result,
};
#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct QueueBody {
    pub id: Uuid,
    pub id_string: String,
}

/// A message with map-shaped fields, sent with the default (`json`) content type.
#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct MapQueueBody {
    pub id: Uuid,
    pub map: HashMap<String, String>,
    pub value: serde_json::Value,
}

#[event(queue)]
pub async fn queue(message_batch: MessageBatch<QueueBody>, _env: Env, _ctx: Context) -> Result<()> {
    if message_batch.queue() == "map_queue" {
        let mut guard = GLOBAL_MAP_QUEUE_STATE.lock().unwrap();
        for raw in message_batch.raw_iter() {
            let message = Message::<MapQueueBody>::try_from(raw)?;
            guard.push(message.into_body());
        }
        return Ok(());
    }
    let mut guard = GLOBAL_QUEUE_STATE.lock().unwrap();
    for message in message_batch.messages()? {
        console_log!(
            "Received queue message {:?}, with id {} and timestamp: {}",
            message.body(),
            message.id(),
            message.timestamp().to_string()
        );
        guard.push(message.into_body());
    }
    Ok(())
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
