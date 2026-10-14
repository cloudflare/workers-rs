use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Ident, ItemStruct, Result};

enum Methods {
    Fetch,
    Alarm,
}

pub fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let methods = if attr.is_empty() {
        Methods::Alarm
    } else {
        let ident: Ident = syn::parse2(attr)?;
        match ident.to_string().as_str() {
            "fetch" => Methods::Fetch,
            "alarm" => Methods::Alarm,
            _ => {
                return Err(Error::new(
                    ident.span(),
                    "expected `fetch` or `alarm`, e.g. #[durable_object(alarm)]",
                ))
            }
        }
    };
    let target: ItemStruct = syn::parse2(item)?;
    let name = &target.ident;

    let alarm = match methods {
        Methods::Fetch => quote! {},
        Methods::Alarm => quote! {
            #[wasm_bindgen(js_name = alarm)]
            pub async fn alarm(&self) -> ::core::result::Result<(), JsValue> {
                <Self as ::ts_gen_worker::DurableObject>::alarm(self).await.map_err(JsValue::from)
            }
        },
    };

    Ok(quote! {
        #target

        const _: () = {
            use ::ts_gen_worker::wasm_bindgen::prelude::*;

            #[wasm_bindgen(wasm_bindgen = ::ts_gen_worker::wasm_bindgen)]
            #[::ts_gen_worker::__macro::consume]
            #target

            #[wasm_bindgen(
                wasm_bindgen = ::ts_gen_worker::wasm_bindgen,
                wasm_bindgen_futures = ::ts_gen_worker::wasm_bindgen_futures,
                js_sys = ::ts_gen_worker::js_sys
            )]
            impl #name {
                #[wasm_bindgen(constructor)]
                pub fn new(
                    state: ::ts_gen_worker::DurableObjectState,
                    env: <#name as ::ts_gen_worker::DurableObject>::Env,
                ) -> Self {
                    <Self as ::ts_gen_worker::DurableObject>::new(state, env)
                }

                #[wasm_bindgen(js_name = fetch)]
                pub async fn fetch(
                    &self,
                    req: ::ts_gen_worker::Request,
                ) -> ::core::result::Result<::ts_gen_worker::Response, JsValue> {
                    <Self as ::ts_gen_worker::DurableObject>::fetch(self, req).await.map_err(JsValue::from)
                }

                #alarm
            }
        };
    })
}
