extern crate wasm_bindgen_macro_support;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn};

#[proc_macro_attribute]
pub fn worker(attr: TokenStream, item: TokenStream) -> TokenStream {
    let exit_missing_attr =
        || panic!("must have either 'fetch' or 'scheduled' attribute, e.g. #[cf::worker(fetch)]");
    if attr.is_empty() {
        exit_missing_attr();
    }

    // create new var using syn item of the attributed fn
    let mut input_fn = parse_macro_input!(item as ItemFn);

    match attr.to_string().as_str() {
        "fetch" => {
            // TODO: validate the inputs / signature
            // let input_arg = input_fn.sig.inputs.first().expect("#[cf::worker(fetch)] attribute requires exactly one input, of type `worker::Request`");

            // save original fn name for re-use in the wrapper fn
            let original_input_fn_ident = input_fn.sig.ident.clone();
            let output_fn_ident = Ident::new("glue_fetch", input_fn.sig.ident.span());
            // rename the original attributed fn
            input_fn.sig.ident = output_fn_ident.clone();

            // create a new "main" function that takes the edgeworker_sys::Request, and calls the
            // original attributed function, passing in a converted worker::Request
            let wrapper_fn = quote! {
                pub async fn #original_input_fn_ident(ty: String, req: edgeworker_sys::Request) -> worker::Result<edgeworker_sys::Response> {
                    // get the worker::Result<worker::Response> by calling the original fn
                    #output_fn_ident(worker::Request::from((ty, req))).await
                        .map(edgeworker_sys::Response::from)
                        .map_err(worker::Error::from)
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
        "scheduled" => {
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
        _ => exit_missing_attr(),
    }
}
