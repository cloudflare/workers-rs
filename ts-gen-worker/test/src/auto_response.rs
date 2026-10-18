use crate::Env;
use ts_gen_worker::*;

#[durable_object]
pub struct AutoResponseObject {
    state: DurableObjectState,
}

impl DurableObject for AutoResponseObject {
    type Env = Env;

    fn new(state: DurableObjectState, _env: Env) -> AutoResponseObject {
        Self { state }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        match req.path().as_str() {
            "/set" => {
                let pair = WebSocketRequestResponsePair::new("ping", "pong")?;
                self.state
                    .set_web_socket_auto_response_with_req_resp(&pair)?;
                Response::builder().ok("ok")
            }
            "/get" => {
                if let Some(pair) = self.state.get_web_socket_auto_response()? {
                    let request_str = pair.request();
                    let response_str = pair.response();
                    Response::builder().ok(format!("{request_str}:{response_str}"))
                } else {
                    Response::builder().ok("none")
                }
            }
            _ => Response::builder().error("Not Found", 404),
        }
    }
}

pub async fn handle_auto_response(
    _req: Request,
    env: Env,
    _data: crate::SomeSharedData,
) -> Result<Response> {
    let namespace = env.auto();
    let stub = namespace.get(&namespace.id_from_name("singleton")?)?;
    stub.fetch_with_str("https://fake-host/set").await?;
    stub.fetch_with_str("https://fake-host/get").await
}
