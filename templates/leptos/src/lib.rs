mod app;
mod api;
mod components;

#[cfg(feature = "ssr")]
#[worker::event(fetch)]
async fn fetch(
    req: worker::HttpRequest,
    env: worker::Env,
    _ctx: worker::Context,
) -> worker::Result<axum::http::Response<axum::body::Body>> {
    use std::sync::Arc;

    use axum::{Extension, Router};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tower_service::Service;

    use app::{App, shell};
    
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    // build our application with a route
    let mut router = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .with_state(leptos_options)
        .layer(Extension(Arc::new(env))); // <- Allow leptos server functions to access Worker stuff

    Ok(router.call(req).await?)
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    leptos::mount::hydrate_body(app::App);
}
