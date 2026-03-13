use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, Ident, ItemFn};

#[derive(strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
enum HandlerType {
    Fetch,
    Scheduled,
    Start,
    #[cfg(feature = "queue")]
    Queue,
}

fn validate_event_fn(
    input_fn: &ItemFn,
    handler_type: HandlerType,
    expected_params: usize,
    must_be_async: bool,
) {
    let sig = &input_fn.sig;

    if must_be_async != sig.asyncness.is_some() {
        let not = if must_be_async { "" } else { " not" };
        panic!("the `{handler_type}` handler must{not} be an async function");
    }

    let argument_word_form = if expected_params == 1 {
        "argument"
    } else {
        "arguments"
    };

    let actual_params = sig.inputs.len();
    if actual_params != expected_params {
        panic!(
            "the `{handler_type}` handler should be a function with {expected_params} {argument_word_form} but found {actual_params}"
        );
    }
}

pub fn expand_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs: Punctuated<Ident, Comma> =
        parse_macro_input!(attr with Punctuated::parse_terminated);

    use HandlerType::*;

    let mut handler_type = None;
    let mut respond_with_errors = false;

    for attr in attrs {
        let attr_str = attr.to_string();
        if attr_str == "respond_with_errors" {
            respond_with_errors = true;
        } else if let Ok(ht) = attr_str.parse() {
            handler_type = Some(ht);
        } else {
            panic!("Invalid attribute: {attr}");
        }
    }
    let handler_type = handler_type.expect(
        "must have either 'fetch', 'scheduled', 'queue' or 'start' attribute, e.g. #[event(fetch)]",
    );

    // create new var using syn item of the attributed fn
    let mut input_fn = parse_macro_input!(item as ItemFn);

    match handler_type {
        Fetch => {
            validate_event_fn(&input_fn, Fetch, 3, true);
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
            // original attributed function, passing in a converted worker::Request.
            // We use a synchronous wrapper that returns a Promise via future_to_promise
            // with AssertUnwindSafe to support panic=unwind.
            let wrapper_fn = quote! {
                pub fn #wrapper_fn_ident(
                    req: ::worker::worker_sys::web_sys::Request,
                    env: ::worker::Env,
                    ctx: ::worker::worker_sys::Context
                ) -> ::worker::js_sys::Promise {
                    ::worker::wasm_bindgen_futures::future_to_promise(::std::panic::AssertUnwindSafe(async move {
                        let ctx = worker::Context::new(ctx);
                        let response: ::worker::worker_sys::web_sys::Response = match ::worker::FromRequest::from_raw(req) {
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
                        };
                        Ok(::worker::wasm_bindgen::JsValue::from(response))
                    }))
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
            validate_event_fn(&input_fn, Scheduled, 3, true);
            // save original fn name for re-use in the wrapper fn
            let input_fn_ident = Ident::new(
                &(input_fn.sig.ident.to_string() + "_scheduled_glue"),
                input_fn.sig.ident.span(),
            );
            let wrapper_fn_ident = Ident::new("scheduled", input_fn.sig.ident.span());
            // rename the original attributed fn
            input_fn.sig.ident = input_fn_ident.clone();

            // Use a synchronous wrapper that returns a Promise via future_to_promise
            // with AssertUnwindSafe to support panic=unwind.
            let wrapper_fn = quote! {
                pub fn #wrapper_fn_ident(event: ::worker::worker_sys::ScheduledEvent, env: ::worker::Env, ctx: ::worker::worker_sys::ScheduleContext) -> ::worker::js_sys::Promise {
                    ::worker::wasm_bindgen_futures::future_to_promise(::std::panic::AssertUnwindSafe(async move {
                        // call the original fn
                        #input_fn_ident(::worker::ScheduledEvent::from(event), env, ::worker::ScheduleContext::from(ctx)).await;
                        Ok(::worker::wasm_bindgen::JsValue::UNDEFINED)
                    }))
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
            validate_event_fn(&input_fn, Queue, 3, true);
            // save original fn name for re-use in the wrapper fn
            let input_fn_ident = Ident::new(
                &(input_fn.sig.ident.to_string() + "_queue_glue"),
                input_fn.sig.ident.span(),
            );
            let wrapper_fn_ident = Ident::new("queue", input_fn.sig.ident.span());
            // rename the original attributed fn
            input_fn.sig.ident = input_fn_ident.clone();

            // Use a synchronous wrapper that returns a Promise via future_to_promise
            // with AssertUnwindSafe to support panic=unwind.
            let wrapper_fn = quote! {
                pub fn #wrapper_fn_ident(event: ::worker::worker_sys::MessageBatch, env: ::worker::Env, ctx: ::worker::worker_sys::Context) -> ::worker::js_sys::Promise {
                    ::worker::wasm_bindgen_futures::future_to_promise(::std::panic::AssertUnwindSafe(async move {
                        // call the original fn
                        let ctx = worker::Context::new(ctx);
                        match #input_fn_ident(::worker::MessageBatch::from(event), env, ctx).await {
                            Ok(()) => {},
                            Err(e) => {
                                ::worker::console_log!("{}", &e);
                                panic!("{}", e);
                            }
                        }
                        Ok(::worker::wasm_bindgen::JsValue::UNDEFINED)
                    }))
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
        Start => {
            validate_event_fn(&input_fn, Start, 0, false);
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
