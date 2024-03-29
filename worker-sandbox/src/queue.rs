use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{SomeSharedData, GLOBAL_QUEUE_STATE};
use worker::{
    console_log, event, Context, Env, MessageBatch, MessageExt, Request, Response, Result,
};
#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct QueueBody {
    pub id: Uuid,
    pub id_string: String,
}

#[event(queue)]
pub async fn queue(message_batch: MessageBatch<QueueBody>, _env: Env, _ctx: Context) -> Result<()> {
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
    let id = match segments
        .nth(2)
        .map(|id| Uuid::try_parse(id).ok())
        .and_then(|u| u)
    {
        Some(id) => id,
        None => {
            return Response::error("Failed to parse id, expected a UUID", 400);
        }
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
        Ok(_) => Response::ok("Message sent"),
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
