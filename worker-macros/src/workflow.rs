use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, ItemImpl, ItemStruct};

pub fn expand_macro(tokens: TokenStream) -> syn::Result<TokenStream> {
    let target = match syn::parse2::<ItemStruct>(tokens.clone()) {
        Ok(s) => s,
        Err(e) => {
            if syn::parse2::<ItemImpl>(tokens).is_ok() {
                return Err(Error::new(
                    proc_macro2::Span::call_site(),
                    "#[workflow] should only be applied to struct definitions, not impl blocks",
                ));
            }
            return Err(e);
        }
    };
    let target_name = &target.ident;
    let marker_fn_name = format_ident!("__wf_{}", target_name);
    let marker_js_name = format!("__wf_{target_name}");

    Ok(quote! {
        #target

        impl ::worker::HasWorkflowAttribute for #target_name {}

        const _: () = {
            use ::worker::wasm_bindgen::prelude::*;
            #[allow(unused_imports)]
            use ::worker::WorkflowEntrypoint;

            // Marker export read by `worker-build` to detect workflow classes
            // in the generated `index.js`. Only the function name matters.
            #[allow(non_snake_case)]
            #[wasm_bindgen(js_name = #marker_js_name, wasm_bindgen=::worker::wasm_bindgen)]
            pub fn #marker_fn_name() {}

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
                    // SAFETY: widen `&Self` to `&'static Self`. The Workers runtime keeps
                    // `self` alive until the returned Promise settles, which is the
                    // lifecycle contract for Workflow instances.
                    let static_self: &'static Self = unsafe { &*(self as *const _) };

                    ::worker::wasm_bindgen_futures::future_to_promise(::std::panic::AssertUnwindSafe(async move {
                        let event: ::worker::WorkflowEvent<<Self as ::worker::WorkflowEntrypoint>::Input> =
                            ::worker::WorkflowEvent::from_js(event)
                                .map_err(|e| ::worker::wasm_bindgen::JsValue::from_str(&e.to_string()))?;
                        let step = ::worker::WorkflowStep::from(step);
                        let output = <Self as ::worker::WorkflowEntrypoint>::run(static_self, event, step).await
                            .map_err(::worker::wasm_bindgen::JsValue::from)?;
                        ::worker::serialize_as_object(&output)
                            .map_err(|e| ::worker::wasm_bindgen::JsValue::from_str(&e.to_string()))
                    }))
                }
            }
        };
    })
}
