//! Rust-side ergonomics the ported suite needs that the generated bindings do
//! not provide. Each helper mirrors a `worker` API.

use std::time::Duration;

use js_sys::Function;
use ts_gen_worker::*;
use wasm_bindgen::prelude::*;

pub async fn sleep(ms: f64) {
    Delay::from(Duration::from_millis(ms as u64)).await;
}

/// `worker::WebsocketEvent`: `WebSocket::events()` is a stream over these.
#[allow(dead_code)]
pub enum WebsocketEvent {
    Message(MessageEvent),
    Close(CloseEvent),
}

/// `worker::WebSocket::events()`. Listeners stay attached for the socket's
/// lifetime, as the stream's do.
pub fn on_websocket_events(
    ws: &WebSocket,
    mut handler: impl FnMut(WebsocketEvent) + 'static,
) -> Result<()> {
    let handler = std::rc::Rc::new(std::cell::RefCell::new(move |event: WebsocketEvent| {
        handler(event)
    }));
    let on_message = {
        let handler = handler.clone();
        Function::from_fn(move |event: MessageEvent| {
            (handler.borrow_mut())(WebsocketEvent::Message(event));
        })
    };
    let on_close = Function::from_fn(move |event: CloseEvent| {
        (handler.borrow_mut())(WebsocketEvent::Close(event));
    });
    ws.add_event_listener_message(&on_message)?;
    ws.add_event_listener_close(&on_close)?;
    Ok(())
}

/// `worker::WebSocket::events()` as a channel the caller can `await` on.
pub fn websocket_events(
    ws: &WebSocket,
) -> Result<futures_channel::mpsc::UnboundedReceiver<WebsocketEvent>> {
    let (tx, rx) = futures_channel::mpsc::unbounded();
    on_websocket_events(ws, move |event| {
        let _ = tx.unbounded_send(event);
    })?;
    Ok(rx)
}

/// `MessageEvent::data` as text, mirroring `worker::MessageEvent::text()`.
pub fn message_text(event: &MessageEvent) -> Option<String> {
    match event.data() {
        DataKind::String(text) => Some(text),
        _ => None,
    }
}

/// `worker::State::block_concurrency_while`: runs a Rust future inside the
/// input gate and hands its output back out. The binding returns its `Promise`
/// synchronously, so the gate is entered before this returns and a Durable
/// Object constructor can let the returned future run in the background.
pub fn block_concurrency_while<F, T>(
    state: &DurableObjectState,
    future: F,
) -> impl std::future::Future<Output = Result<T>>
where
    F: std::future::Future<Output = Result<T>> + 'static,
    T: 'static,
{
    use std::cell::RefCell;
    use std::rc::Rc;
    let output: Rc<RefCell<Option<T>>> = Rc::new(RefCell::new(None));
    let slot = output.clone();
    let callback = Function::once(move || {
        wasm_bindgen_futures::future_to_promise(async move {
            let value = future.await?;
            *slot.borrow_mut() = Some(value);
            Ok(JsValue::NULL)
        })
    });
    let gated = state.block_concurrency_while(&callback);
    async move {
        gated.await?;
        let value = output.borrow_mut().take();
        value.ok_or_else(|| "block_concurrency_while callback did not run".into())
    }
}
