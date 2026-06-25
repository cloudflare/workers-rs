use worker::{
    durable_object, js_sys::Uint8Array, DurableObject, Env, Request, Response, Result, State,
    WebSocketRequestResponsePair,
};

#[durable_object]
pub struct AutoResponseObject {
    state: State,
}

impl DurableObject for AutoResponseObject {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        match req.path().as_str() {
            "/set" => {
                let pair = WebSocketRequestResponsePair::new("ping", "pong")?;
                self.state.set_websocket_auto_response(&pair);
                Response::ok("ok")
            }
            "/set-binary" => {
                let request_data: &[u8] = &[0x01, 0x02, 0x03];
                let response_data: &[u8] = &[0x04, 0x05, 0x06];
                let pair = WebSocketRequestResponsePair::new_bytes(
                    &Uint8Array::from(request_data),
                    &Uint8Array::from(response_data),
                )?;
                self.state.set_websocket_auto_response(&pair);
                Response::ok("ok")
            }
            "/get" => {
                if let Some(pair) = self.state.get_websocket_auto_response() {
                    let request_str = pair.request();
                    let response_str = pair.response();
                    Response::ok(format!("{request_str}:{response_str}"))
                } else {
                    Response::ok("none")
                }
            }
            "/get-binary" => {
                if let Some(pair) = self.state.get_websocket_auto_response() {
                    let req_bytes = pair.request_bytes().to_vec();
                    let res_bytes = pair.response_bytes().to_vec();
                    Response::ok(format!("{req_bytes:?}:{res_bytes:?}"))
                } else {
                    Response::ok("none")
                }
            }
            _ => Response::error("Not Found", 404),
        }
    }
}

#[worker::send]
pub async fn handle_auto_response(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("AUTO")?;
    let stub = namespace.id_from_name("singleton")?.get_stub()?;
    stub.fetch_with_str("https://fake-host/set").await?;
    stub.fetch_with_str("https://fake-host/get").await
}

#[worker::send]
pub async fn handle_auto_response_binary(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("AUTO")?;
    let stub = namespace.id_from_name("singleton-binary")?.get_stub()?;
    stub.fetch_with_str("https://fake-host/set-binary").await?;
    stub.fetch_with_str("https://fake-host/get-binary").await
}
