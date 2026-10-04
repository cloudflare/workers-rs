use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Error, ItemImpl, ItemStruct};

enum DurableObjectType {
    Fetch,
    Alarm,
    WebSocket,
    Connect,
}
impl syn::parse::Parse for DurableObjectType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        match &*ident.to_string() {
            "fetch" => Ok(Self::Fetch),
            "alarm" => Ok(Self::Alarm),
            "websocket" => Ok(Self::WebSocket),
            "connect" => Ok(Self::Connect),
            _ => Err(Error::new(ident.span(), "must have either 'fetch', 'alarm', 'websocket' or 'connect' attribute, e.g. #[durable_object(websocket)]"))
        }
    }
}

mod bindgen_methods {
    use crate::async_export::{async_export, Tokio};
    use proc_macro2::TokenStream;
    use quote::quote;

    pub fn core() -> TokenStream {
        let fetch = async_export(
            Tokio::Ambient,
            quote! { js_name = fetch },
            quote! { fetch(&self, req: ::worker::worker_sys::web_sys::Request) },
            quote! {
                <Self as ::worker::DurableObject>::fetch(self, req.into()).await
                    .map(::worker::worker_sys::web_sys::Response::from)
                    .map(::worker::wasm_bindgen::JsValue::from)
                    .map_err(::worker::wasm_bindgen::JsValue::from)
            },
        );
        quote! {
            #[wasm_bindgen(constructor, wasm_bindgen=::worker::wasm_bindgen)]
            pub fn new(
                state: ::worker::worker_sys::DurableObjectState,
                env:   ::worker::Env
            ) -> Self {
                <Self as ::worker::DurableObject>::new(
                    ::worker::durable::State::from(state),
                    env
                )
            }

            #fetch
        }
    }

    pub fn alarm() -> TokenStream {
        async_export(
            Tokio::Ambient,
            quote! { js_name = alarm },
            quote! { alarm(&self) },
            quote! {
                <Self as ::worker::DurableObject>::alarm(self).await
                    .map(::worker::worker_sys::web_sys::Response::from)
                    .map(::worker::wasm_bindgen::JsValue::from)
                    .map_err(::worker::wasm_bindgen::JsValue::from)
            },
        )
    }

    pub fn connect() -> TokenStream {
        async_export(
            Tokio::Ambient,
            quote! { js_name = connect },
            quote! { connect(&self, socket: ::worker::worker_sys::Socket) },
            quote! {
                <Self as ::worker::DurableObject>::connect(self, ::worker::Socket::from(socket)).await
                    .map(|_| ::worker::wasm_bindgen::JsValue::UNDEFINED)
                    .map_err(::worker::wasm_bindgen::JsValue::from)
            },
        )
    }

    pub fn websocket() -> TokenStream {
        let message = async_export(
            Tokio::Ambient,
            quote! { js_name = webSocketMessage },
            quote! {
                websocket_message(
                    &self,
                    ws: ::worker::worker_sys::web_sys::WebSocket,
                    message: ::worker::wasm_bindgen::JsValue
                )
            },
            quote! {
                let message = match message.as_string() {
                    Some(message) => ::worker::WebSocketIncomingMessage::String(message),
                    None => ::worker::WebSocketIncomingMessage::Binary(
                        ::worker::js_sys::Uint8Array::new(&message).to_vec()
                    )
                };
                <Self as ::worker::DurableObject>::websocket_message(self, ws.into(), message).await
                    .map(|_| ::worker::wasm_bindgen::JsValue::NULL)
                    .map_err(::worker::wasm_bindgen::JsValue::from)
            },
        );
        let close = async_export(
            Tokio::Ambient,
            quote! { js_name = webSocketClose },
            quote! {
                websocket_close(
                    &self,
                    ws: ::worker::worker_sys::web_sys::WebSocket,
                    code: usize,
                    reason: String,
                    was_clean: bool
                )
            },
            quote! {
                <Self as ::worker::DurableObject>::websocket_close(self, ws.into(), code, reason, was_clean).await
                    .map(|_| ::worker::wasm_bindgen::JsValue::NULL)
                    .map_err(::worker::wasm_bindgen::JsValue::from)
            },
        );
        let error = async_export(
            Tokio::Ambient,
            quote! { js_name = webSocketError },
            quote! {
                websocket_error(
                    &self,
                    ws: ::worker::worker_sys::web_sys::WebSocket,
                    error: ::worker::wasm_bindgen::JsValue
                )
            },
            quote! {
                <Self as ::worker::DurableObject>::websocket_error(self, ws.into(), error.into()).await
                    .map(|_| ::worker::wasm_bindgen::JsValue::NULL)
                    .map_err(::worker::wasm_bindgen::JsValue::from)
            },
        );
        quote! {
            #message
            #close
            #error
        }
    }
}

pub fn expand_macro(attr: TokenStream, tokens: TokenStream) -> syn::Result<TokenStream> {
    // Try to give a nice error for previous impl usage
    if syn::parse2::<ItemImpl>(tokens.clone()).is_ok() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "The #[durable_object] macro is no longer required for `impl` blocks, and can be removed"
        ));
    }

    let target = syn::parse2::<ItemStruct>(tokens)?;

    let durable_object_type = (!attr.is_empty())
        .then(|| syn::parse2::<DurableObjectType>(attr))
        .transpose()?;

    let bindgen_methods = match durable_object_type {
        // if not specified, bindgen all.
        // this is expected behavior, and is also required for #[durable_object] to compile and work
        None => vec![
            bindgen_methods::core(),
            bindgen_methods::alarm(),
            bindgen_methods::websocket(),
            bindgen_methods::connect(),
        ],

        // if specified, bindgen only related methods.
        Some(DurableObjectType::Fetch) => vec![bindgen_methods::core()],
        Some(DurableObjectType::Alarm) => vec![bindgen_methods::core(), bindgen_methods::alarm()],
        Some(DurableObjectType::WebSocket) => {
            vec![bindgen_methods::core(), bindgen_methods::websocket()]
        }
        Some(DurableObjectType::Connect) => {
            vec![bindgen_methods::core(), bindgen_methods::connect()]
        }
    };

    let target_name = &target.ident;
    Ok(quote! {
        #target

        impl ::worker::has_durable_object_attribute for #target_name {}

        const _: () = {
            use ::worker::wasm_bindgen::prelude::*;
            #[allow(unused_imports)]
            use ::worker::DurableObject;

            #[wasm_bindgen(wasm_bindgen=::worker::wasm_bindgen)]
            #[::worker::consume]
            #target

            #[wasm_bindgen(
                wasm_bindgen=::worker::wasm_bindgen,
                wasm_bindgen_futures=::worker::wasm_bindgen_futures,
                js_sys=::worker::js_sys
            )]
            impl #target_name {
                #(#bindgen_methods)*
            }
        };
    })
}
