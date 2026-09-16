use proc_macro2::TokenStream;
use quote::quote;

/// A wasm-bindgen export running `body` (statements ending in an `async move`
/// block resolving to `Result<JsValue, JsValue>`) to a JS Promise.
///
/// Normally a synchronous export returning `future_to_promise(body)`. Under
/// `worker-build --emscripten --tokio` the `worker_tokio = "event_loop"` cfg
/// schedules the future on a Tokio event loop whose wait is the host event
/// loop instead.
pub fn async_export(opts: TokenStream, sig: TokenStream, body: TokenStream) -> TokenStream {
    let opts = if opts.is_empty() {
        opts
    } else {
        quote! { #opts, }
    };
    quote! {
        #[cfg(not(worker_tokio = "event_loop"))]
        #[wasm_bindgen(#opts wasm_bindgen=::worker::wasm_bindgen)]
        pub fn #sig -> ::worker::js_sys::Promise {
            ::worker::js_sys::futures::future_to_promise(::std::panic::AssertUnwindSafe({ #body }))
        }

        #[cfg(worker_tokio = "event_loop")]
        #[wasm_bindgen(#opts wasm_bindgen=::worker::wasm_bindgen)]
        pub fn #sig -> ::worker::js_sys::Promise {
            ::worker::__tokio_promise(::std::panic::AssertUnwindSafe({ #body }))
        }
    }
}

/// The same for a free function, placed in `mod_name` with `uses` in scope.
/// The `#[wasm_bindgen]` attribute is left for rustc to expand so that only
/// the variant surviving `cfg` produces an export descriptor. `worker_tokio`
/// is set by worker-build and unknown to the crate's check-cfg otherwise.
pub fn async_export_mod(
    mod_name: &proc_macro2::Ident,
    uses: TokenStream,
    sig: TokenStream,
    body: TokenStream,
) -> TokenStream {
    let export = async_export(TokenStream::new(), sig, body);
    quote! {
        #[allow(unexpected_cfgs)]
        mod #mod_name {
            use ::worker::wasm_bindgen::prelude::wasm_bindgen;
            #uses
            #export
        }
    }
}
