//! Rust-side ergonomics the ported suite needs that the generated bindings do
//! not provide. Each helper mirrors a `worker` API and is a parity gap; see
//! `parity/test-port.md`.

use js_sys::{Function, Promise};
use ts_gen_worker::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// `worker::Delay`.
pub async fn sleep(ms: f64) {
    let promise = Promise::<JsValue>::new(&mut |resolve, _| {
        let _ = set_timeout_with_function_and_ms_delay(resolve.unchecked_ref(), ms);
    });
    let _ = JsFuture::from(promise).await;
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
        Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
            (handler.borrow_mut())(WebsocketEvent::Message(event.unchecked_into()));
        })
    };
    let on_close = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
        (handler.borrow_mut())(WebsocketEvent::Close(event.unchecked_into()));
    });
    ws.add_event_listener("message", &Function::from_closure(on_message))?;
    ws.add_event_listener("close", &Function::from_closure(on_close))?;
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
/// Upstream types `data` as `any`.
pub fn message_text(event: &MessageEvent) -> Option<String> {
    event.data().as_string()
}

/// `worker::State::block_concurrency_while`: runs a Rust future inside the
/// input gate and hands its output back out. The gate is entered before this
/// returns, via the sync `_promise` form, so a Durable Object constructor can
/// call it and let the returned future run in the background.
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
    let callback = Closure::once(move || {
        wasm_bindgen_futures::future_to_promise(async move {
            let value = future.await?;
            *slot.borrow_mut() = Some(value);
            Ok(JsValue::NULL)
        })
    });
    let gated = state.block_concurrency_while_promise::<JsValue>(&Function::from_closure(callback));
    async move {
        JsFuture::from(gated?).await?;
        let value = output.borrow_mut().take();
        value.ok_or_else(|| "block_concurrency_while callback did not run".into())
    }
}
