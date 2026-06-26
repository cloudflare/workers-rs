use worker::{ Context, Env, Fetch, Request, Response, Result, RouteContext, Router, event };
use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize, Clone)]
pub struct ImageURL {
    pub image_url: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Images {
    pub jpg: ImageURL,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub mal_id: u32,
    pub url: String,
    pub title: String,
    pub date: String,
    pub author_username: String,
    pub author_url: String,
    pub forum_url: String,
    pub images: Images,
    pub comments: u32,
    pub excerpt: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Pagination {
    pub last_visible_page: u32,
    pub has_next_page: bool,
}

// The main struct from the API
#[derive(Serialize, Deserialize, Clone)]
pub struct APIResult {
    pub pagination: Pagination,
    pub data: Vec<Data>,
}

// Handle route for /anime/:id
async fn get_anime_news(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id = ctx.param("id").ok_or_else(|| worker::Error::RustError("Missing 'id' param".into()))?;

    let url = format!("https://api.jikan.moe/v4/anime/{}/news", id); //Ex id: 38524

    let request = http::Request
        ::builder()
        .method("GET")
        .uri(url.as_str())
        .header("Content-Type", "application/json")
        .header("Accept", "*/*")
        .body(String::new())
        .unwrap();

    let api_res: APIResult = Fetch::Request(worker::Request::try_from(request).unwrap())
        .send().await?
        .json().await?;

    Response::from_json(&api_res)
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new().get_async("/anime/:id", get_anime_news).run(req, env).await
}
