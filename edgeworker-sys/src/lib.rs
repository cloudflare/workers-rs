mod cf;
mod headers;
mod global;
mod request;
mod response;

pub mod prelude {
    pub use crate::cf::Cf;
    pub use crate::headers::Headers;
    pub use crate::global::WorkerGlobalScope;
    pub use crate::request::Request;
    pub use crate::response::Response;
}

pub use cf::Cf;
pub use headers::Headers;
pub use global::WorkerGlobalScope;
pub use request::{Request};
pub use web_sys::{RequestInit, ResponseInit};
pub use response::Response;
