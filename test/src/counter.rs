use std::cell::RefCell;
use tokio_stream::{StreamExt, StreamMap};
use worker::{
    durable_object, wasm_bindgen, wasm_bindgen_futures, Env, Error, Method, Request, Response,
    ResponseBuilder, Result, State, WebSocket, WebSocketIncomingMessage, WebSocketPair,
    WebsocketEvent,
};

use crate::SomeSharedData;

#[durable_object]
pub struct Counter {
    count: RefCell<usize>,
    unstored_count: RefCell<usize>,
    state: State,
    initialized: RefCell<bool>,
    env: Env,
}

impl DurableObject for Counter {
    fn new(state: State, env: Env) -> Self {
        Self {
            count: RefCell::new(0),
            unstored_count: RefCell::new(0),
            initialized: RefCell::new(false),
            state,
            env,
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        if !*self.initialized.borrow() {
            *self.initialized.borrow_mut() = true;
            *self.count.borrow_mut() = self.state.storage().get("count").await.unwrap_or(0);
        }

        if req.path().eq("/ws") {
            let pair = WebSocketPair::new()?;
            let server = pair.server;
            // accept websocket with hibernation api
            self.state.accept_web_socket(&server);
            server
                .serialize_attachment("hello")
                .expect("failed to serialize attachment");

            return Ok(ResponseBuilder::new()
                .with_status(101)
                .with_websocket(pair.client)
                .empty());
        }

        *self.unstored_count.borrow_mut() += 1;
        *self.count.borrow_mut() += 10;
        let count = *self.count.borrow();
        self.state.storage().put("count", count).await?;

        Response::ok(format!(
            "[durable_object]: self.count: {}, self.unstored_count: {}, secret value: {}",
            self.count.borrow(),
            self.unstored_count.borrow(),
            self.env.secret("SOME_SECRET")?
        ))
    }

    async fn websocket_message(
        &self,
        ws: WebSocket,
        _message: WebSocketIncomingMessage,
    ) -> Result<()> {
        let _attach: String = ws
            .deserialize_attachment()?
            .expect("websockets should have an attachment");
        // get and increment storage by 10
        let mut count: usize = self.state.storage().get("count").await.unwrap_or(0);
        count += 10;
        self.state.storage().put("count", count).await?;
        // send value to client
        ws.send_with_str(format!("{count}"))
            .expect("failed to send value to client");
        Ok(())
    }

    async fn websocket_close(
        &self,
        _ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> Result<()> {
        Ok(())
    }

    async fn websocket_error(&self, _ws: WebSocket, _error: Error) -> Result<()> {
        Ok(())
    }
}

#[worker::send]
pub async fn handle_id(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let durable_object_name = if req.path().contains("shared") {
        "SHARED_COUNTER"
    } else {
        "COUNTER"
    };
    let namespace = env.durable_object(durable_object_name).expect("DAWJKHDAD");
    let stub = namespace.id_from_name("A")?.get_stub()?;
    // when calling fetch to a Durable Object, a full URL must be used. Alternatively, a
    // compatibility flag can be provided in wrangler.toml to opt-in to older behavior:
    // https://developers.cloudflare.com/workers/platform/compatibility-dates#durable-object-stubfetch-requires-a-full-url
    stub.fetch_with_str("https://fake-host/").await
}

#[worker::send]
pub async fn handle_websocket(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let durable_object_name = if req.path().contains("shared") {
        "SHARED_COUNTER"
    } else {
        "COUNTER"
    };
    // Accept / handle a websocket connection
    let pair = WebSocketPair::new()?;
    let server = pair.server;
    server.accept()?;

    // Connect to Durable Object via WS
    let namespace = env
        .durable_object(durable_object_name)
        .expect("failed to get namespace");
    let stub = namespace.id_from_name("A")?.get_stub()?;
    let mut req = Request::new("https://fake-host/ws", Method::Get)?;
    req.headers_mut()?.set("upgrade", "websocket")?;

    let res = stub.fetch_with_request(req).await?;
    let do_ws = res.websocket().expect("server did not accept websocket");
    do_ws.accept()?;

    wasm_bindgen_futures::spawn_local(async move {
        let event_stream = server.events().expect("could not open stream");
        let do_event_stream = do_ws.events().expect("could not open stream");

        let mut map = StreamMap::new();
        map.insert("client", event_stream);
        map.insert("durable", do_event_stream);

        while let Some((key, event)) = map.next().await {
            match key {
                "client" => match event.expect("received error in websocket") {
                    WebsocketEvent::Message(msg) => {
                        if let Some(text) = msg.text() {
                            do_ws.send_with_str(text).expect("could not relay text");
                        }
                    }
                    WebsocketEvent::Close(_) => {
                        let _res = do_ws.close(Some(1000), Some("client closed".to_string()));
                    }
                },
                "durable" => match event.expect("received error in websocket") {
                    WebsocketEvent::Message(msg) => {
                        if let Some(text) = msg.text() {
                            server.send_with_str(text).expect("could not relay text");
                        }
                    }
                    WebsocketEvent::Close(_) => {
                        let _res = server.close(Some(1000), Some("durable closed".to_string()));
                    }
                },
                _ => unreachable!(),
            }
        }
    });

    Response::from_websocket(pair.client)
}
