use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

pub fn expand(item: TokenStream) -> syn::Result<TokenStream> {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
        ..
    } = syn::parse2(item)?;
    if sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            sig.fn_token,
            "#[send] applies to async functions",
        ));
    }
    let stmts = &block.stmts;
    Ok(quote! {
        #(#attrs)* #vis #sig {
            ::ts_gen_worker::send::SendFuture::new(async {
                #(#stmts)*
            }).await
        }
    })
}
