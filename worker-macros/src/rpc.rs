use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ImplItem, ItemImpl, Type, Error};
use syn::spanned::Spanned;

pub fn expand_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);

    let struct_ident = match &*input.self_ty {
        Type::Path(p) => &p.path.segments.last().unwrap().ident,
        _ => return Error::new(input.self_ty.span(), "Expected a named type").to_compile_error().into(),
    };

    let mut exported_methods = Vec::new();

    for item in &mut input.items {
        if let ImplItem::Fn(ref mut func) = item {
            if let Some(rpc_pos) = func.attrs.iter().position(|attr| attr.path().is_ident("rpc")) {
                func.attrs.remove(rpc_pos);
                func.attrs.insert(0, syn::parse_quote!(#[wasm_bindgen]));
                exported_methods.push(func.clone());
            }
        }
    }

    if exported_methods.is_empty() {
        return Error::new(input.span(), "No methods marked with #[rpc] found.").to_compile_error().into();
    }

    TokenStream::from(quote! {
        #[wasm_bindgen]
        pub struct #struct_ident {
            env: worker::Env,
        }

        #[wasm_bindgen]
        impl #struct_ident {
            #[wasm_bindgen(js_name = "__is_rpc__")]
            pub fn is_rpc(&self) -> bool {
                true
            }

            #[wasm_bindgen(constructor)]
            pub fn new(env: worker::Env) -> Self {
                Self { env }
            }

            #(#exported_methods)*
        }
    })
}

