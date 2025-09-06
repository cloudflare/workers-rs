use worker::{event, Context, Env, Fetch, Request, Response, Result, RouteContext, Router, Url};

const ENDPOINT: &str = "https://pokeapi.co/api/v2/pokemon";

async fn fetch_pokemon(url_string: &str) -> Result<Response> {
    // construct a new Url
    let url = Url::parse(url_string)?;
    Fetch::Url(url).send().await
}

async fn handle_single(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    Ok(fetch_pokemon(&format!("{ENDPOINT}/{}", ctx.param("name").unwrap())).await?)
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/pokemon/:name", handle_single)
        .run(req, env)
        .await
}
