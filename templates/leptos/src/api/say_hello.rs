use leptos::*;

#[server(SayHello)]
pub async fn say_hello(num: i32) -> Result<String, ServerFnError> {
    Ok(format!("Hello from the API!!! I got {num}"))
}
