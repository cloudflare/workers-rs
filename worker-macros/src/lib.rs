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
