use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use syn::{Error, ImplItem, Item, spanned::Spanned};

pub fn expand_macro(tokens: TokenStream) -> syn::Result<TokenStream> {
    let item = syn::parse2::<Item>(tokens)?;
    match item {
        Item::Impl(imp) => {
            let impl_token = imp.impl_token;
            let trai = imp.trait_.clone();
            let (_, trai, _) = trai.ok_or_else(|| Error::new_spanned(impl_token, "Must be a DurableObject trait impl"))?;
            if !trai.segments.last().map(|x| x.ident == "DurableObject").unwrap_or(false) {
                return Err(Error::new(trai.span(), "Must be a DurableObject trait impl"))
            }

            let pound = syn::Token![#](imp.span()).to_token_stream();
            let wasm_bindgen_attr = quote! {#pound[::wasm_bindgen::prelude::wasm_bindgen]};


            let struct_name = imp.self_ty;
            let items = imp.items;
            let mut tokenized = vec![];
            for item in items {
                let mut method = match item {
                    ImplItem::Method(m) => m, 
                    _ => return Err(Error::new_spanned(item, "Impl block must only contain methods"))
                };
                let tokens = match method.sig.ident.to_string().as_str() {
                    "constructor" => {
                        method.sig.ident = Ident::new("_constructor", method.sig.ident.span());
                        quote! {
                            #pound[::wasm_bindgen::prelude::wasm_bindgen(constructor)]
                            pub #method
                        }
                    },
                    "fetch" => {
                        method.sig.ident = Ident::new("_fetch_raw", method.sig.ident.span());
                        quote! {
                            #pound[::wasm_bindgen::prelude::wasm_bindgen(js_name = fetch)]
                            pub fn _fetch(&mut self, req: ::edgeworker_sys::Request) -> ::js_sys::Promise {
                                // SAFETY:
                                // On the surface, this is unsound because the Durable Object could be dropped
                                // while JavaScript still has possession of the future. However,
                                // we know something that Rust doesn't: that the Durable Object will never be destroyed
                                // while there is still a running promise inside of it, therefore we can let a reference
                                // to the durable object escape into a static-lifetime future.
                                let static_self: &'static mut Self = unsafe {&mut *(self as *mut _)};

                                ::wasm_bindgen_futures::future_to_promise(async move {
                                    static_self._fetch_raw(req.into()).await.map(::edgeworker_sys::Response::from).map(::wasm_bindgen::JsValue::from)
                                        .map_err(::wasm_bindgen::JsValue::from)
                                })
                            }

                            #method
                        }
                    },
                    _ => panic!()
                };
                tokenized.push(tokens);
            }
            Ok(quote! {
                #wasm_bindgen_attr
                impl #struct_name {
                    #(#tokenized)*
                }

                #pound[async_trait::async_trait(?Send)]
                impl ::worker::durable::DurableObject for #struct_name {
                    fn constructor(state: ::worker::durable::State, env: ::worker::Env) -> Self {
                        Self::_constructor(state, env)
                    }

                    async fn fetch(&mut self, req: ::worker::Request) -> ::worker::Result<worker::Response> {
                        self._fetch_raw(req).await
                    }
                }

                trait __Need_Durable_Object_Trait_Impl_With_durable_object_Attribute { const MACROED: bool = true; }
                impl __Need_Durable_Object_Trait_Impl_With_durable_object_Attribute for #struct_name {}
            })
        },
        Item::Struct(struc) => {
            let tokens = struc.to_token_stream();
            let pound = syn::Token![#](struc.span()).to_token_stream();
            let struct_name = struc.ident;
            Ok(quote! {
                #pound[::wasm_bindgen::prelude::wasm_bindgen]
                #tokens

                const _: bool = <#struct_name as __Need_Durable_Object_Trait_Impl_With_durable_object_Attribute>::MACROED;
            })
        },
        _ => Err(Error::new(item.span(), "Durable Object macro can only be applied to structs and their impl of DurableObject trait"))
    }
}