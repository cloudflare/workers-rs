use gloo_timers::future::TimeoutFuture;
use std::cell::RefCell;
use worker::*;

#[shared_durable_object]
pub struct SharedCounter {
    count: RefCell<usize>,
    state: RefCell<State>,
    initialized: RefCell<bool>,
    env: Env,
}

#[shared_durable_object]
impl SharedDurableObject for SharedCounter {
    fn new(state: State, env: Env) -> Self {
        Self {
            count: RefCell::new(0),
            initialized: RefCell::new(false),
            state: RefCell::new(state),
            env,
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        if !*self.initialized.borrow() {
            *self.initialized.borrow_mut() = true;
            let storage = self.state.borrow().storage();
            let count = storage.get("count").await.unwrap_or(0);
            *self.count.borrow_mut() = count;
        }

        if req.path().eq("/ws") {
            let pair = WebSocketPair::new()?;
            let server = pair.server;
            // accept websocket with hibernation api
            self.state.borrow().accept_web_socket(&server);
            server
                .serialize_attachment("hello")
                .expect("failed to serialize attachment");

            return Ok(ResponseBuilder::new()
                .with_status(101)
                .with_websocket(pair.client)
                .empty());
        }

        // simulated delay, to allow testing concurrency
        TimeoutFuture::new(1_000).await;

        *self.count.borrow_mut() += 15;
        let mut storage = self.state.borrow().storage();
        let count = *self.count.borrow();
        storage.put("count", count).await?;

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

        // simulated delay, to allow testing concurrency
        TimeoutFuture::new(1_000).await;

        // get and increment storage by 15
        let mut storage = self.state.borrow().storage();
        let mut count = storage.get("count").await.unwrap_or(0);
        count += 15;
        storage.put("count", count).await?;
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
