use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{SomeSharedData, GLOBAL_QUEUE_STATE};
use crate::Env;
use ts_gen_worker::*;

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct QueueBody {
    pub id: Uuid,
    pub id_string: String,
}

#[event(queue)]
pub async fn queue(message_batch: MessageBatch, _env: Env, _ctx: ExecutionContext) -> Result<()> {
    let mut guard = GLOBAL_QUEUE_STATE.lock().unwrap();
    for message in message_batch.messages() {
        let body: QueueBody = message.body_into()?;
        console_log!(
            "Received queue message {:?}, with id {} and timestamp: {}",
            body,
            message.id(),
            message.timestamp().to_string()
        );
        guard.push(body);
    }
    Ok(())
}

pub async fn handle_queue_send(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let uri = req.url_parsed()?;
    let mut segments = uri.path_segments().unwrap();
    let Some(id) = segments
        .nth(2)
        .map(|id| Uuid::try_parse(id).ok())
        .and_then(|u| u)
    else {
        return Response::builder().error("Failed to parse id, expected a UUID", 400);
    };

    let my_queue = env.my_queue();
    let body = serde_wasm_bindgen::to_value(&QueueBody {
        id,
        id_string: id.to_string(),
    })?;
    match my_queue.send(&body).await {
        Ok(_) => Response::builder().ok("Message sent"),
        Err(err) => {
            Response::builder().error(format!("Failed to send message to queue: {err:?}"), 500)
        }
    }
}

pub async fn handle_batch_send(req: Request, env: Env, _: SomeSharedData) -> Result<Response> {
    let messages: Vec<QueueBody> = match req.json_into().await {
        Ok(messages) => messages,
        Err(err) => {
            return Response::builder()
                .error(format!("Failed to parse request body: {err:?}"), 400);
        }
    };

    let my_queue = env.my_queue();

    let batch = messages
        .iter()
        .map(|message| {
            Ok(MessageSendRequest::new(&serde_wasm_bindgen::to_value(
                message,
            )?))
        })
        .collect::<Result<Vec<_>>>()?;

    match my_queue.send_batch(&batch).await {
        Ok(_) => Response::builder().ok("Message sent"),
        Err(err) => Response::builder().error(
            format!("Failed to batch send message to queue: {err:?}"),
            500,
        ),
    }
}

pub async fn handle_queue(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let guard = GLOBAL_QUEUE_STATE.lock().unwrap();
    let messages: Vec<QueueBody> = guard.clone();
    Response::builder().from_json(&messages)
}
