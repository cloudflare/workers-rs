//! Cloudflare Workers bindings generated from `types/worker.d.ts` by ts-gen, plus the
//! Rust-side ergonomics (`#[event]`, `Result`, serde, futures) layered on them as
//! `impl` blocks.
//!
//! `src/bindings.rs` is generated; regenerate it with `chomp build:ts-gen-worker:types`
//! after editing the declarations.

#[allow(clippy::all, non_upper_case_globals)]
mod bindings;
mod error;
mod ext;

pub use bindings::*;
pub use error::{Error, Result};
pub use ext::*;
pub use ts_gen_worker_macros::event;

pub use js_sys;
pub use wasm_bindgen;
pub use wasm_bindgen_futures;

#[doc(hidden)]
pub mod __macro {
    use wasm_bindgen::JsValue;

    use crate::{Error, Response};

    /// Logs the handler error and builds the 500 response; `respond_with_errors`
    /// puts the error message in the body.
    pub fn error_response(err: JsValue, respond_with_errors: bool) -> Response {
        crate::console::error(std::slice::from_ref(&err));
        let body = if respond_with_errors {
            Error::from(err).to_string()
        } else {
            "INTERNAL SERVER ERROR".to_owned()
        };
        // Constructing a text response with status 500 cannot throw.
        Response::builder().error(body, 500).unwrap()
    }
}
