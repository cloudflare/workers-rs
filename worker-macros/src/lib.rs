mod durable_object;
mod event;
mod test;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn durable_object(_attr: TokenStream, item: TokenStream) -> TokenStream {
    durable_object::expand_macro(item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    event::expand_macro(attr, item)
}

#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    test::expand_macro(attr, item)
}
