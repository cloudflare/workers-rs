use worker::{
    durable_object, DurableObject, Env, Request, Response, Result, State,
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
                // Configure ping -> pong auto-response for all websockets bound to this DO.
                let pair = WebSocketRequestResponsePair::new("ping", "pong")?;
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
            _ => Response::error("Not Found", 404),
        }
    }
}

// Route handler to exercise the Durable Object from tests.
#[worker::send]
pub async fn handle_auto_response(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("AUTO")?;
    let stub = namespace.id_from_name("singleton")?.get_stub()?;
    // Ensure auto-response is configured
    stub.fetch_with_str("https://fake-host/set").await?;
    // Retrieve and return it for assertion
    stub.fetch_with_str("https://fake-host/get").await
}
