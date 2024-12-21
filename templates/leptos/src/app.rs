use crate::components::show_data_from_api::ShowDataFromApi;
use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <h1>"Hello world!"</h1>
        <ShowDataFromApi />
    }
}

// Leptos 0.7
#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    use leptos_meta::*;

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>

                <AutoReload options=options.clone() />

                <HydrationScripts options />

                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}
