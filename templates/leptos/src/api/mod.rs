pub mod say_hello;

#[cfg(feature = "ssr")]
pub fn register_server_functions() {
    use leptos::server_fn::axum::register_explicit;

    // Add all of your server functions here

    register_explicit::<say_hello::SayHello>();
}
