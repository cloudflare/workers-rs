use futures_util::StreamExt;
use wasm_bindgen::{throw_str, UnwrapThrowExt};
use wasm_bindgen_futures::spawn_local;
use worker::*;

use crate::SomeSharedData;

#[durable_object]
pub struct EchoContainer {
    state: State,
}

impl DurableObject for EchoContainer {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        match self.state.container() {
            Some(container) => container.get_tcp_port(8080)?.fetch_request(req).await,
            None => Response::error("No container", 500),
        }
    }
}

const CONTAINER_NAME: &str = "my-container";

#[worker::send]
pub async fn handle_container(
    mut req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("ECHO_CONTAINER")?;
    let id = namespace.id_from_name(CONTAINER_NAME)?;
    let stub = id.get_stub()?;
    match req.method() {
        Method::Post => {
            let body = req.text().await?;
            let req = Request::new_with_init(
                "https://fake-host/echo",
                RequestInit::new()
                    .with_method(Method::Post)
                    .with_body(Some(body.into())),
            )?;
            stub.fetch_with_request(req).await
        }
        Method::Get => {
            let WebSocketPair { server, client } = WebSocketPair::new()?;
            server.accept()?;

            let mut req = Request::new("https://fake-host/ws", Method::Get)?;
            req.headers_mut()?.set("upgrade", "websocket")?;

            let resp = stub.fetch_with_request(req).await?;
            let ws = match resp.websocket() {
                Some(ws) => ws,
                None => return Response::error("Expected websocket response", 500),
            };
            ws.accept()?;
            spawn_local(redir_websocket(ws, server));
            Response::from_websocket(client)
        }
        _ => Response::error("Container method not allowed", 405),
    }
}

async fn redir_websocket(dst: WebSocket, src: WebSocket) {
    let mut src_events = src.events().expect_throw("could not open src events");
    let mut dst_events = dst.events().expect_throw("could not open dst events");

    while let Some(event) = src_events.next().await {
        match event.expect_throw("received error in src websocket") {
            WebsocketEvent::Message(msg) => {
                dst.send_with_str(msg.text().expect_throw("expect a text message from src"))
                    .expect_throw("failed to send to dst");
                if let Some(Ok(WebsocketEvent::Message(msg))) = dst_events.next().await {
                    src.send_with_str(msg.text().expect_throw("expect a text message from dst"))
                        .expect_throw("failed to send to src");
                } else {
                    throw_str("expect a text message from dst");
                }
            }
            WebsocketEvent::Close(reason) => {
                dst.close(Some(reason.code()), Some(reason.reason()))
                    .expect_throw("failed to close dst websocket");
            }
        }
    }
}
