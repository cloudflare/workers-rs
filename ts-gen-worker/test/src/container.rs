use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use futures_util::StreamExt;
use ts_gen_worker::*;
use wasm_bindgen::{throw_str, UnwrapThrowExt};
use wasm_bindgen_futures::spawn_local;

use crate::util::{message_text, sleep, websocket_events, WebsocketEvent};
use crate::{Env, SomeSharedData};

#[durable_object]
pub struct EchoContainer {
    state: DurableObjectState,
    ready: Arc<AtomicBool>,
}

impl DurableObject for EchoContainer {
    type Env = Env;

    fn new(state: DurableObjectState, _env: Env) -> Self {
        let container = state.container().expect_throw("failed to get container");
        if !container.running() {
            container.start().expect_throw("failed to start container");
        }
        let ready = Arc::new(AtomicBool::new(false));
        let ready_clone = Arc::clone(&ready);
        spawn_local(async move {
            for _ in 0..10 {
                match container
                    .get_tcp_port(8080)
                    .expect_throw("failed to get tcp port 8080")
                    .fetch_with_str("http://container.miniflare.mocks/ping")
                    .await
                {
                    Ok(resp) => {
                        if !(200..300).contains(&resp.status()) {
                            throw_str("failed to fetch ping: bad status");
                        }
                        ready_clone.store(true, Ordering::Release);
                        return;
                    }
                    // workerd's message for a not-yet-reachable port varies by version.
                    Err(_) => sleep(300.0).await,
                }
            }
            throw_str("failed to fetch ping");
        });
        Self { state, ready }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        while !self.ready.load(Ordering::Acquire) {
            sleep(100.0).await;
        }
        match self.state.container() {
            Some(container) => container.get_tcp_port(8080)?.fetch(&req).await,
            None => Response::builder().error("No container", 500),
        }
    }
}

const CONTAINER_NAME: &str = "my-container";

pub async fn handle_container(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let namespace = env.echo_container();
    let id = namespace.id_from_name(CONTAINER_NAME)?;
    let stub = namespace.get(&id)?;
    match req.method().as_str() {
        "POST" => {
            let body = req.text().await?;
            let req = Request::new_with_str_and_init(
                "http://container.miniflare.mocks/echo",
                &RequestInit::builder()
                    .method("POST")
                    .body_with_str(body)
                    .build(),
            )?;
            stub.fetch(&req).await
        }
        "GET" => {
            let (client, server) = WebSocketPair::new()?.into();
            server.accept()?;
            let headers = Headers::new()?;
            headers.set("upgrade", "websocket")?;
            let req = Request::new_with_str_and_init(
                "http://container.miniflare.mocks/ws",
                &RequestInit::builder()
                    .method("GET")
                    .headers(&headers)
                    .build(),
            )?;
            let resp = stub.fetch(&req).await?;
            let Some(ws) = resp.web_socket() else {
                return Response::builder().error("Expected websocket response", 500);
            };
            ws.accept()?;
            spawn_local(redir_websocket(ws, server));
            Response::builder().status(101).web_socket(&client).empty()
        }
        _ => Response::builder().error("Container method not allowed", 405),
    }
}

async fn redir_websocket(dst: WebSocket, src: WebSocket) {
    let mut src_events = websocket_events(&src).expect_throw("could not open src events");
    let mut dst_events = websocket_events(&dst).expect_throw("could not open dst events");

    while let Some(event) = src_events.next().await {
        match event {
            WebsocketEvent::Message(msg) => {
                dst.send_with_str(
                    message_text(&msg).expect_throw("expect a text message from src"),
                )
                .expect_throw("failed to send to dst");
                if let Some(WebsocketEvent::Message(msg)) = dst_events.next().await {
                    src.send_with_str(
                        message_text(&msg).expect_throw("expect a text message from dst"),
                    )
                    .expect_throw("failed to send to src");
                } else {
                    throw_str("expect a text message from dst");
                }
            }
            WebsocketEvent::Close(reason) => {
                dst.close_with_code_and_reason(reason.code(), reason.reason())
                    .expect_throw("failed to close dst websocket");
            }
        }
    }
}
