use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, ItemImpl, ItemStruct};

pub fn expand_macro(tokens: TokenStream) -> syn::Result<TokenStream> {
    if syn::parse2::<ItemImpl>(tokens.clone()).is_ok() {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "#[workflow] should only be applied to struct definitions, not impl blocks",
        ));
    }

    let target = syn::parse2::<ItemStruct>(tokens)?;
    let target_name = &target.ident;
    let marker_fn_name = format_ident!("__wf_{}", target_name);
    let marker_js_name = format!("__wf_{}", target_name);
    let target_name_str = target_name.to_string();

    Ok(quote! {
        #target

        impl ::worker::HasWorkflowAttribute for #target_name {}

        const _: () = {
            use ::worker::wasm_bindgen::prelude::*;
            #[allow(unused_imports)]
            use ::worker::WorkflowEntrypoint;

            #[allow(non_snake_case)]
            #[wasm_bindgen(js_name = #marker_js_name, wasm_bindgen=::worker::wasm_bindgen)]
            pub fn #marker_fn_name() -> ::worker::js_sys::JsString {
                ::worker::js_sys::JsString::from(#target_name_str)
            }

            #[wasm_bindgen(wasm_bindgen=::worker::wasm_bindgen)]
            #[::worker::consume]
            #target

            #[wasm_bindgen(wasm_bindgen=::worker::wasm_bindgen)]
            impl #target_name {
                #[wasm_bindgen(constructor, wasm_bindgen=::worker::wasm_bindgen)]
                pub fn new(
                    ctx: ::worker::worker_sys::Context,
                    env: ::worker::Env
                ) -> Self {
                    <Self as ::worker::WorkflowEntrypoint>::new(
                        ::worker::Context::new(ctx),
                        env
                    )
                }

                #[wasm_bindgen(js_name = run, wasm_bindgen=::worker::wasm_bindgen)]
                pub fn run(
                    &self,
                    event: ::worker::wasm_bindgen::JsValue,
                    step: ::worker::worker_sys::WorkflowStep
                ) -> ::worker::js_sys::Promise {
                    // SAFETY: The Cloudflare Workers runtime manages the Workflow instance
                    // lifecycle. The runtime guarantees that:
                    // 1. The instance is created before run() is called
                    // 2. The instance is not destroyed while any Promise returned by run() is pending
                    // 3. WASM execution is single-threaded, so no concurrent access is possible
                    // This is the same lifecycle model used by Durable Objects and WorkerEntrypoint.
                    let static_self: &'static Self = unsafe { &*(self as *const _) };

                    ::worker::wasm_bindgen_futures::future_to_promise(::std::panic::AssertUnwindSafe(async move {
                        let event = ::worker::WorkflowEvent::from_js(event)
                            .map_err(|e| ::worker::wasm_bindgen::JsValue::from_str(&e.to_string()))?;
                        let step = ::worker::WorkflowStep::from(step);
                        let result = <Self as ::worker::WorkflowEntrypoint>::run(static_self, event, step).await
                            .map_err(::worker::wasm_bindgen::JsValue::from)?;
                        ::worker::serialize_as_object(&result)
                            .map_err(|e| ::worker::wasm_bindgen::JsValue::from_str(&e.to_string()))
                    }))
                }
            }
        };
    })
}
