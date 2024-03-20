mod durable_object;
mod event;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn durable_object(_attr: TokenStream, item: TokenStream) -> TokenStream {
    durable_object::expand_macro(item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(feature = "http")]
#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    event::expand_macro(attr, item, true)
}

#[cfg(not(feature = "http"))]
#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    event::expand_macro(attr, item, false)
}

#[proc_macro_attribute]
pub fn send(_attr: TokenStream, stream: TokenStream) -> TokenStream {
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
            worker::SendFuture::new(async {
                #(#stmts)*
            }).await
        }
    };

    TokenStream::from(tokens)
}
