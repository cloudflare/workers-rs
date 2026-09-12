use proc_macro2::TokenStream;
use quote::quote;

/// A wasm-bindgen export running `body` (statements ending in an `async move`
/// block resolving to `Result<JsValue, JsValue>`) to a JS Promise.
///
/// Normally a synchronous export returning `future_to_promise(body)`. On
/// emscripten the export is a JSPI promising function that suspends on the
/// future's promise instead, so the activation runs on its own fiber and
/// blocking waits inside it (Tokio parks, `epoll_wait`) suspend the stack.
pub fn async_export(opts: TokenStream, sig: TokenStream, body: TokenStream) -> TokenStream {
    let opts = if opts.is_empty() {
        opts
    } else {
        quote! { #opts, }
    };
    quote! {
        #[cfg(not(target_os = "emscripten"))]
        #[wasm_bindgen(#opts wasm_bindgen=::worker::wasm_bindgen)]
        pub fn #sig -> ::worker::js_sys::Promise {
            ::worker::js_sys::futures::future_to_promise(::std::panic::AssertUnwindSafe({ #body }))
        }

        #[cfg(target_os = "emscripten")]
        #[allow(deprecated)]
        #[wasm_bindgen(jspi, #opts wasm_bindgen=::worker::wasm_bindgen)]
        pub fn #sig -> ::std::result::Result<::worker::wasm_bindgen::JsValue, ::worker::wasm_bindgen::JsValue> {
            ::worker::js_sys::futures::jspi_block_on_promise(
                &::worker::js_sys::futures::future_to_promise(::std::panic::AssertUnwindSafe({ #body }))
            )
        }
    }
}

/// The same for a free function, placed in `mod_name` with `uses` in scope.
/// The `#[wasm_bindgen]` attribute is left for rustc to expand so that only
/// the variant surviving `cfg` produces an export descriptor.
pub fn async_export_mod(
    mod_name: &proc_macro2::Ident,
    uses: TokenStream,
    sig: TokenStream,
    body: TokenStream,
) -> TokenStream {
    let export = async_export(TokenStream::new(), sig, body);
    quote! {
        mod #mod_name {
            use ::worker::wasm_bindgen::prelude::wasm_bindgen;
            #uses
            #export
        }
    }
}
