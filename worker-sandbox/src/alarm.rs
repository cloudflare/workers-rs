use std::time::Duration;
use tokio_stream::{StreamExt, StreamMap};

use worker::*;

use super::SomeSharedData;

#[durable_object]
pub struct AlarmObject {
    state: State,
}

#[durable_object]
impl DurableObject for AlarmObject {
    fn new(state: State, _: Env) -> Self {
        Self { state }
    }

    async fn fetch(&mut self, _: Request) -> Result<Response> {
        let alarmed: bool = match self.state.storage().get("alarmed").await {
            Ok(alarmed) => alarmed,
            Err(e) if e.to_string() == "No such value in storage." => {
                // Trigger our alarm method in 100ms.
                self.state
                    .storage()
                    .set_alarm(Duration::from_millis(100))
                    .await?;

                false
            }
            Err(e) => return Err(e),
        };

        Response::ok(alarmed.to_string())
    }

    async fn alarm(&mut self) -> Result<Response> {
        self.state.storage().put("alarmed", true).await?;

        console_log!("Alarm has been triggered!");

        Response::ok("ALARMED")
    }
}

#[worker::send]
pub async fn handle_alarm(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let namespace = env.durable_object("ALARM")?;
    let stub = namespace.id_from_name("alarm")?.get_stub()?;
    // when calling fetch to a Durable Object, a full URL must be used. Alternatively, a
    // compatibility flag can be provided in wrangler.toml to opt-in to older behavior:
    // https://developers.cloudflare.com/workers/platform/compatibility-dates#durable-object-stubfetch-requires-a-full-url
    stub.fetch_with_str("https://fake-host/alarm").await
}

#[worker::send]
pub async fn handle_id(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let namespace = env.durable_object("COUNTER").expect("DAWJKHDAD");
    let stub = namespace.id_from_name("A")?.get_stub()?;
    // when calling fetch to a Durable Object, a full URL must be used. Alternatively, a
    // compatibility flag can be provided in wrangler.toml to opt-in to older behavior:
    // https://developers.cloudflare.com/workers/platform/compatibility-dates#durable-object-stubfetch-requires-a-full-url
    stub.fetch_with_str("https://fake-host/").await
}

#[worker::send]
pub async fn handle_put_raw(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let namespace = env.durable_object("PUT_RAW_TEST_OBJECT")?;
    let id = namespace.unique_id()?;
    let stub = id.get_stub()?;
    stub.fetch_with_request(req).await
}

#[worker::send]
pub async fn handle_websocket(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    // Accept / handle a websocket connection
    let pair = WebSocketPair::new()?;
    let server = pair.server;
    server.accept()?;

    // Connect to Durable Object via WS
    let namespace = env
        .durable_object("COUNTER")
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
