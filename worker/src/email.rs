use wasm_bindgen::JsCast;

pub use crate::bindings::email::*;
use crate::{EnvBinding, Result};

impl EnvBinding for SendEmail {
    const TYPE_NAME: &'static str = "SendEmail";

    // `SendEmail` is a TypeScript interface, not a class — the runtime
    // doesn't expose a `SendEmail` global for the default
    // `constructor.name` check to match against. The TS types are
    // authoritative: if `env.EMAIL` is bound to a SendEmail per
    // `wrangler.toml`, the runtime hands us the right shape, so we
    // skip the check and `unchecked_into`.
    fn get(val: wasm_bindgen::JsValue) -> Result<Self> {
        Ok(val.unchecked_into())
    }
}

#[cfg(test)]
mod send_check {
    // `SendEmail` is `Send` automatically — wasm-bindgen makes `JsValue`
    // `Send + Sync` and every extern `pub type` in `email.rs` carries that
    // through. This compile-time check guards against an upstream
    // regression.
    use super::SendEmail;
    fn _assert_send<T: Send>() {}
    #[allow(dead_code)]
    fn _check() {
        _assert_send::<SendEmail>();
    }
}
