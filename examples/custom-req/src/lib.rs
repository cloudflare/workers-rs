use worker::*;

#[event(fetch)]
async fn main(_req: MyRequest, _env: Env, _ctx: Context) -> Result<MyResponse> {
    Ok(MyResponse::new("Hello, World!"))
}

struct MyRequest {}

impl FromRequest for MyRequest {
    fn from_raw(
        _req: crate::worker_sys::web_sys::Request,
    ) -> std::result::Result<Self, impl Into<Box<dyn std::error::Error>>> {
        // we don't care about the request, so we just return a new instance
        Ok::<Self, Error>(Self {})
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

impl IntoResponse for MyResponse {
    fn into_raw(
        self,
    ) -> std::result::Result<
        crate::worker_sys::web_sys::Response,
        impl Into<Box<dyn std::error::Error>>,
    > {
        crate::worker_sys::web_sys::Response::new_with_opt_str(Some(self.data))
            .map_err(|e| Error::JsError(format!("{e:?}")))
    }
}
