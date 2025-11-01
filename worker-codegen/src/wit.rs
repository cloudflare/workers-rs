use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::{format_ident, quote};
use syn::Ident;
use wit_parser::Interface;

fn path_type(name: &str) -> anyhow::Result<syn::Type> {
    let ty: syn::TypePath = syn::parse_str(name)?;
    Ok(syn::Type::Path(ty))
}

fn wit_type_to_syn(
    resolve: &wit_parser::Resolve,
    ty: &wit_parser::Type,
) -> anyhow::Result<syn::Type> {
    Ok(match ty {
        wit_parser::Type::Bool => path_type("bool")?,
        wit_parser::Type::U8 => path_type("u8")?,
        wit_parser::Type::U16 => path_type("u16")?,
        wit_parser::Type::U32 => path_type("u32")?,
        wit_parser::Type::U64 => path_type("u64")?,
        wit_parser::Type::S8 => path_type("i8")?,
        wit_parser::Type::S16 => path_type("i16")?,
        wit_parser::Type::S32 => path_type("i32")?,
        wit_parser::Type::S64 => path_type("i64")?,
        wit_parser::Type::F32 => path_type("f32")?,
        wit_parser::Type::F64 => path_type("f64")?,
        wit_parser::Type::Char => path_type("char")?,
        wit_parser::Type::String => path_type("String")?,
        wit_parser::Type::ErrorContext => {
            anyhow::bail!("Unsupported type: 'ErrorContext'")
        }
        wit_parser::Type::Id(type_id) => {
            let type_def = resolve
                .types
                .get(*type_id)
                .ok_or_else(|| anyhow::anyhow!("Unknown type id: {:?}", type_id))?;
            match &type_def.kind {
                wit_parser::TypeDefKind::List(inner) => {
                    let inner_ty = wit_type_to_syn(resolve, inner)?;
                    #[allow(clippy::match_single_binding)]
                    match inner {
                        // Should this be a special case like so?
                        // https://component-model.bytecodealliance.org/design/wit.html#lists
                        // wit_parser::Type::U8 => {
                        //     syn::parse2::<syn::Type>(quote!(::worker::js_sys::Uint8Array))?
                        // }
                        _ => syn::parse2::<syn::Type>(quote!(::std::vec::Vec<#inner_ty>))?,
                    }
                }
                wit_parser::TypeDefKind::Type(inner) => wit_type_to_syn(resolve, inner)?,
                other => anyhow::bail!("Unsupported type: '{other:?}'"),
            }
        }
    })
}

fn expand_args(
    resolve: &wit_parser::Resolve,
    method: &wit_parser::Function,
) -> anyhow::Result<Vec<syn::FnArg>> {
    let mut args = Vec::with_capacity(method.params.len());
    for (arg_name, arg) in &method.params {
        let param = syn::FnArg::Typed(syn::PatType {
            attrs: vec![],
            pat: Box::new(syn::Pat::Ident(syn::PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: format_ident!("{}", arg_name),
                subpat: None,
            })),
            colon_token: Default::default(),
            ty: Box::new(wit_type_to_syn(resolve, arg)?),
        });
        args.push(param);
    }
    Ok(args)
}

fn method_result_type(
    resolve: &wit_parser::Resolve,
    method: &wit_parser::Function,
) -> anyhow::Result<syn::Type> {
    if let Some(ty) = &method.result {
        wit_type_to_syn(resolve, ty)
    } else {
        anyhow::bail!("Unsupported return type: '{:?}'", method.result);
    }
}

fn expand_trait(
    resolve: &wit_parser::Resolve,
    interface: &Interface,
    interface_name: &Ident,
) -> anyhow::Result<syn::ItemTrait> {
    let trait_raw = quote!(
        #[async_trait::async_trait]
        pub trait #interface_name {
        }
    );
    let mut trait_item: syn::ItemTrait = syn::parse2(trait_raw)?;

    for (name, method) in &interface.functions {
        let ident = format_ident!("{}", name.to_case(Case::Snake));
        let ret_type = method_result_type(resolve, method)?;

        let method_raw = quote!(
            // TODO: docs
            async fn #ident(&self) -> ::worker::Result<#ret_type>;
        );

        let mut method_item: syn::TraitItemFn = syn::parse2(method_raw)?;

        method_item.sig.inputs.extend(expand_args(resolve, method)?);
        trait_item.items.push(syn::TraitItem::Fn(method_item));
    }

    Ok(trait_item)
}

fn expand_struct(struct_name: &Ident, sys_name: &Ident) -> anyhow::Result<syn::ItemStruct> {
    let struct_raw = quote!(
        pub struct #struct_name(::worker::send::SendWrapper<sys::#sys_name>);
    );
    let struct_item: syn::ItemStruct = syn::parse2(struct_raw)?;
    Ok(struct_item)
}

fn expand_from_impl(struct_name: &Ident, from_type: &syn::Type) -> anyhow::Result<syn::ItemImpl> {
    let impl_raw = quote!(
        impl From<#from_type> for #struct_name {
            fn from(fetcher: #from_type) -> Self {
                Self(::worker::send::SendWrapper::new(fetcher.into_rpc()))
            }
        }
    );
    let impl_item: syn::ItemImpl = syn::parse2(impl_raw)?;
    Ok(impl_item)
}

