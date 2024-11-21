use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, Ident, ItemFn};

pub fn expand_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs: Punctuated<Ident, Comma> =
        parse_macro_input!(attr with Punctuated::parse_terminated);

    enum HandlerType {
        Fetch,
        Scheduled,
        Start,
        #[cfg(feature = "queue")]
        Queue,
        Email,
    }
    use HandlerType::*;

    let mut handler_type = None;
    let mut respond_with_errors = false;

    for attr in attrs {
        match attr.to_string().as_str() {
            "fetch" => handler_type = Some(Fetch),
            "scheduled" => handler_type = Some(Scheduled),
            "start" => handler_type = Some(Start),
            #[cfg(feature = "queue")]
            "queue" => handler_type = Some(Queue),
            "respond_with_errors" => {
                respond_with_errors = true;
            }
            "email" => handler_type = Some(Email),
            _ => panic!("Invalid attribute: {}", attr),
        }
    }
    let handler_type = handler_type.expect(
        "must have either 'fetch', 'scheduled', 'queue' or 'start' attribute, e.g. #[event(fetch)]",
    );

    // create new var using syn item of the attributed fn
    let mut input_fn = parse_macro_input!(item as ItemFn);

    match handler_type {
        Fetch => {
            // TODO: validate the inputs / signature
            // save original fn name for re-use in the wrapper fn
            let input_fn_ident = Ident::new(
                &(input_fn.sig.ident.to_string() + "_fetch_glue"),
                input_fn.sig.ident.span(),
            );
            let wrapper_fn_ident = Ident::new("fetch", input_fn.sig.ident.span());
            // rename the original attributed fn
            input_fn.sig.ident = input_fn_ident.clone();

            let error_handling = if respond_with_errors {
                quote! { ::worker::Response::error(e.to_string(), 500).unwrap().into() }
            } else {
                quote! { ::worker::Response::error("INTERNAL SERVER ERROR", 500).unwrap().into() }
            };

            // create a new "main" function that takes the worker_sys::Request, and calls the
            // original attributed function, passing in a converted worker::Request
            let wrapper_fn = quote! {
                pub async fn #wrapper_fn_ident(
                    req: ::worker::worker_sys::web_sys::Request,
                    env: ::worker::Env,
                    ctx: ::worker::worker_sys::Context
                ) -> ::worker::worker_sys::web_sys::Response {
                    let ctx = worker::Context::new(ctx);
                    match ::worker::FromRequest::from_raw(req) {
                        Ok(req) => {
                            let result = #input_fn_ident(req, env, ctx).await;
                            // get the worker::Result<worker::Response> by calling the original fn
                            match result {
                                Ok(raw_res) => {
                                    match ::worker::IntoResponse::into_raw(raw_res) {
                                        Ok(res) => res,
                                        Err(err) => {
                                            let e: Box<dyn std::error::Error> = err.into();
                                            ::worker::console_error!("Error converting response: {}", &e);
                                            #error_handling
                                        }
                                    }
                                },
                                Err(err) => {
                                    let e: Box<dyn std::error::Error> = err.into();
                                    ::worker::console_error!("{}", &e);
                                    #error_handling
                                }
                            }
                        },
                        Err(err) => {
                            let e: Box<dyn std::error::Error> = err.into();
                            ::worker::console_error!("Error converting request: {}", &e);
                            #error_handling
                        }
                    }
                }
            };
            let wasm_bindgen_code =
                wasm_bindgen_macro_support::expand(TokenStream::new().into(), wrapper_fn)
                    .expect("wasm_bindgen macro failed to expand");

            let output = quote! {
                #input_fn

                mod _worker_fetch {
                    use ::worker::{wasm_bindgen, wasm_bindgen_futures};
                    use super::#input_fn_ident;
                    #wasm_bindgen_code
                }
            };

            TokenStream::from(output)
        }
        Scheduled => {
            // save original fn name for re-use in the wrapper fn
            let input_fn_ident = Ident::new(
                &(input_fn.sig.ident.to_string() + "_scheduled_glue"),
                input_fn.sig.ident.span(),
            );
            let wrapper_fn_ident = Ident::new("scheduled", input_fn.sig.ident.span());
            // rename the original attributed fn
            input_fn.sig.ident = input_fn_ident.clone();

            let wrapper_fn = quote! {
                pub async fn #wrapper_fn_ident(event: ::worker::worker_sys::ScheduledEvent, env: ::worker::Env, ctx: ::worker::worker_sys::ScheduleContext) {
                    // call the original fn
                    #input_fn_ident(::worker::ScheduledEvent::from(event), env, ::worker::ScheduleContext::from(ctx)).await
                }
            };
            let wasm_bindgen_code =
                wasm_bindgen_macro_support::expand(TokenStream::new().into(), wrapper_fn)
                    .expect("wasm_bindgen macro failed to expand");

            let output = quote! {
                #input_fn

                mod _worker_scheduled {
                    use ::worker::{wasm_bindgen, wasm_bindgen_futures};
                    use super::#input_fn_ident;
                    #wasm_bindgen_code
                }
            };

            TokenStream::from(output)
        }
        #[cfg(feature = "queue")]
        Queue => {
            // save original fn name for re-use in the wrapper fn
            let input_fn_ident = Ident::new(
                &(input_fn.sig.ident.to_string() + "_queue_glue"),
                input_fn.sig.ident.span(),
            );
            let wrapper_fn_ident = Ident::new("queue", input_fn.sig.ident.span());
            // rename the original attributed fn
            input_fn.sig.ident = input_fn_ident.clone();

            let wrapper_fn = quote! {
                pub async fn #wrapper_fn_ident(event: ::worker::worker_sys::MessageBatch, env: ::worker::Env, ctx: ::worker::worker_sys::Context) {
                    // call the original fn
                    let ctx = worker::Context::new(ctx);
                    match #input_fn_ident(::worker::MessageBatch::from(event), env, ctx).await {
                        Ok(()) => {},
                        Err(e) => {
                            ::worker::console_log!("{}", &e);
                            panic!("{}", e);
                        }
                    }
                }
            };
            let wasm_bindgen_code =
                wasm_bindgen_macro_support::expand(TokenStream::new().into(), wrapper_fn)
                    .expect("wasm_bindgen macro failed to expand");

            let output = quote! {
                #input_fn

                mod _worker_queue {
                    use ::worker::{wasm_bindgen, wasm_bindgen_futures};
                    use super::#input_fn_ident;
                    #wasm_bindgen_code
                }
            };

            TokenStream::from(output)
        }
        Email => {
            let input_fn_ident = Ident::new(
                &(input_fn.sig.ident.to_string() + "_email_glue"),
                input_fn.sig.ident.span(),
            );
            let wrapper_fn_ident = Ident::new("email_handler", input_fn.sig.ident.span());
            // rename the original attributed fn
            input_fn.sig.ident = input_fn_ident.clone();

            let wrapper_fn = quote! {
                pub async fn #wrapper_fn_ident(message: ::worker::worker_sys::ForwardableEmailMessage, env: ::worker::Env, ctx: ::worker::worker_sys::Context) {
                    // call the original fn
                    let ctx = worker::Context::new(ctx);
                    match #input_fn_ident(::worker::ForwardableEmailMessage::from(message), env, ctx).await {
                        Ok(()) => {},
                        Err(e) => {
                            ::worker::console_log!("{}", &e);
                            panic!("{}", e);
                        }
                    }
                }
            };

            let wasm_bindgen_code =
                wasm_bindgen_macro_support::expand(TokenStream::new().into(), wrapper_fn)
                    .expect("wasm_bindgen macro failed to expand");

            let output = quote! {
                #input_fn

                mod _worker_email {
                    use ::worker::{wasm_bindgen, wasm_bindgen_futures};
                    use super::#input_fn_ident;
                    #wasm_bindgen_code
                }
            };

            TokenStream::from(output)
        }
        Start => {
            // save original fn name for re-use in the wrapper fn
            let input_fn_ident = Ident::new(
                &(input_fn.sig.ident.to_string() + "_start_glue"),
                input_fn.sig.ident.span(),
            );
            let wrapper_fn_ident = Ident::new("start", input_fn.sig.ident.span());
            // rename the original attributed fn
            input_fn.sig.ident = input_fn_ident.clone();

            let wrapper_fn = quote! {
                pub fn #wrapper_fn_ident() {
                    // call the original fn
                    #input_fn_ident()
                }
            };
            let wasm_bindgen_code =
                wasm_bindgen_macro_support::expand(quote! { start }, wrapper_fn)
                    .expect("wasm_bindgen macro failed to expand");

            let output = quote! {
                #input_fn

                mod _worker_start {
                    use ::worker::{wasm_bindgen, wasm_bindgen_futures};
                    use super::#input_fn_ident;
                    #wasm_bindgen_code
                }
            };

            TokenStream::from(output)
        }
    }
}
