use std::cell::RefCell;

use worker::*;

#[durable_object]
pub struct Counter {
    count: RefCell<usize>,
    state: State,
    initialized: RefCell<bool>,
    env: Env,
}

#[durable_object]
impl DurableObject for Counter {
    fn new(state: State, env: Env) -> Self {
        Self {
            count: RefCell::new(0),
            initialized: RefCell::new(false),
            state,
            env,
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        if !*self.initialized.borrow() {
            *self.initialized.borrow_mut() = true;
            *self.count.borrow_mut() = self.state.storage().get("count").await.unwrap_or(0);
        }

        if req.path().eq("/ws") {
            let pair = WebSocketPair::new()?;
            let server = pair.server;
            // accept websocket with hibernation api
            self.state.accept_web_socket(&server);
            server
                .serialize_attachment("hello")
                .expect("failed to serialize attachment");

            return Ok(ResponseBuilder::new()
                .with_status(101)
                .with_websocket(pair.client)
                .empty());
        }

        *self.count.borrow_mut() += 10;
        self.state
            .storage()
            .put("count", *self.count.borrow())
            .await?;

        Response::ok(format!(
            "[durable_object]: self.count: {}, secret value: {}",
            self.count.borrow(),
            self.env.secret("SOME_SECRET")?
        ))
    }

    async fn websocket_message(
        &self,
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
        &self,
        _ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> Result<()> {
        Ok(())
    }

    async fn websocket_error(&self, _ws: WebSocket, _error: Error) -> Result<()> {
        Ok(())
    }
}
