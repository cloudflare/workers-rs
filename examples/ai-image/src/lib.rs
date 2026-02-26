//! Example demonstrating AI image generation using Workers AI.
//!
//! This example shows how to use the `run_bytes` method to generate images
//! using Stable Diffusion and return them as a PNG response.

use futures_util::StreamExt;
use serde::Serialize;
use worker::{event, Env, Request, Response, Result, Router, Url};

#[derive(Serialize)]
struct ImageGenRequest {
    prompt: String,
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/stream", |req, ctx| async move {
            // This endpoint streams the image directly without buffering
            generate_image_stream(req, ctx.env).await
        })
        .get_async("/buffered", |req, ctx| async move {
            // This endpoint buffers the entire image before responding
            generate_image_buffered(req, ctx.env).await
        })
        .run(req, env)
        .await
}

/// Stream the image directly to the response without buffering.
/// This is more memory-efficient for large responses.
async fn generate_image_stream(req: Request, env: Env) -> Result<Response> {
    let url = req.url()?;
    let prompt = get_prompt_from_url(&url).unwrap_or_else(|| "a beautiful sunset".to_string());

    let ai = env.ai("AI")?;
    let request = ImageGenRequest { prompt };

    // Get the ByteStream from the AI model
    let stream = ai
        .run_bytes("@cf/stabilityai/stable-diffusion-xl-base-1.0", &request)
        .await?;

    // Stream directly to the response without buffering in memory
    let mut response = Response::from_stream(stream)?;
    response.headers_mut().set("Content-Type", "image/png")?;

    Ok(response)
}

/// Buffer the entire image before responding.
/// This allows you to inspect or modify the bytes before sending.
async fn generate_image_buffered(req: Request, env: Env) -> Result<Response> {
    let url = req.url()?;
    let prompt = get_prompt_from_url(&url).unwrap_or_else(|| "a beautiful sunset".to_string());

    let ai = env.ai("AI")?;
    let request = ImageGenRequest { prompt };

    // Get the ByteStream from the AI model
    let mut stream = ai
        .run_bytes("@cf/stabilityai/stable-diffusion-xl-base-1.0", &request)
        .await?;

    // Collect all chunks into a Vec<u8>
    let mut image_bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        image_bytes.extend_from_slice(&chunk?);
    }

    // Return the image with appropriate content type
    let mut response = Response::from_bytes(image_bytes)?;
    response.headers_mut().set("Content-Type", "image/png")?;

    Ok(response)
}

fn get_prompt_from_url(url: &Url) -> Option<String> {
    url.query_pairs()
        .find(|(key, _)| key == "prompt")
        .map(|(_, value)| value.into_owned())
}
