use crate::Env;
use std::cell::Cell;
use ts_gen_worker::*;

use crate::util::{message_text, on_websocket_events, WebsocketEvent};
use crate::SomeSharedData;

#[durable_object]
pub struct Counter {
    count: Cell<usize>,
    unstored_count: Cell<usize>,
    state: DurableObjectState,
    initialized: Cell<bool>,
    env: Env,
}

impl DurableObject for Counter {
    type Env = Env;

    fn new(state: DurableObjectState, env: Env) -> Counter {
        Self {
            count: Cell::new(0),
            unstored_count: Cell::new(0),
            initialized: Cell::new(false),
            state,
            env,
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        if !self.initialized.get() {
            self.initialized.set(true);
            self.count
                .set(self.state.storage().get_into("count").await?.unwrap_or(0));
        }

        if req.path().eq("/ws") {
            let (client, server) = WebSocketPair::new()?.into();
            self.state.accept_web_socket(&server)?;
            server.serialize_attachment(&"hello".into())?;

            return Response::builder().from_websocket(client);
        }

        self.unstored_count.set(self.unstored_count.get() + 1);
        self.count.set(self.count.get() + 10);
        let count = self.count.get();
        self.state.storage().put_serialized("count", &count).await?;

        Response::builder().ok(format!(
            "[durable_object]: self.count: {}, self.unstored_count: {}, secret value: {}",
            self.count.get(),
            self.unstored_count.get(),
            self.env.some_secret()
        ))
    }

    async fn websocket_message(&self, ws: WebSocket, _message: DataKind) -> Result<()> {
        let _attach: String = ws
            .deserialize_attachment()?
            .and_then(|v| v.as_string())
            .expect("websockets should have an attachment");
        let mut count: usize = self.state.storage().get_into("count").await?.unwrap_or(0);
        count += 10;
        self.state.storage().put_serialized("count", &count).await?;
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

fn counter_namespace(req: &Request, env: &Env) -> Result<DurableObjectNamespace> {
    if req.path().contains("shared") {
        Err("SHARED_COUNTER is not bound".into())
    } else {
        Ok(env.counter())
    }
}

pub async fn handle_id(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let namespace = counter_namespace(&req, &env).expect("DAWJKHDAD");
    let stub = namespace.get(&namespace.id_from_name("A")?)?;
    stub.fetch_with_str("https://fake-host/").await
}

pub async fn handle_websocket(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let (client, server) = WebSocketPair::new()?.into();
    server.accept()?;

    let namespace = counter_namespace(&req, &env).expect("failed to get namespace");
    let stub = namespace.get(&namespace.id_from_name("A")?)?;
    let headers = Headers::new()?;
    headers.set("upgrade", "websocket")?;
    let req = Request::new_with_str_and_init(
        "https://fake-host/ws",
        &RequestInit::builder()
            .method("GET")
            .headers(&headers)
            .build(),
    )?;

    let res = stub.fetch(&req).await?;
    let do_ws = res.web_socket().expect("server did not accept websocket");
    do_ws.accept()?;

    {
        let do_ws = do_ws.clone();
        on_websocket_events(&server, move |event| match event {
            WebsocketEvent::Message(msg) => {
                if let Some(text) = message_text(&msg) {
                    do_ws.send_with_str(&text).expect("could not relay text");
                }
            }
            WebsocketEvent::Close(_) => {
                let _res = do_ws.close_with_code_and_reason(1000, "client closed");
            }
        })?;
    }
    {
        let server = server.clone();
        on_websocket_events(&do_ws, move |event| match event {
            WebsocketEvent::Message(msg) => {
                if let Some(text) = message_text(&msg) {
                    server.send_with_str(&text).expect("could not relay text");
                }
            }
            WebsocketEvent::Close(_) => {
                let _res = server.close_with_code_and_reason(1000, "durable closed");
            }
        })?;
    }

    Response::builder().from_websocket(client)
}
