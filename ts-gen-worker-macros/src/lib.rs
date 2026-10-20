//! Proc macros for `ts-gen-worker`. The bindings are generated; this crate holds the one
//! piece that cannot be, the handler exports.

mod durable_object;
mod event;
mod send;

use proc_macro::TokenStream;

/// Exports the function as a Worker handler.
///
/// * `fetch`: `async fn(Request, Env, ExecutionContext) -> Result<Response, E>`; the request
///   and response types may be anything `FromRequest`/`IntoResponse` cover, such as the
///   `http` crate's under that feature
/// * `scheduled`: `async fn(ScheduledController, Env, ExecutionContext)`
/// * `queue`: `async fn(MessageBatch<_>, Env, ExecutionContext) -> Result<(), E>`
/// * `email`: `async fn(ForwardableEmailMessage, Env, ExecutionContext) -> Result<(), E>`
/// * `connect`: `async fn(Socket, Env, ExecutionContext) -> Result<(), E>`; one export
///   serves TCP and UDP, so a handler for both dispatches on the socket's protocol
/// * `start`: `fn()`, run once at module initialization
///
/// Parameter types are taken verbatim from the signature, so the app's generated `Env`
/// (or any other type) works. The error type needs `Into<JsValue>`; `ts_gen_worker::Error`
/// and `JsValue` both qualify.
///
/// A failed `fetch` logs the error and responds `500 INTERNAL SERVER ERROR`;
/// `#[event(fetch, respond_with_errors)]` puts the error's `Display` text in the body
/// instead, which additionally requires `Display` on the error type.
/// Failed `queue` and `email` handlers reject, so the runtime retries the batch or
/// bounces the message; a failed `connect` handler rejects, closing only that socket.
#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    event::expand(attr.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Exports the struct as a Durable Object class. The struct must implement
/// `ts_gen_worker::DurableObject`; its `Env` associated type is the constructor's `env`
/// parameter type.
///
/// By default `fetch`, `alarm` and the `webSocket*` hibernation callbacks are exported;
/// `#[durable_object(fetch)]`, `(alarm)` or `(websocket)` export only `fetch` plus the
/// named group.
#[proc_macro_attribute]
pub fn durable_object(attr: TokenStream, item: TokenStream) -> TokenStream {
    durable_object::expand(attr.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Makes the future an `async fn` returns `Send` by running its body inside
/// `ts_gen_worker::send::SendFuture`, for frameworks whose handler bounds require it.
#[proc_macro_attribute]
pub fn send(_attr: TokenStream, item: TokenStream) -> TokenStream {
    send::expand(item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Removes the item it is attached to; lets `#[wasm_bindgen]` see a struct that is
/// defined elsewhere.
#[doc(hidden)]
#[proc_macro_attribute]
pub fn consume(_: TokenStream, _: TokenStream) -> TokenStream {
    TokenStream::new()
}
