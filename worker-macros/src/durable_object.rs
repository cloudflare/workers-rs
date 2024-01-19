use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Error, FnArg, ImplItem, Item, Type, TypePath, Visibility};

pub fn expand_macro(tokens: TokenStream) -> syn::Result<TokenStream> {
    let item = syn::parse2::<Item>(tokens)?;
    match item {
        Item::Impl(imp) => {
            let impl_token = imp.impl_token;
            let trai = imp.trait_.clone();
            let (_, trai, _) = trai.ok_or_else(|| Error::new_spanned(impl_token, "Must be a DurableObject trait impl"))?;
            if !trai.segments.last().map(|x| x.ident == "DurableObject").unwrap_or(false) {
                return Err(Error::new(trai.span(), "Must be a DurableObject trait impl"))
            }

            let pound = syn::Token![#](imp.span()).to_token_stream();
            let wasm_bindgen_attr = quote! {#pound[wasm_bindgen::prelude::wasm_bindgen]};


            let struct_name = imp.self_ty;
            let items = imp.items;
            let mut tokenized = vec![];

            #[derive(Default)]
            struct OptionalMethods {
                has_alarm: bool,
                has_websocket_message: bool,
                has_websocket_close: bool,
                has_websocket_error: bool,
            }

            let mut optional_methods = OptionalMethods::default();

            for item in items {
                let impl_method = match item {
                    ImplItem::Fn(func) => func,
                    _ => return Err(Error::new_spanned(item, "Impl block must only contain methods"))
                };
                
                let span = impl_method.sig.ident.span();

                let tokens = match impl_method.sig.ident.to_string().as_str() {
                    "new" => {
                        let mut method = impl_method.clone();
                        method.sig.ident = Ident::new("_new", method.sig.ident.span());
                        method.vis = Visibility::Inherited;


                        // modify the `state` argument so it is type ObjectState
                        let arg_tokens = method.sig.inputs.first_mut().expect("DurableObject `new` method must have 2 arguments: state and env").into_token_stream();                        
                        match syn::parse2::<FnArg>(arg_tokens)? {
                            FnArg::Typed(pat) => {
                                let path = syn::parse2::<TypePath>(quote!{worker::worker_sys::DurableObjectState})?;
                                let mut updated_pat = pat;
                                updated_pat.ty = Box::new(Type::Path(path));

                                let state_arg = FnArg::Typed(updated_pat);
                                let env_arg = method.sig.inputs.pop().expect("DurableObject `new` method expects a second argument: env");
                                method.sig.inputs.clear();
                                method.sig.inputs.insert(0, state_arg);
                                method.sig.inputs.insert(1, env_arg.into_value())
                            },
                            _ => return Err(Error::new(method.sig.inputs.span(), "DurableObject `new` method expects `state: State` as first argument."))
                        }

                        // prepend the function block's statements to convert the ObjectState to State type
                        let mut prepended = vec![syn::parse_quote! {
                            let state = ::worker::durable::State::from(state);
                        }];
                        prepended.extend(method.block.stmts);
                        method.block.stmts = prepended;

                        Ok(quote! {
                            #pound[wasm_bindgen::prelude::wasm_bindgen(constructor)]
                            pub #method
                        })
                    },
                    "fetch" => {
                        let mut method = impl_method.clone();
                        method.sig.ident = Ident::new("_fetch_raw", method.sig.ident.span());
                        method.vis = Visibility::Inherited;

                        Ok(quote! {
                            #pound[wasm_bindgen::prelude::wasm_bindgen(js_name = fetch)]
                            pub fn _fetch(&mut self, req: worker::worker_sys::web_sys::Request) -> worker::js_sys::Promise {
                                // SAFETY:
                                // On the surface, this is unsound because the Durable Object could be dropped
                                // while JavaScript still has possession of the future. However,
                                // we know something that Rust doesn't: that the Durable Object will never be destroyed
                                // while there is still a running promise inside of it, therefore we can let a reference
                                // to the durable object escape into a static-lifetime future.
                                let static_self: &'static mut Self = unsafe {&mut *(self as *mut _)};

                                wasm_bindgen_futures::future_to_promise(async move {
                                    static_self._fetch_raw(req.into()).await.map(worker::worker_sys::web_sys::Response::from).map(wasm_bindgen::JsValue::from)
                                        .map_err(wasm_bindgen::JsValue::from)
                                })
                            }

                            #method
                        })
                    },
                    "alarm" => {
                        optional_methods.has_alarm = true;

                        let mut method = impl_method.clone();
                        method.sig.ident = Ident::new("_alarm_raw", method.sig.ident.span());
                        method.vis = Visibility::Inherited;

                        Ok(quote! {
                            #pound[wasm_bindgen::prelude::wasm_bindgen(js_name = alarm)]
                            pub fn _alarm(&mut self) -> worker::js_sys::Promise {
                                // SAFETY:
                                // On the surface, this is unsound because the Durable Object could be dropped
                                // while JavaScript still has possession of the future. However,
                                // we know something that Rust doesn't: that the Durable Object will never be destroyed
                                // while there is still a running promise inside of it, therefore we can let a reference
                                // to the durable object escape into a static-lifetime future.
                                let static_self: &'static mut Self = unsafe {&mut *(self as *mut _)};

                                wasm_bindgen_futures::future_to_promise(async move {
                                    static_self._alarm_raw().await.map(worker::worker_sys::web_sys::Response::from).map(wasm_bindgen::JsValue::from)
                                        .map_err(wasm_bindgen::JsValue::from)
                                })
                            }

                            #method
                        })
                    },
                    "websocket_message" => {
                        optional_methods.has_websocket_message = true;

                        let mut method = impl_method.clone();
                        method.sig.ident = Ident::new("_websocket_message_raw", method.sig.ident.span());
                        method.vis = Visibility::Inherited;

                        Ok(quote! {
                            #pound[wasm_bindgen::prelude::wasm_bindgen(js_name = webSocketMessage)]
                            pub fn _websocket_message(&mut self, ws: worker::worker_sys::web_sys::WebSocket, message: wasm_bindgen::JsValue) -> worker::js_sys::Promise {
                                let ws_message = if let Some(string_message) = message.as_string() {
                                    worker::WebSocketIncomingMessage::String(string_message)
                                } else {
                                    let v = worker::js_sys::Uint8Array::new(&message).to_vec();
                                    worker::WebSocketIncomingMessage::Binary(v)
                                };
                                
                                // SAFETY:
                                // On the surface, this is unsound because the Durable Object could be dropped
                                // while JavaScript still has possession of the future. However,
                                // we know something that Rust doesn't: that the Durable Object will never be destroyed
                                // while there is still a running promise inside of it, therefore we can let a reference
                                // to the durable object escape into a static-lifetime future.
                                let static_self: &'static mut Self = unsafe {&mut *(self as *mut _)};

                                wasm_bindgen_futures::future_to_promise(async move {
                                    static_self._websocket_message_raw(ws.into(), ws_message).await.map(|_| wasm_bindgen::JsValue::NULL)
                                        .map_err(wasm_bindgen::JsValue::from)
                                })
                            }

                            #method
                        })
                    },
                    "websocket_close" => {
                        optional_methods.has_websocket_close = true;

                        let mut method = impl_method.clone();
                        method.sig.ident = Ident::new("_websocket_close_raw", method.sig.ident.span());
                        method.vis = Visibility::Inherited;

                        Ok(quote! {
                            #pound[wasm_bindgen::prelude::wasm_bindgen(js_name = webSocketClose)]
                            pub fn _websocket_close(&mut self, ws: worker::worker_sys::web_sys::WebSocket, code: usize, reason: String, was_clean: bool) -> worker::js_sys::Promise {
                                // SAFETY:
                                // On the surface, this is unsound because the Durable Object could be dropped
                                // while JavaScript still has possession of the future. However,
                                // we know something that Rust doesn't: that the Durable Object will never be destroyed
                                // while there is still a running promise inside of it, therefore we can let a reference
                                // to the durable object escape into a static-lifetime future.
                                let static_self: &'static mut Self = unsafe {&mut *(self as *mut _)};

                                wasm_bindgen_futures::future_to_promise(async move {
                                    static_self._websocket_close_raw(ws.into(), code, reason, was_clean).await.map(|_| wasm_bindgen::JsValue::NULL)
                                        .map_err(wasm_bindgen::JsValue::from)
                                })
                            }

                            #method
                        })
                    },
                    "websocket_error" => {
                        optional_methods.has_websocket_error = true;

                        let mut method = impl_method.clone();
                        method.sig.ident = Ident::new("_websocket_error_raw", method.sig.ident.span());
                        method.vis = Visibility::Inherited;

                        Ok(quote! {
                            #pound[wasm_bindgen::prelude::wasm_bindgen(js_name = webSocketError)]
                            pub fn _websocket_error(&mut self, ws: worker::worker_sys::web_sys::WebSocket, error: wasm_bindgen::JsValue) -> worker::js_sys::Promise {
                                // SAFETY:
                                // On the surface, this is unsound because the Durable Object could be dropped
                                // while JavaScript still has possession of the future. However,
                                // we know something that Rust doesn't: that the Durable Object will never be destroyed
                                // while there is still a running promise inside of it, therefore we can let a reference
                                // to the durable object escape into a static-lifetime future.
                                let static_self: &'static mut Self = unsafe {&mut *(self as *mut _)};

                                wasm_bindgen_futures::future_to_promise(async move {
                                    static_self._websocket_error_raw(ws.into(), error.into()).await.map(|_| wasm_bindgen::JsValue::NULL)
                                        .map_err(wasm_bindgen::JsValue::from)
                                })
                            }

                            #method
                        })
                    },
                    ident => Err(Error::new(span, format!("Unsupported method `{}`, please move extra impl methods to a separate impl definition", ident)))
                };
                tokenized.push(tokens?);
            }

            let alarm_tokens = optional_methods.has_alarm.then(|| quote! {
                async fn alarm(&mut self) -> ::worker::Result<worker::Response> {
                    self._alarm_raw().await
                }
            });
            
            let websocket_message_tokens = optional_methods.has_websocket_message.then(|| quote! {
                async fn websocket_message(&mut self, ws: ::worker::WebSocket, message: ::worker::WebSocketIncomingMessage) -> ::worker::Result<()> {
                    self._websocket_message_raw(ws, message).await
                }
            });

            let websocket_close_tokens = optional_methods.has_websocket_close.then(|| quote! {
                async fn websocket_close(&mut self, ws: ::worker::WebSocket, code: usize, reason: String, was_clean: bool) -> ::worker::Result<()> {
                    self._websocket_close_raw(ws, code, reason, was_clean).await
                }
            });

            let websocket_error_tokens = optional_methods.has_websocket_error.then(|| quote! {
                async fn websocket_error(&mut self, ws: ::worker::WebSocket, error: ::worker::Error) -> ::worker::Result<()> {
                    self._websocket_error_raw(ws, error).await
                }
            });

            Ok(quote! {
                #wasm_bindgen_attr
                impl #struct_name {
                    #(#tokenized)*
                }

                #pound[async_trait::async_trait(?Send)]
                impl ::worker::durable::DurableObject for #struct_name {
                    fn new(state: ::worker::durable::State, env: ::worker::Env) -> Self {
                        Self::_new(state._inner(), env)
                    }

                    async fn fetch(&mut self, req: ::worker::Request) -> ::worker::Result<worker::Response> {
                        self._fetch_raw(req).await
                    }

                    #alarm_tokens

                    #websocket_message_tokens

                    #websocket_close_tokens

                    #websocket_error_tokens
                }

                trait __Need_Durable_Object_Trait_Impl_With_durable_object_Attribute { const MACROED: bool = true; }
                impl __Need_Durable_Object_Trait_Impl_With_durable_object_Attribute for #struct_name {}
            })
        },
        Item::Struct(struc) => {
            let tokens = struc.to_token_stream();
            let pound = syn::Token![#](struc.span()).to_token_stream();
            let struct_name = struc.ident;
            Ok(quote! {
                #pound[wasm_bindgen::prelude::wasm_bindgen]
                #tokens

                const _: bool = <#struct_name as __Need_Durable_Object_Trait_Impl_With_durable_object_Attribute>::MACROED;
            })
        },
        _ => Err(Error::new(item.span(), "Durable Object macro can only be applied to structs and their impl of DurableObject trait"))
    }
}
