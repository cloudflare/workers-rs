#[cfg(feature = "http")]
use std::convert::TryInto;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use futures_util::StreamExt;
use wasm_bindgen::{throw_str, UnwrapThrowExt};
use wasm_bindgen_futures::spawn_local;
use worker::*;

use crate::SomeSharedData;

#[durable_object]
pub struct EchoContainer {
    state: State,
    ready: Arc<AtomicBool>,
}

impl DurableObject for EchoContainer {
    fn new(state: State, _env: Env) -> Self {
        let container = state.container().expect_throw("failed to get container");
        if !container.running() {
            container
                .start(None)
                .expect_throw("failed to start container");
        }
        let ready = Arc::new(AtomicBool::new(false));
        let ready_clone = Arc::clone(&ready);
        spawn_local(async move {
            for _ in 0..10 {
                match container
                    .get_tcp_port(8080)
                    .expect_throw("failed to get tcp port 8080")
                    .fetch("http://container.miniflare.mocks/ping", None)
                    .await
                {
                    Ok(resp) => {
                        #[cfg(feature = "http")]
                        if !resp.status().is_success() {
                            throw_str("failed to fetch ping: bad status");
                        }
                        #[cfg(not(feature = "http"))]
                        if !(200..300).contains(&resp.status_code()) {
                            throw_str("failed to fetch ping: bad status");
                        }
                        ready_clone.store(true, Ordering::Release);
                        return;
                    }
                    Err(e) if e.to_string().contains("container port not found") => {
                        Delay::from(Duration::from_millis(300)).await;
                        continue;
                    }
                    Err(e) => throw_str(&format!("failed to fetch ping: {e}")),
                }
            }
            throw_str("failed to fetch ping");
        });
        Self { state, ready }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        while !self.ready.load(Ordering::Acquire) {
            Delay::from(Duration::from_millis(100)).await;
        }
        match self.state.container() {
            Some(container) => match container.get_tcp_port(8080)?.fetch_request(req).await {
                Ok(resp) => {
                    #[cfg(feature = "http")]
                    let resp = resp.try_into()?;
                    Ok(resp)
                }
                Err(e) => Err(e),
            },
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
                "http://container.miniflare.mocks/echo",
                RequestInit::new()
                    .with_method(Method::Post)
                    .with_body(Some(body.into())),
            )?;
            stub.fetch_with_request(req).await
        }
        Method::Get => {
            let WebSocketPair { server, client } = WebSocketPair::new()?;
            server.accept()?;
            let mut req = Request::new("http://container.miniflare.mocks/ws", Method::Get)?;
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
