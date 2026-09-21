use proc_macro2::TokenStream;
use quote::quote;

/// An async wasm-bindgen export resolving to `Result<JsValue, JsValue>`.
/// The body is asserted unwind-safe as under `panic = "unwind"` wasm-bindgen
/// requires it of the exported future, and handler futures hold
/// `worker::Error`.
///
/// With the `tokio` feature each invocation is driven on its own Tokio event
/// loop.
pub fn async_export(opts: TokenStream, sig: TokenStream, body: TokenStream) -> TokenStream {
    let opts = if opts.is_empty() {
        opts
    } else {
        quote! { #opts, }
    };
    let tokio = cfg!(feature = "tokio").then(|| quote! { tokio = "isolated" });
    quote! {
        #[wasm_bindgen(#opts #tokio)]
        pub async fn #sig -> ::std::result::Result<::worker::wasm_bindgen::JsValue, ::worker::wasm_bindgen::JsValue> {
            ::std::panic::AssertUnwindSafe(async move { #body }).await
        }
    }
}

/// The same for a free function, placed in `mod_name` with `uses` in scope.
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
        mod #mod_name {
            use ::worker::wasm_bindgen::prelude::wasm_bindgen;
            #uses
            #export
        }
    }
}
