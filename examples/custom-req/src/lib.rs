use worker::*;

#[event(fetch)]
async fn main(_req: MyRequest, _env: Env, _ctx: Context) -> Result<MyResponse> {
    Ok(MyResponse::new("Hello, World!"))
}

struct MyRequest {}

impl WorkerRequest for MyRequest {
    fn from_web_sys(_req: crate::worker_sys::web_sys::Request) -> Self {
        // we don't care about the request, so we just return a new instance
        Self {}
    }
}

struct MyResponse {
    data: &'static str,
}

impl MyResponse {
    fn new(data: &'static str) -> Self {
        Self { data }
    }
}

impl WorkerResponse for MyResponse {
    fn into_web_sys(self) -> crate::worker_sys::web_sys::Response {
        crate::worker_sys::web_sys::Response::new_with_opt_str(Some(self.data)).unwrap()
    }
}
