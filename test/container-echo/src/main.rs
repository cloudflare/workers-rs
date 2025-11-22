use std::io;

use axum::{
    body::Body,
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let router = Router::new()
        .route("/ping", get(|| async {}))
        .route("/echo", post(|b: Body| async { b }))
        .route("/ws", get(ws_handler));
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, router).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_ws)
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
