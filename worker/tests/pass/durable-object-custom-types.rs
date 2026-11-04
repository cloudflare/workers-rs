// A Durable Object may pick its own `fetch` request/response types, like the
// `#[event(fetch)]` handler; the `#[durable_object]` glue infers them from the impl.
use worker::{
    durable_object, worker_sys::web_sys, DurableObject, Env, Request, Response, Result, State,
};

#[durable_object]
pub struct DefaultTypes {
    _state: State,
}

impl DurableObject for DefaultTypes {
    fn new(state: State, _env: Env) -> Self {
        Self { _state: state }
    }

    async fn fetch(&self, _req: Request) -> Result<Response> {
        Response::ok("default")
    }
}

#[durable_object]
pub struct RawTypes {
    _state: State,
}

impl DurableObject<web_sys::Request, web_sys::Response> for RawTypes {
    fn new(state: State, _env: Env) -> Self {
        Self { _state: state }
    }

    async fn fetch(&self, req: web_sys::Request) -> Result<web_sys::Response> {
        Ok(web_sys::Response::new_with_opt_str(Some(&req.url()))?)
    }

    async fn alarm(&self) -> Result<Response> {
        Response::ok("alarm")
    }
}

fn main() {}
