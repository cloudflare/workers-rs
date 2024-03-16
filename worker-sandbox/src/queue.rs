use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{SomeSharedData, GLOBAL_QUEUE_STATE};
use worker::{
    console_log, event, Context, Env, MessageBatch, Request, Response, Result, RouteContext,
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
            message.body,
            message.id,
            message.timestamp.to_string()
        );
        guard.push(message.body);
    }
    Ok(())
}

pub async fn handle_queue_send(
    _req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    let id = match ctx
        .param("id")
        .map(|id| Uuid::try_parse(id).ok())
        .and_then(|u| u)
    {
        Some(id) => id,
        None => {
            return Response::error("Failed to parse id, expected a UUID", 400);
        }
    };
    let my_queue = match ctx.env.queue("my_queue") {
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

pub async fn handle_queue(_req: Request, _ctx: RouteContext<SomeSharedData>) -> Result<Response> {
    let guard = GLOBAL_QUEUE_STATE.lock().unwrap();
    let messages: Vec<QueueBody> = guard.clone();
    Response::from_json(&messages)
}
