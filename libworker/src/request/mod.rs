pub mod cf;
pub mod headers;
pub mod method;

use cf::Cf;
use edgeworker_sys::request::Request as FfiRequest;
use headers::Headers;
use method::Method;
use url::Url;

/// The Request interface represents an HTTP request, and is part of the Fetch API.
pub struct Request {
    inner: FfiRequest,
}

impl Request {
    pub fn method(&self) -> Method {
        self.inner.method().into()
    }

    pub fn url(&self) -> Url {
        // unwrap: this should be fine cause the url will always be valid
        // if the worker is deployed. like, the request was able to get here.
        Url::parse(&self.inner.url()).unwrap()
    }

    pub fn headers(&self) -> Headers {
        self.inner.headers().into()
    }
}
