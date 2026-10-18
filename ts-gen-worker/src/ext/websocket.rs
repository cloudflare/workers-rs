use crate::{WebSocket, WebSocketPair};

/// Upstream declares the pair as an object keyed `0` and `1`, which lowers to `_0()` /
/// `_1()` getters; `let (client, server) = WebSocketPair::new()?.into();` reads as
/// `worker`'s `WebSocketPair { client, server }`.
impl From<WebSocketPair> for (WebSocket, WebSocket) {
    fn from(pair: WebSocketPair) -> Self {
        (pair._0(), pair._1())
    }
}
