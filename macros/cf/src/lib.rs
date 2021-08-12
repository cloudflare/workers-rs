extern crate wasm_bindgen_macro_support;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, Ident, ItemFn};

#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs: Punctuated<Ident, Comma> =
        parse_macro_input!(attr with Punctuated::parse_terminated);

    enum HandlerType {
        Fetch,
        Scheduled,
    }
    use HandlerType::*;

    let mut handler_type = None;
    let mut respond_with_errors = false;

    for attr in attrs {
        match attr.to_string().as_str() {
            "fetch" => handler_type = Some(Fetch),
            "scheduled" => handler_type = Some(Scheduled),
            "respond_with_errors" => {
                respond_with_errors = true;
            }
            _ => panic!("Invalid attribute: {}", attr.to_string()),
        }
    }
    let handler_type = handler_type
        .expect("must have either 'fetch' or 'scheduled' attribute, e.g. #[cf::worker(fetch)]");

    // create new var using syn item of the attributed fn
    let mut input_fn = parse_macro_input!(item as ItemFn);

    match handler_type {
        Fetch => {
            // TODO: validate the inputs / signature
            // let input_arg = input_fn.sig.inputs.first().expect("#[cf::worker(fetch)] attribute requires exactly one input, of type `worker::Request`");

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
                        Response::error(e.to_string(), 500).unwrap().into()
                    }
                }
                false => {
                    quote! { panic!("{}", e) }
                }
            };

            // create a new "main" function that takes the edgeworker_ffi::Request, and calls the
            // original attributed function, passing in a converted worker::Request
            let wrapper_fn = quote! {
                pub async fn #wrapper_fn_ident(req: ::edgeworker_ffi::Request, env: ::worker::Env) -> ::edgeworker_ffi::Response {
                    // get the worker::Result<worker::Response> by calling the original fn
                    match #input_fn_ident(worker::Request::from(req), env).await.map(edgeworker_ffi::Response::from) {
                        Ok(res) => res,
                        Err(e) => {
                            ::worker::prelude::console_log!("{}", &e);
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

                #wasm_bindgen_code
            };

            TokenStream::from(output)
        }
        Scheduled => {
            // save original fn name for re-use in the wrapper fn
            let original_input_fn_ident = input_fn.sig.ident.clone();
            let output_fn_ident = Ident::new("glue_cron", input_fn.sig.ident.span());
            // rename the original attributed fn
            input_fn.sig.ident = output_fn_ident.clone();

            let wrapper_fn = quote! {
                pub async fn #original_input_fn_ident(ty: String, schedule: u64, cron: String) -> worker::Result<()> {
                    // get the worker::Result<worker::Response> by calling the original fn
                    #output_fn_ident(worker::Schedule::from((ty, schedule, cron))).await
                }
            };
            let wasm_bindgen_code =
                wasm_bindgen_macro_support::expand(TokenStream::new().into(), wrapper_fn)
                    .expect("wasm_bindgen macro failed to expand");

            let output = quote! {
                #input_fn

                #wasm_bindgen_code
            };

            TokenStream::from(output)
        }
    }
}

#[proc_macro_attribute]
pub fn durable_object(_attr: TokenStream, item: TokenStream) -> TokenStream {
    durable_object::expand_macro(item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
