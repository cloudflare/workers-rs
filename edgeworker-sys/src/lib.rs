mod cf;
mod global;
mod headers;
mod request;
mod response;

pub mod prelude {
    pub use crate::cf::Cf;
    pub use crate::global::WorkerGlobalScope;
    pub use crate::headers::Headers;
    pub use crate::request::Request;
    pub use crate::response::Response;
}

pub use cf::Cf;
pub use global::WorkerGlobalScope;
pub use headers::Headers;
pub use request::Request;
pub use response::Response;
pub use web_sys::{RequestInit, ResponseInit};
