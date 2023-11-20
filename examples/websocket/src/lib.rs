//! This is the Edge Chat Demo built using Durable Object.
//!
//! # Prerequisites
//! * #### Workers Paid plan
//! * #### Configure Durable Object bindings
//!
//! This demo requires the usage of Durable Objects in order to persist state over multiple clients and connections.
//! Using Durable Objects also allows real time communication between multiple clients and the server.
//!
//! This demo demonstrates a way to hold existing connections and generate messages to connected clients.

mod counter;
mod error;
mod helpers;
mod message;
mod route;
mod storage;
mod timeout;
mod user;

use worker::{event, Context, Env, Request, Response, Result, RouteContext, Router, Stub};

const INDEX_HTML: &str = include_str!("./index.html");

fn index(_: Request, _: RouteContext<()>) -> Result<Response> {
    Response::from_html(INDEX_HTML)
}

async fn websocket(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    DurableObject::new(&ctx, "LIVE_CHAT", "/chat")?
        .fetch(req)
        .await
}

pub struct DurableObject(Stub);

impl DurableObject {
    pub fn new<D>(ctx: &RouteContext<D>, binding: &str, name: &str) -> Result<Self> {
        // durable object's binding name
        let namespace = ctx.durable_object(binding)?;
        // room's name, it can be anything but the below will be used for the demo
        let id = namespace.id_from_name(name)?;
        // durable object's stub
        let stub = id.get_stub()?;

        Ok(Self(stub))
    }

    pub async fn fetch(&self, req: Request) -> Result<Response> {
        // calling fetch_with_request will call counter::LiveCounter::fetch method
        let stub_fetch = self.0.fetch_with_request(req);
        stub_fetch.await
    }
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> worker::Result<Response> {
    Router::new()
        .get("/", index)
        .on_async("/chat/:name", websocket)
        .run(req, env)
        .await
}
