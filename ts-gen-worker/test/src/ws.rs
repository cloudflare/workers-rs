use super::SomeSharedData;
use crate::util::{message_text, on_websocket_events, websocket_events, WebsocketEvent};
use crate::Env;
use futures_util::StreamExt;
use ts_gen_worker::*;
use wasm_bindgen_futures::spawn_local;

pub async fn handle_websocket(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let (client, server) = WebSocketPair::new()?.into();
    server.accept()?;

    let some_namespace_kv = env.some_namespace();

    {
        let server = server.clone();
        on_websocket_events(&server.clone(), move |event| match event {
            WebsocketEvent::Message(msg) => {
                if let Some(text) = message_text(&msg) {
                    server.send_with_str(&text).expect("could not relay text");
                }
            }
            WebsocketEvent::Close(_) => {
                // Sets a key in a test KV so the integration tests can query if we
                // actually got the close event.
                let kv = some_namespace_kv.clone();
                spawn_local(async move {
                    kv.put("got-close-event", "true").await.unwrap();
                });
            }
        })?;
    }

    Response::builder().from_websocket(client)
}

/// `worker::WebSocket::connect`: an upgrade fetch whose response carries the socket.
async fn connect(url: &str) -> Result<WebSocket> {
    let url = url
        .replacen("wss://", "https://", 1)
        .replacen("ws://", "http://", 1);
    let headers = Headers::new()?;
    headers.set("upgrade", "websocket")?;
    let init = RequestInit::builder()
        .method("GET")
        .headers(&headers)
        .build();
    let res = fetch_with_str_and_init(&url, &init).await?;
    res.web_socket()
        .ok_or_else(|| Error::from("server did not accept"))
}

pub async fn handle_websocket_client(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let ws = connect("wss://echo.miniflare.mocks/").await?;

    // Listeners must be attached before the first send so the echo is observed.
    let mut event_stream = websocket_events(&ws)?;

    ws.accept()?;
    ws.send_with_str("Hello, world!")?;

    while let Some(event) = event_stream.next().await {
        if let WebsocketEvent::Message(msg) = event {
            if let Some(text) = message_text(&msg) {
                return Response::builder().ok(text);
            }
        }
    }

    Response::builder().error("never got a message echoed back :(", 500)
}
