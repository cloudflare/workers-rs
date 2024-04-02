mod durable_object;
mod event;
mod send;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn durable_object(_attr: TokenStream, item: TokenStream) -> TokenStream {
    durable_object::expand_macro(item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(feature = "http")]
#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    event::expand_macro(attr, item, true)
}

/// The `event` macro is used to denote a [Worker handler](https://developers.cloudflare.com/workers/runtime-apis/handlers/), essentially binding from
/// the JS runtime to a Rust function.
///
/// As of right now, the following attributes are supported:
/// * `fetch`: [Fetch Handler](https://developers.cloudflare.com/workers/runtime-apis/handlers/fetch/)
/// * `scheduled`: [Scheduled Handler](https://developers.cloudflare.com/workers/runtime-apis/handlers/scheduled/)
/// * `queue`: [Queue Handler](https://developers.cloudflare.com/queues/reference/javascript-apis/#consumer)
///   * This attribute is only available when the `queue` feature is enabled.
/// * `start`: merely creates a [wasm-bindgen start function](https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-rust-exports/start.html)
/// * `respond_with_errors`: if this attribute is present, the function will return a `Response` object with a 500 status code and the status text of the error message, if an error occurs
///
/// The macro is expanded into a different function signature, depending on the attributes used
///
/// # Fetch
///
/// At a high-level, the `fetch` handler is used to handle incoming HTTP requests. The function signature for a `fetch` handler is conceptually something like:
///  
/// ```rust
/// async fn fetch(req: impl From<web_sys::Request>, env: Env, ctx: Context) -> Result<impl Into<web_sys::Response>, worker::Error>
/// ```
///
/// In other words, it takes a some "request" object that can be derived *from* a `web_sys::Request` (into whatever concrete Request type you like), and returns some "response" object that can be converted *into* a `web_sys::Response` (from whatever concrete Response type you like).
/// It also receives the worker `Env` and `Context` objects.
///
/// In practice, the "request" and "response" objects are usually one of these concrete types, supported out of the box:
///
/// ### worker::{Request, Response}
///
/// ```rust
/// #[event(fetch, respond_with_errors)]
/// async fn main(req: worker::Request, env: Env, ctx: Context) -> Result<worker::Response> {
///   worker::Response::ok("Hello World (worker type)")
/// }
/// ```
///
/// ### web_sys::{Request, Response}
///
/// ```rust
/// #[event(fetch, respond_with_errors)]
/// async fn main(req: web_sys::Request, env: Env, ctx: Context) -> Result<web_sys::Response> {
///   Ok(web_sys::Response::new_with_opt_str(Some("Hello World (native type)".into())).unwrap())
/// }
/// ```
///
/// ### axum (with `http` feature)
///
/// ```rust
///  #[event(fetch)]
/// async fn fetch(req: HttpRequest, env: Env, ctx: Context) -> Result<http::Response<axum::body::Body>> {
///   Ok(router().call(req).await?)
/// }
/// ```
#[cfg(not(feature = "http"))]
#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    event::expand_macro(attr, item, false)
}

#[proc_macro_attribute]
/// Convert an async function which is `!Send` to be `Send`.
///
/// This is useful for implementing async handlers in frameworks which
/// expect the handler to be `Send`, such as `axum`.
///
/// ```rust
/// #[worker::send]
/// async fn foo() {
///     // JsFuture is !Send
///     let fut = JsFuture::from(promise);
///     fut.await
/// }
/// ```
pub fn send(attr: TokenStream, stream: TokenStream) -> TokenStream {
    send::expand_macro(attr, stream)
}
