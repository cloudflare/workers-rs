use crate::{
    counter::{LiveCounter, Update},
    error::Error,
    message::{Message, WsMessage},
    user::User,
};

use futures_util::{Future, StreamExt};
use worker::{wasm_bindgen_futures, Request, Response, WebSocketPair, WebsocketEvent};

pub enum Route {
    Counter,
    InvalidRoute,
}

impl Route {
    pub fn new(url: std::str::Split<'_, char>) -> Self {
        let paths = url.collect::<Box<[_]>>();
        if paths[0] != "chat" {
            return Self::InvalidRoute;
        }

        Self::Counter
    }

    pub fn invalid(_: Request) -> Result<Response, Error> {
        Ok(Response::error("Route not found", 400)?)
    }

    pub async fn websocket(counter: LiveCounter, req: &Request) -> Result<Response, Error> {
        let pair = WebSocketPair::new()?;
        // unique ws key
        let key = req.headers().get("sec-websocket-key").unwrap();
        let name = req.path().split('/').last().unwrap().to_owned();

        let WebSocketPair { client, server } = pair;
        let user = User::new(key.unwrap(), name, server);
        wasm_bindgen_futures::spawn_local(accept_connection(user, counter).await?);

        Ok(Response::from_websocket(client)?)
    }
}

async fn handle_connection(user: User, counter: LiveCounter) {
    let User { info, session } = user;
    if let Ok(mut stream) = session.events() {
        while let Some(Ok(event)) = stream.next().await {
            match event {
                // incoming messages
                WebsocketEvent::Message(msg) => {
                    let msg_ = msg.text();
                    let message = WsMessage::Send(msg_.as_deref());
                    if counter.broadcast(&Message::new(&info, &message)).is_err() {
                        return;
                    }
                }
                // received whenever a client disconnects
                WebsocketEvent::Close(_) => {
                    // remove the disconnected client from the existing connections
                    let remove = counter.remove(&info.id);
                    // decrease the number of connected clients and broadcast to existing connections
                    let update = counter.update(Update::Decrease, &info).await;

                    if remove.is_err() || update.is_err() {
                        return;
                    }
                }
            }
        }
    };
}

async fn accept_connection(
    user: User,
    counter: LiveCounter,
) -> Result<impl Future<Output = ()> + 'static, Error> {
    user.session.accept()?;
    // add the connection to existing connections
    counter.add(user.clone())?;
    // increase the number of connected clients and broadcast to existing connections
    counter.update(Update::Increase, &user.info).await?;

    // handle messages
    Ok(handle_connection(user, counter))
}
