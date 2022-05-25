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
    }
    use HandlerType::*;

    let mut handler_type = None;
    let mut respond_with_errors = false;
    let mut http_support = false;

    for attr in attrs {
        match attr.to_string().as_str() {
            "fetch" => handler_type = Some(Fetch),
            "scheduled" => handler_type = Some(Scheduled),
            "start" => handler_type = Some(Start),
            "respond_with_errors" => {
                respond_with_errors = true;
            }
            "http" => {
                http_support = true;
            }
            _ => panic!("Invalid attribute: {}", attr),
        }
    }
    let handler_type = handler_type.expect(
        "must have either 'fetch', 'scheduled', or 'start' attribute, e.g. #[event(fetch)]",
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

            let error_handling = match respond_with_errors {
                true => {
                    quote! {
                        ::worker::Response::error(e.to_string(), 500).unwrap().into()
                    }
                }
                false => {
                    quote! { panic!("{}", e) }
                }
            };

            let request_transform = if http_support {
                quote! {
                    let request: ::worker::http::Request<::worker::ByteStream> = ::worker::http_types::EdgeRequest(req).try_into().unwrap();
                }
            } else {
                quote! {
                    let request = ::worker::Request::from(req);
                }
            };

            // create a new "main" function that takes the worker_sys::Request, and calls the
            // original attributed function, passing in a converted worker::Request
            let wrapper_fn = quote! {
                pub async fn #wrapper_fn_ident(
                    req: ::worker::worker_sys::Request,
                    env: ::worker::Env,
                    ctx: ::worker::worker_sys::Context
                ) -> ::worker::worker_sys::Response {
                    use std::convert::{TryFrom, TryInto};

                    let ctx = worker::Context::new(ctx);

                    #request_transform

                    // get the worker::Result<worker::Response> by calling the original fn
                    match #input_fn_ident(request, env, ctx).await {
                        Ok(resp) => {
                            let resp: ::worker::Response = resp.try_into().unwrap();
                            resp.try_into().unwrap()
                        },
                        Err(e) => {
                            ::worker::console_log!("{}", &e);
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
