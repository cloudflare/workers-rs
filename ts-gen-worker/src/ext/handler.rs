use crate::{Request, Response, Result};

/// The request type a `#[event(fetch)]` handler accepts: the export always receives the
/// platform `Request` and converts it through this trait.
pub trait FromRequest: Sized {
    fn from_request(req: Request) -> Result<Self>;
}

impl FromRequest for Request {
    fn from_request(req: Request) -> Result<Self> {
        Ok(req)
    }
}

/// The response type a `#[event(fetch)]` handler returns.
pub trait IntoResponse {
    fn into_response(self) -> Result<Response>;
}

impl IntoResponse for Response {
    fn into_response(self) -> Result<Response> {
        Ok(self)
    }
}
