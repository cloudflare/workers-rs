use crate::{
    counter::LiveCounter,
    error::Error,
    message::{Data, MessageEvent, MsgResponse, SendData, WsMessage},
    storage::Update,
    timeout::{HeartBeat, TimeoutHandle},
    user::{User, Users},
};

use futures_util::{Future, StreamExt};
use worker::{wasm_bindgen_futures, Headers, Request, Response, WebSocketPair};

const SEC_WEBSOCKET_KEY: &str = "sec-websocket-key";

pub struct RouteInfo(String, String);

impl RouteInfo {
    fn new(name: &str, headers: &Headers) -> Result<Self, Error> {
        let id = headers
            .get(SEC_WEBSOCKET_KEY)?
            .ok_or_else(|| Error::from(("sec-websocket-key not found in headers.", 500)))?;

        Ok(Self(name.to_owned(), id))
    }
}

pub enum Route {
    Chat(RouteInfo),
    InvalidRoute,
}

impl Route {
    /// Returns a Route based on the request's [`worker::Url`].
    ///
    /// # Errors
    ///
    /// Error if url is cannot-be-a-base URLs or
    /// `sec-websocket-key` is missing from request headers.
    ///
    pub fn new(req: &Request) -> Result<Self, Error> {
        let url = req.url()?;
        let paths = url
            .path_segments()
            .ok_or_else(|| Error::from(("Paths are empty.", 500)))?;

        // url should look like /chat/:/name thus the array will be ["chat", "{name}"].
        let paths = paths.collect::<Box<[_]>>();
        if paths.first() != Some(&"chat") {
            return Ok(Self::InvalidRoute);
        }

        Ok(Self::Chat(RouteInfo::new(paths[1], req.headers())?))
    }

    pub fn invalid() -> Result<Response, Error> {
        Ok(Response::error("Route not found.", 400)?)
    }

    pub async fn websocket(counter: LiveCounter, info: RouteInfo) -> Result<Response, Error> {
        let WebSocketPair { client, server } = WebSocketPair::new()?;
        let RouteInfo(name, id) = info;
        let user = User::new(id, name, server);
        wasm_bindgen_futures::spawn_local(accept_connection(user, counter).await?);
        Ok(Response::from_websocket(client)?)
    }
}

fn keep_alive(users: Users, id: Box<str>) {
    TimeoutHandle::new((users, id), 5_000, 5_000)
        .max_heart_beat(5)
        .run();
}

async fn handle_connection(user: User, counter: LiveCounter) {
    keep_alive(counter.users().clone(), (&*user.info.id).into());
    let events = user.session.events();
    if let Ok(mut stream) = events {
        while let Some(Ok(event)) = stream.next().await {
            match MessageEvent::new(event) {
                MessageEvent::Close(..) => return handle_disconnect(&counter, &user).await,
                MessageEvent::Message(msg) => handle_message(msg, &counter, &user),
                MessageEvent::Ping => {}
                MessageEvent::Pong => counter.users().pong(&user.info.id),
            }
        }
    };
}

async fn handle_disconnect(counter: &LiveCounter, user: &User) {
    // remove the disconnected client from the existing connections
    counter.remove(&user.info.id);
    // decrease the number of connected clients and broadcast to existing connections
    let count = counter.update(Update::Decrease);
    let message = &WsMessage::Conn(count.await.ok());

    counter.broadcast(&SendData::Text(
        (&*MsgResponse::new(&user.info, message).as_string()).into(),
    ));
}

fn handle_message(msg: Data, counter: &LiveCounter, user: &User) {
    match msg {
        Data::Text(text) => {
            let message = WsMessage::Send(Some(&text));
            counter.broadcast(&SendData::Text(
                (&*MsgResponse::new(&user.info, &message).as_string()).into(),
            ));
        }
        Data::None | Data::Binary(_) => {}
    }
}

async fn accept_connection(
    user: User,
    counter: LiveCounter,
) -> Result<impl Future<Output = ()> + 'static, Error> {
    // accept and add the connection to existing connections
    user.session.accept()?;
    counter.add(user.clone())?;
    // increase the number of connected clients and broadcast to existing connections
    let count = counter.update(Update::Increase);
    let message = &WsMessage::Conn(Some(count.await?));
    counter.broadcast(&SendData::Text(
        (&*MsgResponse::new(&user.info, message).as_string()).into(),
    ));
    // handle messages
    Ok(handle_connection(user, counter))
}