fn expand_rpc_impl(
    resolve: &wit_parser::Resolve,
    interface: &Interface,
    interface_name: &Ident,
    struct_name: &Ident,
) -> anyhow::Result<syn::ItemImpl> {
    let impl_raw = quote!(
        #[async_trait::async_trait]
        impl #interface_name for #struct_name {}
    );
    let mut impl_item: syn::ItemImpl = syn::parse2(impl_raw)?;

    for (name, method) in &interface.functions {
        println!("\tFound method: '{name}'.");
        let ident = format_ident!("{}", name.to_case(Case::Snake));
        let invocation_raw = quote!(self.0.#ident());
        let mut invocation_item: syn::ExprMethodCall = syn::parse2(invocation_raw)?;
        for (arg_name, _) in &method.params {
            let mut segments = syn::punctuated::Punctuated::new();
            segments.push(syn::PathSegment {
                ident: format_ident!("{}", arg_name),
                arguments: syn::PathArguments::None,
            });
            invocation_item.args.push(syn::Expr::Path(syn::ExprPath {
                attrs: vec![],
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments,
                },
            }));
        }

        let ret_type = method_result_type(resolve, method)?;

        let method_raw = quote!(
            async fn #ident(&self) -> ::worker::Result<#ret_type> {
                let promise = #invocation_item?;
                let fut = ::worker::send::SendFuture::new(::worker::wasm_bindgen_futures::JsFuture::from(promise));
                let output = fut.await?;
                Ok(::serde_wasm_bindgen::from_value(output)?)
            }
        );

        let mut method_item: syn::ImplItemFn = syn::parse2(method_raw)?;
        method_item.sig.inputs.extend(expand_args(resolve, method)?);
        impl_item.items.push(syn::ImplItem::Fn(method_item));
    }
    Ok(impl_item)
}

fn expand_sys_module(
    resolve: &wit_parser::Resolve,
    interface: &Interface,
    sys_name: &Ident,
) -> anyhow::Result<syn::ItemMod> {
    let f_mod_raw = quote!(
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(extends=::worker::js_sys::Object)]
            pub type #sys_name;
        }
    );
    let mut f_mod_item: syn::ItemForeignMod = syn::parse2(f_mod_raw)?;

    for (name, method) in &interface.functions {
        let ident = format_ident!("{}", name.to_case(Case::Snake));
        let extern_name = name.to_case(Case::Camel);
        let method_raw = quote!(
            #[wasm_bindgen(method, catch, js_name = #extern_name)]
            // TODO: args
            pub fn #ident(
                this: &#sys_name,
            ) -> std::result::Result<::worker::js_sys::Promise, ::worker::wasm_bindgen::JsValue>;
        );
        let mut method_item: syn::ForeignItemFn = syn::parse2(method_raw)?;
        method_item.sig.inputs.extend(expand_args(resolve, method)?);
        f_mod_item.items.push(syn::ForeignItem::Fn(method_item));
    }

    let mod_raw = quote!(
        mod sys {
            use ::wasm_bindgen::prelude::*;
        }
    );
    let mut mod_item: syn::ItemMod = syn::parse2(mod_raw)?;
    if let Some(ref mut content) = mod_item.content {
        content.1.push(syn::Item::ForeignMod(f_mod_item));
    }

    Ok(mod_item)
}

fn expand_wit(path: &str) -> anyhow::Result<syn::File> {
    let mut resolver = wit_parser::Resolve::new();
    resolver.push_file(path)?;

    // Items: Trait, Struct, Trait Impl, From Impl, Sys Module
    let mut items = Vec::with_capacity(resolver.interfaces.len() * 5);

    for (_, interface) in resolver.interfaces.iter() {
        let name = interface.name.clone().unwrap();
        println!("Found Interface: '{name}'");
        let interface_name = format_ident!("{}", name.to_case(Case::Pascal));
        println!("Generating Trait '{interface_name}'");
        let struct_name = format_ident!("{}Service", interface_name);
        let sys_name = format_ident!("{}Sys", interface_name);

        // Sys Module
        items.push(syn::Item::Mod(expand_sys_module(
            &resolver, interface, &sys_name,
        )?));
        //  Trait
        items.push(syn::Item::Trait(expand_trait(
            &resolver,
            interface,
            &interface_name,
        )?));
        // Struct
        items.push(syn::Item::Struct(expand_struct(&struct_name, &sys_name)?));
        // Trait Impl
        items.push(syn::Item::Impl(expand_rpc_impl(
            &resolver,
            interface,
            &interface_name,
            &struct_name,
        )?));
        // From Impl for Fetcher and Stub
        items.push(syn::Item::Impl(expand_from_impl(
            &struct_name,
            &syn::parse_str("::worker::Fetcher")?,
        )?));
        items.push(syn::Item::Impl(expand_from_impl(
            &struct_name,
            &syn::parse_str("::worker::Stub")?,
        )?));
    }

    let rust_file = syn::File {
        shebang: None,
        attrs: vec![],
        items,
    };
    Ok(rust_file)
}

/// Expands a WIT file into a Rust source file as a string.
pub fn expand_wit_source(path: &str) -> anyhow::Result<String> {
    let file = expand_wit(path)?;
    Ok(prettyplease::unparse(&file))
}

/// Expands a WIT file into a Rust source file as a token stream.
pub fn expand_wit_tokens(path: &str) -> anyhow::Result<TokenStream> {
    let file = expand_wit(path)?;
    Ok(file.into_token_stream())
}
