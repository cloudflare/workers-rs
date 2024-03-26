use worker::*;

#[durable_object]
pub struct Counter {
    count: usize,
    state: State,
    initialized: bool,
    env: Env,
}

#[durable_object]
impl DurableObject for Counter {
    fn new(state: State, env: Env) -> Self {
        Self {
            count: 0,
            initialized: false,
            state,
            env,
        }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        if !self.initialized {
            self.initialized = true;
            self.count = self.state.storage().get("count").await.unwrap_or(0);
        }

        if req.path().eq("/ws") {
            let pair = WebSocketPair::new()?;
            let server = pair.server;
            // accept websocket with hibernation api
            self.state.accept_web_socket(&server);
            server
                .serialize_attachment("hello")
                .expect("failed to serialize attachment");

            return Ok(Response::empty()
                .unwrap()
                .with_status(101)
                .with_websocket(Some(pair.client)));
        }

        self.count += 10;
        self.state.storage().put("count", self.count).await?;

        Response::ok(format!(
            "[durable_object]: self.count: {}, secret value: {}",
            self.count,
            self.env.secret("SOME_SECRET")?.to_string()
        ))
    }

    async fn websocket_message(
        &mut self,
        ws: WebSocket,
        _message: WebSocketIncomingMessage,
    ) -> Result<()> {
        let _attach: String = ws
            .deserialize_attachment()?
            .expect("websockets should have an attachment");
        // get and increment storage by 10
        let mut count: usize = self.state.storage().get("count").await.unwrap_or(0);
        count += 10;
        self.state.storage().put("count", count).await?;
        // send value to client
        ws.send_with_str(format!("{}", count))
            .expect("failed to send value to client");
        Ok(())
    }

    async fn websocket_close(
        &mut self,
        _ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> Result<()> {
        Ok(())
    }

    async fn websocket_error(&mut self, _ws: WebSocket, _error: Error) -> Result<()> {
        Ok(())
    }
}
