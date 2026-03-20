use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Error, ItemImpl, ItemStruct};

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

/// Generate flat wasm_bindgen function exports for a Durable Object.
///
/// Each DO instance is stored in a `thread_local! HashMap<u32, T>` keyed by an
/// instance ID assigned on the JS side.  Every exported function takes the
/// instance key as its first `u32` argument so the correct instance is looked up.
mod bindgen_fns {
    use proc_macro2::{Ident, TokenStream};
    use quote::quote;

    /// Wraps `body` in the boilerplate that borrows the instance from the map.
    /// Inside `body`, `static_ref: &'static T` is available.
    fn with_instance(class_name: &Ident, body: TokenStream) -> TokenStream {
        quote! {
            __INSTANCES.with(|map| {
                let borrow = map.borrow();
                let instance = borrow.get(&key)
                    .expect("Durable Object not initialized — call init first");
                // SAFETY: The instance reference is extended to `'static` lifetime for use
                // inside the `async move` future handed to `future_to_promise`. This is
                // sound under the following invariants:
                //   1. DO instances are stored in a `thread_local! HashMap` and are never
                //      removed while a future is alive — there is no drop export wired up
                //      from the JS side.  If a cleanup mechanism is added in the future
                //      this must be revisited (e.g. by storing `Arc<T>` instead of `T`).
                //   2. WASM is single-threaded; no concurrent mutation of the map is possible.
                //   3. The Workers runtime serialises requests to a single DO instance so
                //      only one `async` future runs at a time per instance.
                let static_ref: &'static #class_name = unsafe { &*(instance as *const _) };
                #body
            })
        }
    }

