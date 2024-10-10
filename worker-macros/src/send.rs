use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

pub fn expand_macro(_attr: TokenStream, stream: TokenStream) -> TokenStream {
    let stream_clone = stream.clone();
    let input = parse_macro_input!(stream_clone as ItemFn);

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;
    let stmts = &block.stmts;

    let tokens = quote! {
        #(#attrs)* #vis #sig {
            worker::send::SendFuture::new(async {
                #(#stmts)*
            }).await
        }
    };

    TokenStream::from(tokens)
}
