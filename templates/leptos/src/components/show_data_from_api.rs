use leptos::{prelude::*, task::spawn_local};

use crate::api::say_hello;

#[component]
pub fn ShowDataFromApi() -> impl IntoView {
    let (value, set_value) = signal(String::new());
    let (counter, set_counter) = signal(0);

    let on_click = move |_| {
        spawn_local(async move {
            let api_said = say_hello(counter.get()).await.unwrap();
            set_value.set(api_said);
            set_counter.update(|value| *value += 1);
        });
    };

    view! {
        <div>
            <button on:click=on_click>"What does the API say?"</button>
            <p>{move || value.get()}</p>
        </div>
    }
}