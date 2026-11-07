use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, ItemImpl, ItemStruct};

use crate::async_export::{async_export, Tokio};

pub fn expand_macro(tokens: TokenStream) -> syn::Result<TokenStream> {
    if syn::parse2::<ItemImpl>(tokens.clone()).is_ok() {
        return Err(Error::new(
            Span::call_site(),
            "#[workflow] should only be applied to struct definitions, not impl blocks",
        ));
    }
    let target = syn::parse2::<ItemStruct>(tokens)?;
    let target_name = &target.ident;
    // `worker-build` recognizes workflow classes by this prefix and exports
    // each one under the struct's name as a `WorkflowEntrypoint` subclass.
    let js_class = format!("__wf_{target_name}");
    let run = async_export(
        Tokio::Isolated,
        quote! { js_name = run },
        quote! {
            run(
                &self,
                event: ::worker::wasm_bindgen::JsValue,
                step: ::worker::worker_sys::WorkflowStep
            )
        },
        quote! { ::worker::run_workflow(self, event, step).await },
    );

    Ok(quote! {
        #target

        impl ::worker::HasWorkflowAttribute for #target_name {}

        const _: () = {
            use ::worker::wasm_bindgen::prelude::*;

            #[wasm_bindgen(js_name = #js_class, wasm_bindgen=::worker::wasm_bindgen)]
            #[::worker::consume]
            #target

            #[wasm_bindgen(
                js_class = #js_class,
                wasm_bindgen=::worker::wasm_bindgen,
                wasm_bindgen_futures=::worker::wasm_bindgen_futures,
                js_sys=::worker::js_sys
            )]
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

                #run
            }
        };
    })
}
