//! Setup our Cloudflare worker (`feature == "ssr"`) and our leptos hydration function (`feature ==
//! "hydrate"`)

#[cfg(feature = "ssr")]
use worker::*;

use leptos::*;

mod api;
mod app;
mod components;

use crate::app::App;

#[cfg(feature = "ssr")]
async fn router(env: Env) -> axum::Router {
    use std::sync::Arc;

    use axum::{routing::post, Extension};
    use leptos_axum::{generate_route_list, LeptosRoutes};

    use crate::api::register_server_functions;

    // Match what's in Cargo.toml
    // Doesn't seem to be able to do this automatically
    let leptos_options = LeptosOptions {
        output_name: "leptos_worker".into(),
        site_root: "public".into(),
        site_pkg_dir: "pkg".into(),
        env: leptos_config::Env::DEV,
        site_addr: "127.0.0.1:8787".parse().unwrap(),
        reload_port: 3001,
        reload_external_port: None,
        reload_ws_protocol: leptos_config::ReloadWSProtocol::WS,
        not_found_path: "/404".into(),
        hash_file: "hash.txt".into(),
        hash_files: false,
    };
    let routes = generate_route_list(|| view! { <App /> });

    register_server_functions();

    // build our application with a route
    axum::Router::new()
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        .leptos_routes(&leptos_options, routes, || view! { <App/> })
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
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    leptos::mount_to_body(|| view! { <App/> });
}
