use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Ident, ItemStruct, Result};

#[derive(Clone, Copy)]
enum Methods {
    All,
    Fetch,
    Alarm,
    WebSocket,
}

pub fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let methods = if attr.is_empty() {
        Methods::All
    } else {
        let ident: Ident = syn::parse2(attr)?;
        match ident.to_string().as_str() {
            "fetch" => Methods::Fetch,
            "alarm" => Methods::Alarm,
            "websocket" => Methods::WebSocket,
            _ => {
                return Err(Error::new(
                    ident.span(),
                    "expected `fetch`, `alarm` or `websocket`, e.g. #[durable_object(alarm)]",
                ))
            }
        }
    };
    let target: ItemStruct = syn::parse2(item)?;
    let name = &target.ident;

    let alarm = match methods {
        Methods::All | Methods::Alarm => quote! {
            #[wasm_bindgen(js_name = alarm)]
            pub async fn alarm(&self) -> ::core::result::Result<(), JsValue> {
                <Self as ::ts_gen_worker::DurableObject>::alarm(self).await.map_err(JsValue::from)
            }
        },
        Methods::Fetch | Methods::WebSocket => quote! {},
    };
    let websocket = match methods {
        Methods::All | Methods::WebSocket => quote! {
            #[wasm_bindgen(js_name = webSocketMessage)]
            pub async fn websocket_message(
                &self,
                ws: ::ts_gen_worker::WebSocket,
                message: ::ts_gen_worker::DataKind,
            ) -> ::core::result::Result<(), JsValue> {
                <Self as ::ts_gen_worker::DurableObject>::websocket_message(self, ws, message)
                    .await
                    .map_err(JsValue::from)
            }

            #[wasm_bindgen(js_name = webSocketClose)]
            pub async fn websocket_close(
                &self,
                ws: ::ts_gen_worker::WebSocket,
                code: usize,
                reason: String,
                was_clean: bool,
            ) -> ::core::result::Result<(), JsValue> {
                <Self as ::ts_gen_worker::DurableObject>::websocket_close(
                    self, ws, code, reason, was_clean,
                )
                .await
                .map_err(JsValue::from)
            }

            #[wasm_bindgen(js_name = webSocketError)]
            pub async fn websocket_error(
                &self,
                ws: ::ts_gen_worker::WebSocket,
                error: JsValue,
            ) -> ::core::result::Result<(), JsValue> {
                <Self as ::ts_gen_worker::DurableObject>::websocket_error(
                    self, ws, ::ts_gen_worker::Error::from(error),
                )
                .await
                .map_err(JsValue::from)
            }
        },
        Methods::Fetch | Methods::Alarm => quote! {},
    };

    Ok(quote! {
        #target

        impl ::ts_gen_worker::__macro::has_durable_object_attribute for #name {}

        const _: () = {
            use ::ts_gen_worker::wasm_bindgen::prelude::*;

            // After a trap the glue reinstantiates the module; the runtime keeps
            // calling the Durable Object instances it already constructed, so
            // they are rebuilt from their `(state, env)` on next use.
            #[wasm_bindgen(wasm_bindgen = ::ts_gen_worker::wasm_bindgen, reinit = "reconstruct")]
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

                #websocket
            }
        };
    })
}
