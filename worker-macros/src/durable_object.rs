use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Error, ItemStruct, ItemImpl};

enum DurableObjectType {
    Fetch,
    Alarm,
    WebSocket,
}
impl syn::parse::Parse for DurableObjectType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        match &*ident.to_string() {
            "fetch" => Ok(Self::Fetch),
            "alarm" => Ok(Self::Alarm),
            "websocket" => Ok(Self::WebSocket),
            _ => Err(Error::new(ident.span(), "must have either 'fetch', 'alarm' or 'websocket' attribute, e.g. #[durable_object(websocket)]"))
        }
    }
}

mod bindgen_methods {
    use proc_macro2::TokenStream;
    use quote::quote;

    pub fn core() -> TokenStream {
        quote! {
            #[wasm_bindgen(constructor)]
            pub fn new(
                state: ::worker::worker_sys::DurableObjectState,
                env:   ::worker::Env
            ) -> Self {
                <Self as ::worker::DurableObject>::new(
                    ::worker::durable::State::from(state),
                    env
                )
            }

            #[wasm_bindgen(js_name = fetch)]
            pub fn fetch(
                &self,
                req: ::worker::worker_sys::web_sys::Request
            ) -> ::worker::js_sys::Promise {
                // SAFETY:
                // Durable Object will never be destroyed while there is still
                // a running promise inside of it, therefore we can let a reference
                // to the durable object escape into a static-lifetime future.
                let static_self: &'static Self = unsafe { &*(self as *const _) };

                ::worker::wasm_bindgen_futures::future_to_promise(async move {
                    <Self as ::worker::DurableObject>::fetch(static_self, req.into()).await
                        .map(worker::worker_sys::web_sys::Response::from)
                        .map(::worker::wasm_bindgen::JsValue::from)
                        .map_err(::worker::wasm_bindgen::JsValue::from)
                })
            }
        }
    }

    pub fn alarm() -> TokenStream {
        quote! {
            #[wasm_bindgen(js_name = alarm)]
            pub fn alarm(&self) -> ::worker::js_sys::Promise {
                // SAFETY:
                // Durable Object will never be destroyed while there is still
                // a running promise inside of it, therefore we can let a reference
                // to the durable object escape into a static-lifetime future.
                let static_self: &'static Self = unsafe { &*(self as *const _) };

                ::worker::wasm_bindgen_futures::future_to_promise(async move {
                    <Self as ::worker::DurableObject>::alarm(static_self).await
                        .map(::worker::worker_sys::web_sys::Response::from)
                        .map(::worker::wasm_bindgen::JsValue::from)
                        .map_err(::worker::wasm_bindgen::JsValue::from)
                })
            }
        }
    }

    pub fn websocket() -> TokenStream {
        quote! {
            #[wasm_bindgen(js_name = webSocketMessage)]
            pub fn websocket_message(
                &self,
                ws: ::worker::worker_sys::web_sys::WebSocket,
                message: ::worker::wasm_bindgen::JsValue
            ) -> ::worker::js_sys::Promise {
                let message = match message.as_string() {
                    Some(message) => ::worker::WebSocketIncomingMessage::String(message),
                    None => ::worker::WebSocketIncomingMessage::Binary(
                        ::worker::js_sys::Uint8Array::new(&message).to_vec()
                    )
                };

                // SAFETY:
                // Durable Object will never be destroyed while there is still
                // a running promise inside of it, therefore we can let a reference
                // to the durable object escape into a static-lifetime future.
                let static_self: &'static Self = unsafe { &*(self as *const _) };

                ::worker::wasm_bindgen_futures::future_to_promise(async move {
                    <Self as ::worker::DurableObject>::websocket_message(static_self, ws.into(), message).await
                        .map(|_| ::worker::wasm_bindgen::JsValue::NULL)
                        .map_err(::worker::wasm_bindgen::JsValue::from)
                })
            }

            #[wasm_bindgen(js_name = webSocketClose)]
            pub fn websocket_close(
                &self,
                ws: ::worker::worker_sys::web_sys::WebSocket,
                code: usize,
                reason: String,
                was_clean: bool
            ) -> ::worker::js_sys::Promise {
                // SAFETY:
                // Durable Object will never be destroyed while there is still
                // a running promise inside of it, therefore we can let a reference
                // to the durable object escape into a static-lifetime future.
                let static_self: &'static Self = unsafe { &*(self as *const _) };

                ::worker::wasm_bindgen_futures::future_to_promise(async move {
                    <Self as ::worker::DurableObject>::websocket_close(static_self, ws.into(), code, reason, was_clean).await
                        .map(|_| ::worker::wasm_bindgen::JsValue::NULL)
                        .map_err(::worker::wasm_bindgen::JsValue::from)
                })
            }

            #[wasm_bindgen(js_name = webSocketError)]
            pub fn websocket_error(
                &self,
                ws: ::worker::worker_sys::web_sys::WebSocket,
                error: ::worker::wasm_bindgen::JsValue
            ) -> ::worker::js_sys::Promise {
                // SAFETY:
                // Durable Object will never be destroyed while there is still
                // a running promise inside of it, therefore we can let a reference
                // to the durable object escape into a static-lifetime future.
                let static_self: &'static Self = unsafe { &*(self as *const _) };

                ::worker::wasm_bindgen_futures::future_to_promise(async move {
                    <Self as ::worker::DurableObject>::websocket_error(static_self, ws.into(), error.into()).await
                        .map(|_| ::worker::wasm_bindgen::JsValue::NULL)
                        .map_err(::worker::wasm_bindgen::JsValue::from)
                })
            }
        }
    }
}

pub fn expand_macro(attr: TokenStream, tokens: TokenStream) -> syn::Result<TokenStream> {
    // accept an impl block for #[durable_object] backward compatibility
    if let Ok(_) = syn::parse2::<ItemImpl>(tokens.clone()) {
        return Ok(tokens)
    }

    let target = syn::parse2::<ItemStruct>(tokens)?;

    let durable_object_type = (!attr.is_empty())
        .then(|| syn::parse2::<DurableObjectType>(attr))
        .transpose()?;
    
    let bindgen_methods = match durable_object_type {
        // if not specified, bindgen all.
        // this is expected behavoir, and is also required for #[durable_object] to compile and work
        None => vec![
            bindgen_methods::core(),
            bindgen_methods::alarm(),
            bindgen_methods::websocket(),
        ],

        // if specified, bindgen only related methods.
        Some(DurableObjectType::Fetch) => vec![
            bindgen_methods::core(),
        ],
        Some(DurableObjectType::Alarm) => vec![
            bindgen_methods::core(),
            bindgen_methods::alarm(),
        ],
        Some(DurableObjectType::WebSocket) => vec![
            bindgen_methods::core(),
            bindgen_methods::websocket(),
        ],
    };

    let target_name = &target.ident;
    Ok(quote! {
        #target

        impl ::worker::has_DurableObject_attribute for #target_name {}

        const _: () = {
            use ::worker::wasm_bindgen;

            #[::worker::wasm_bindgen::prelude::wasm_bindgen]
            #[::worker::consume]
            #target

            #[::worker::wasm_bindgen::prelude::wasm_bindgen]
            impl #target_name {
                #(#bindgen_methods)*
            }
        };
    })
}
