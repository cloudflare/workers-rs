#[cfg(feature = "ssr")]
use worker::*;

use crate::app::*;

pub mod app;
mod components;


#[cfg(feature = "ssr")]
#[cfg(target_family = "wasm")]
mod wasm_workaround {
    unsafe extern "C" {
        pub(super) fn __wasm_call_ctors();
    }
}

#[cfg(feature = "ssr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn start() {
    // Fix for 'Read a negative address value from the stack. Did we run out of memory?'.
    // See: https://github.com/cloudflare/workers-rs/issues/772
    #[cfg(target_family = "wasm")]
    unsafe {
        wasm_workaround::__wasm_call_ctors()
    };
}

#[cfg(feature = "ssr")]
pub fn register_server_functions() {
    use leptos::server_fn::axum::register_explicit;

    // Add all of your server functions here
    register_explicit::<components::show_data_from_api::SayHello>();
}

#[cfg(feature = "ssr")]
async fn router(env: Env) -> axum::Router {
    use std::sync::Arc;

    use axum::{Extension, Router};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);
    register_server_functions();

    // build our application with a route
    Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .with_state(leptos_options)
        .layer(Extension(Arc::new(env))) // <- Allow leptos server functions to access Worker stuff
}

#[cfg(feature = "ssr")]
#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    use tower_service::Service;

    console_error_panic_hook::set_once();

    Ok(router(env).await.call(req).await?)
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
