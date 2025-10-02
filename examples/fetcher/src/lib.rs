use worker::{event, Context, Env, Fetch, Request, Response, Result, RouteContext, Router, Url};

const POKEMON_API_URL: &str = "https://pokeapi.co/api/v2/pokemon";

/// Fetches Pokémon data from the PokeAPI given a full URL.
async fn fetch_pokemon(url_string: &str) -> Result<Response> {
    // construct a new Url
    let url = Url::parse(url_string)?;
    Fetch::Url(url).send().await
}

/// Route handler for GET /pokemon/:name
async fn get_pokemon(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    let name = ctx
        .param("name")
        .ok_or_else(|| worker::Error::RustError("Missing 'name' param".into()))?;
    let url = format!("{POKEMON_API_URL}/{name}");
    fetch_pokemon(&url).await
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/pokemon/:name", get_pokemon)
        .run(req, env)
        .await
}
