use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Error, FnArg, Ident, ItemFn, Meta, Result, Token, Type};

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Fetch,
    Scheduled,
    Queue,
    Email,
    Connect,
    Start,
}

impl Kind {
    fn parse(ident: &Ident) -> Option<Kind> {
        Some(match ident.to_string().as_str() {
            "fetch" => Kind::Fetch,
            "scheduled" => Kind::Scheduled,
            "queue" => Kind::Queue,
            "email" => Kind::Email,
            "connect" => Kind::Connect,
            "start" => Kind::Start,
            _ => return None,
        })
    }

    fn js_name(self) -> &'static str {
        match self {
            Kind::Fetch => "fetch",
            Kind::Scheduled => "scheduled",
            Kind::Queue => "queue",
            Kind::Email => "email",
            Kind::Connect => "connect",
            Kind::Start => "start",
        }
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let metas = syn::parse::Parser::parse2(Punctuated::<Meta, Token![,]>::parse_terminated, attr)?;
    let mut kind = None;
    let mut respond_with_errors = false;
    for meta in &metas {
        let ident = match meta {
            Meta::Path(path) => path.get_ident(),
            Meta::List(list) if list.path.is_ident("connect") => {
                return Err(Error::new(
                    meta.span(),
                    "`connect` has no protocol option: one export serves TCP and UDP, \
                     dispatch on the socket's protocol in the handler",
                ))
            }
            _ => None,
        };
        match ident {
            Some(ident) if ident == "respond_with_errors" => respond_with_errors = true,
            Some(ident) if Kind::parse(ident).is_some() && kind.is_none() => {
                kind = Kind::parse(ident);
            }
            _ => {
                return Err(Error::new(
                    meta.span(),
                    "expected one of `fetch`, `scheduled`, `queue`, `email`, `connect`, \
                     `start`, optionally with `respond_with_errors`",
                ))
            }
        }
    }
    let Some(kind) = kind else {
        return Err(Error::new(
            Span::call_site(),
            "#[event] needs a handler kind, e.g. #[event(fetch)]",
        ));
    };
    if respond_with_errors && kind != Kind::Fetch {
        return Err(Error::new(
            Span::call_site(),
            "`respond_with_errors` only applies to `fetch` handlers",
        ));
    }

    let func: ItemFn = syn::parse2(item)?;
    let sig = &func.sig;
    let name = &sig.ident;

    if kind == Kind::Start {
        if sig.asyncness.is_some() || !sig.inputs.is_empty() {
            return Err(Error::new(
                sig.span(),
                "a `start` handler is a synchronous function with no arguments",
            ));
        }
        return Ok(quote! {
            #[::ts_gen_worker::wasm_bindgen::prelude::wasm_bindgen(
                start,
                wasm_bindgen = ::ts_gen_worker::wasm_bindgen
            )]
            #func
        });
    }

    if sig.asyncness.is_none() {
        return Err(Error::new(
            sig.span(),
            format!("the `{}` handler must be an async function", kind.js_name()),
        ));
    }
    let types = sig
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Typed(pat) => Ok((*pat.ty).clone()),
            FnArg::Receiver(recv) => Err(Error::new(recv.span(), "handlers cannot be methods")),
        })
        .collect::<Result<Vec<Type>>>()?;
    if types.len() != 3 {
        return Err(Error::new(
            sig.inputs.span(),
            format!(
                "the `{}` handler takes 3 arguments (event, env, ctx), found {}",
                kind.js_name(),
                types.len()
            ),
        ));
    }
    let args: Vec<Ident> = (0..3).map(|i| format_ident!("arg{i}")).collect();
    let mut types = types;
    if kind == Kind::Fetch {
        types[0] = syn::parse_quote!(::ts_gen_worker::Request);
    }
    let js_name = kind.js_name();
    let wrapper = format_ident!("__ts_gen_worker_{js_name}");

    let message = if respond_with_errors {
        quote! { ::core::option::Option::Some(::std::string::ToString::to_string(&err)) }
    } else {
        quote! { ::core::option::Option::None }
    };
    let failed = quote! {
        Err(err) => {
            let message = #message;
            ::ts_gen_worker::__macro::error_response(
                ::core::convert::Into::<::ts_gen_worker::wasm_bindgen::JsValue>::into(err),
                message,
            )
        }
    };
    let (output, body) = match kind {
        Kind::Fetch => (
            quote! { ::ts_gen_worker::Response },
            quote! {
                match ::ts_gen_worker::FromRequest::from_request(arg0) {
                    Ok(req) => match #name(req, arg1, arg2).await {
                        Ok(res) => match ::ts_gen_worker::IntoResponse::into_response(res) {
                            Ok(res) => res,
                            #failed
                        },
                        #failed
                    },
                    #failed
                }
            },
        ),
        Kind::Scheduled => (quote! { () }, quote! { #name(#(#args),*).await }),
        Kind::Queue | Kind::Email | Kind::Connect => (
            quote! { ::core::result::Result<(), ::ts_gen_worker::wasm_bindgen::JsValue> },
            quote! {
                #name(#(#args),*).await.map_err(
                    ::core::convert::Into::<::ts_gen_worker::wasm_bindgen::JsValue>::into,
                )
            },
        ),
        Kind::Start => unreachable!(),
    };

    Ok(quote! {
        #func

        const _: () = {
            #[::ts_gen_worker::wasm_bindgen::prelude::wasm_bindgen(
                js_name = #js_name,
                js_namespace = "default",
                wasm_bindgen = ::ts_gen_worker::wasm_bindgen,
                wasm_bindgen_futures = ::ts_gen_worker::wasm_bindgen_futures,
                js_sys = ::ts_gen_worker::js_sys
            )]
            pub async fn #wrapper(#(#args: #types),*) -> #output {
                ::std::panic::AssertUnwindSafe(async move { #body }).await
            }
        };
    })
}