    pub fn init(class_name: &Ident, js_name: &str) -> TokenStream {
        quote! {
            #[wasm_bindgen(js_name = #js_name, wasm_bindgen=::worker::wasm_bindgen)]
            pub fn __init(
                key: u32,
                state: ::worker::worker_sys::DurableObjectState,
                env:   ::worker::Env
            ) {
                let instance = <#class_name as ::worker::DurableObject>::new(
                    ::worker::durable::State::from(state),
                    env,
                );
                __INSTANCES.with(|map| map.borrow_mut().insert(key, instance));
            }
        }
    }

    pub fn fetch(class_name: &Ident, js_name: &str) -> TokenStream {
        let body = with_instance(
            class_name,
            quote! {
                ::worker::wasm_bindgen_futures::future_to_promise(
                    ::std::panic::AssertUnwindSafe(async move {
                        <#class_name as ::worker::DurableObject>::fetch(static_ref, req.into()).await
                            .map(::worker::worker_sys::web_sys::Response::from)
                            .map(::worker::wasm_bindgen::JsValue::from)
                            .map_err(::worker::wasm_bindgen::JsValue::from)
                    })
                )
            },
        );
        quote! {
            #[wasm_bindgen(js_name = #js_name, wasm_bindgen=::worker::wasm_bindgen)]
            pub fn __fetch(key: u32, req: ::worker::worker_sys::web_sys::Request) -> ::worker::js_sys::Promise {
                #body
            }
        }
    }

    pub fn alarm(class_name: &Ident, js_name: &str) -> TokenStream {
        let body = with_instance(
            class_name,
            quote! {
                ::worker::wasm_bindgen_futures::future_to_promise(
                    ::std::panic::AssertUnwindSafe(async move {
                        <#class_name as ::worker::DurableObject>::alarm(static_ref).await
                            .map(::worker::worker_sys::web_sys::Response::from)
                            .map(::worker::wasm_bindgen::JsValue::from)
                            .map_err(::worker::wasm_bindgen::JsValue::from)
                    })
                )
            },
        );
        quote! {
            #[wasm_bindgen(js_name = #js_name, wasm_bindgen=::worker::wasm_bindgen)]
            pub fn __alarm(key: u32) -> ::worker::js_sys::Promise {
                #body
            }
        }
    }

    pub fn websocket(
        class_name: &Ident,
        msg_name: &str,
        close_name: &str,
        error_name: &str,
    ) -> TokenStream {
        let msg_body = with_instance(
            class_name,
            quote! {
                ::worker::wasm_bindgen_futures::future_to_promise(
                    ::std::panic::AssertUnwindSafe(async move {
                        <#class_name as ::worker::DurableObject>::websocket_message(static_ref, ws.into(), message).await
                            .map(|_| ::worker::wasm_bindgen::JsValue::NULL)
                            .map_err(::worker::wasm_bindgen::JsValue::from)
                    })
                )
            },
        );
        let close_body = with_instance(
            class_name,
            quote! {
                ::worker::wasm_bindgen_futures::future_to_promise(
                    ::std::panic::AssertUnwindSafe(async move {
                        <#class_name as ::worker::DurableObject>::websocket_close(static_ref, ws.into(), code, reason, was_clean).await
                            .map(|_| ::worker::wasm_bindgen::JsValue::NULL)
                            .map_err(::worker::wasm_bindgen::JsValue::from)
                    })
                )
            },
        );
        let error_body = with_instance(
            class_name,
            quote! {
                ::worker::wasm_bindgen_futures::future_to_promise(
                    ::std::panic::AssertUnwindSafe(async move {
                        <#class_name as ::worker::DurableObject>::websocket_error(static_ref, ws.into(), error.into()).await
                            .map(|_| ::worker::wasm_bindgen::JsValue::NULL)
                            .map_err(::worker::wasm_bindgen::JsValue::from)
                    })
                )
            },
        );
        quote! {
            #[wasm_bindgen(js_name = #msg_name, wasm_bindgen=::worker::wasm_bindgen)]
            pub fn __websocket_message(
                key: u32,
                ws: ::worker::worker_sys::web_sys::WebSocket,
                message: ::worker::wasm_bindgen::JsValue
            ) -> ::worker::js_sys::Promise {
                let message = match message.as_string() {
                    Some(message) => ::worker::WebSocketIncomingMessage::String(message),
                    None => ::worker::WebSocketIncomingMessage::Binary(
                        ::worker::js_sys::Uint8Array::new(&message).to_vec()
                    )
                };
                #msg_body
            }

            #[wasm_bindgen(js_name = #close_name, wasm_bindgen=::worker::wasm_bindgen)]
            pub fn __websocket_close(
                key: u32,
                ws: ::worker::worker_sys::web_sys::WebSocket,
                code: usize,
                reason: String,
                was_clean: bool
            ) -> ::worker::js_sys::Promise {
                #close_body
            }

            #[wasm_bindgen(js_name = #error_name, wasm_bindgen=::worker::wasm_bindgen)]
            pub fn __websocket_error(
                key: u32,
                ws: ::worker::worker_sys::web_sys::WebSocket,
                error: ::worker::wasm_bindgen::JsValue
            ) -> ::worker::js_sys::Promise {
                #error_body
            }
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

    let target_name = &target.ident;
    let class_str = target_name.to_string();

    // Build JS export names: ClassName__method
    // The init name uses a distinctive suffix so worker-build can reliably
    // detect DO classes without false-positiving on user-defined exports.
    let init_js = format!("{class_str}__DURABLE_OBJECT_INIT");
    let fetch_js = format!("{class_str}__fetch");
    let alarm_js = format!("{class_str}__alarm");
    let ws_msg_js = format!("{class_str}__webSocketMessage");
    let ws_close_js = format!("{class_str}__webSocketClose");
    let ws_err_js = format!("{class_str}__webSocketError");

    let mut fns = vec![
        bindgen_fns::init(target_name, &init_js),
        bindgen_fns::fetch(target_name, &fetch_js),
    ];

    match durable_object_type {
        None => {
            fns.push(bindgen_fns::alarm(target_name, &alarm_js));
            fns.push(bindgen_fns::websocket(
                target_name,
                &ws_msg_js,
                &ws_close_js,
                &ws_err_js,
            ));
        }
        Some(DurableObjectType::Fetch) => {}
        Some(DurableObjectType::Alarm) => {
            fns.push(bindgen_fns::alarm(target_name, &alarm_js));
        }
        Some(DurableObjectType::WebSocket) => {
            fns.push(bindgen_fns::websocket(
                target_name,
                &ws_msg_js,
                &ws_close_js,
                &ws_err_js,
            ));
        }
    };

    let instances_static = format_ident!("__DO_INSTANCES_{}", class_str);

    Ok(quote! {
        #target

        impl ::worker::has_durable_object_attribute for #target_name {}

        const _: () = {
            use ::worker::wasm_bindgen::prelude::*;
            #[allow(unused_imports)]
            use ::worker::DurableObject;

            ::std::thread_local! {
                static #instances_static: ::std::cell::RefCell<
                    ::std::collections::HashMap<u32, #target_name>
                > = ::std::cell::RefCell::new(::std::collections::HashMap::new());
            }

            use #instances_static as __INSTANCES;

            #(#fns)*
        };
    })
}
