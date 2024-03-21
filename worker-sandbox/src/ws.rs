use super::SomeSharedData;
use futures_util::StreamExt;
use worker::{
    wasm_bindgen_futures, Env, Request, Response, Result, WebSocket, WebSocketPair, WebsocketEvent,
};

pub async fn handle_websocket(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    // Accept / handle a websocket connection
    let pair = WebSocketPair::new()?;
    let server = pair.server;
    server.accept()?;

    let some_namespace_kv = env.kv("SOME_NAMESPACE")?;

    wasm_bindgen_futures::spawn_local(async move {
        let mut event_stream = server.events().expect("could not open stream");

        while let Some(event) = event_stream.next().await {
            match event.expect("received error in websocket") {
                WebsocketEvent::Message(msg) => {
                    if let Some(text) = msg.text() {
                        server.send_with_str(text).expect("could not relay text");
                    }
                }
                WebsocketEvent::Close(_) => {
                    // Sets a key in a test KV so the integration tests can query if we
                    // actually got the close event. We can't use the shared dat a for this
                    // because miniflare resets that every request.
                    some_namespace_kv
                        .put("got-close-event", "true")
                        .unwrap()
                        .execute()
                        .await
                        .unwrap();
                }
            }
        }
    });

    Response::from_websocket(pair.client)
}

#[worker::send]
pub async fn handle_websocket_client(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let ws = WebSocket::connect("wss://echo.miniflare.mocks/".parse()?).await?;

    // It's important that we call this before we send our first message, otherwise we will
    // not have any event listeners on the socket to receive the echoed message.
    let mut event_stream = ws.events()?;

    ws.accept()?;
    ws.send_with_str("Hello, world!")?;

    while let Some(event) = event_stream.next().await {
        let event = event?;

        if let WebsocketEvent::Message(msg) = event {
            if let Some(text) = msg.text() {
                return Response::ok(text);
            }
        }
    }

    Response::error("never got a message echoed back :(", 500)
}
