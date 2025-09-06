use std::io;

use axum::{
    Router,
    body::Body,
    extract::ws::{WebSocket, WebSocketUpgrade},
    routing::{get, post},
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let router = Router::new()
        .route("/echo", post(|b: Body| async { b }))
        .route(
            "/ws",
            get(|ws: WebSocketUpgrade| async { ws.on_upgrade(handle_ws) }),
        );
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, router).await?;
    Ok(())
}

async fn handle_ws(mut ws: WebSocket) {
    while let Some(msg) = ws.recv().await {
        let msg = match msg {
            Ok(v) => v,
            Err(_) => return,
        };
        if ws.send(msg).await.is_err() {
            return;
        }
    }
}
