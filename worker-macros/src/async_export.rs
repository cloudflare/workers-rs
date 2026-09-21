use proc_macro2::TokenStream;
use quote::quote;

/// An async wasm-bindgen export resolving to `Result<JsValue, JsValue>`.
///
/// Under `worker-build --emscripten --tokio` the `worker_tokio = "event_loop"`
/// cfg adds `tokio = "isolated"`, driving each invocation on its own Tokio
/// event loop.
pub fn async_export(opts: TokenStream, sig: TokenStream, body: TokenStream) -> TokenStream {
    let opts = if opts.is_empty() {
        opts
    } else {
        quote! { #opts, }
    };
    quote! {
        #[cfg_attr(not(worker_tokio = "event_loop"), wasm_bindgen(#opts))]
        #[cfg_attr(worker_tokio = "event_loop", wasm_bindgen(#opts tokio = "isolated"))]
        pub async fn #sig -> ::std::result::Result<::worker::wasm_bindgen::JsValue, ::worker::wasm_bindgen::JsValue> {
            #body
        }
    }
}

/// The same for a free function, placed in `mod_name` with `uses` in scope.
/// `worker_tokio` is set by worker-build and unknown to the crate's check-cfg
/// otherwise.
pub fn async_export_mod(
    mod_name: &proc_macro2::Ident,
    uses: TokenStream,
    sig: TokenStream,
    body: TokenStream,
) -> TokenStream {
    let export = async_export(
        quote! {
            wasm_bindgen=::worker::wasm_bindgen,
            wasm_bindgen_futures=::worker::wasm_bindgen_futures,
            js_sys=::worker::js_sys
        },
        sig,
        body,
    );
    quote! {
        #[allow(unexpected_cfgs)]
        mod #mod_name {
            use ::worker::wasm_bindgen::prelude::wasm_bindgen;
            #uses
            #export
        }
    }
}
