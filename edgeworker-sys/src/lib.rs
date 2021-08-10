mod cf;
mod form_data;
pub mod global;
mod headers;
mod request;
mod response;

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (unsafe { $crate::global::log(&format_args!($($t)*).to_string()) })
}

pub mod prelude {
    pub use crate::cf::Cf;
    pub use crate::console_log;
    pub use crate::form_data::FormData;
    pub use crate::global::WorkerGlobalScope;
    pub use crate::headers::Headers;
    pub use crate::request::Request;
    pub use crate::response::Response;
}

pub use cf::Cf;
pub use form_data::FormData;
pub use global::WorkerGlobalScope;
pub use headers::Headers;
pub use request::Request;
pub use response::Response;
pub use web_sys::{RequestInit, ResponseInit};
