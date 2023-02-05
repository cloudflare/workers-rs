use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Ident, ItemFn};

pub fn expand_macro(_: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let ident = input_fn.sig.ident.clone();
    let wrapper_ident = Ident::new(
        &format!("__worker_test_{}", input_fn.sig.ident),
        input_fn.sig.ident.span(),
    );

    let wrapper_fn = if input_fn.sig.asyncness.is_none() {
        quote! {
            pub fn #wrapper_ident(env: ::worker::Env) {
                #input_fn
                #ident(env)
            }
        }
    } else {
        quote! {
            pub async fn #wrapper_ident(env: ::worker::Env) {
                #input_fn
                #ident(env).await
            }
        }
    };

    let wasm_bindgen_code = wasm_bindgen_macro_support::expand(
        TokenStream::new().into(),
        wrapper_fn.into_token_stream(),
    )
    .expect("wasm_bindgen macro failed to expand");

    TokenStream::from(wasm_bindgen_code)
}
