//! Rust-side ergonomics the ported suite needs that the generated bindings do
//! not provide. Each helper mirrors a `worker` API and is a parity gap; see
//! `parity/test-port.md`.

use js_sys::{Function, Null, Promise, Uint8Array};
use serde::Serialize;
use ts_gen_worker::*;
use url::Url;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// `worker::Request::url()`.
pub fn url(req: &Request) -> Result<Url> {
    Ok(Url::parse(&req.url()).map_err(|e| e.to_string())?)
}

/// `worker::Request::path()`.
pub fn path(req: &Request) -> Result<String> {
    Ok(url(req)?.path().to_string())
}

/// `worker::Response::ok`.
pub fn ok(body: impl AsRef<str>) -> Result<Response> {
    Ok(Response::new_with_str(body.as_ref())?)
}

/// `worker::Response::error`.
pub fn error(msg: impl AsRef<str>, status: u16) -> Result<Response> {
    Ok(Response::new_with_str_and_init(
        msg.as_ref(),
        &ResponseInit::builder().status(f64::from(status)).build(),
    )?)
}

/// `worker::Response::from_json`.
pub fn from_json<T: Serialize + ?Sized>(value: &T) -> Result<Response> {
    let headers = Headers::new()?;
    headers.set("content-type", "application/json")?;
    Ok(Response::new_with_str_and_init(
        &serde_json::to_string(value).map_err(|e| e.to_string())?,
        &ResponseInit::builder().headers(&headers).build(),
    )?)
}

/// `worker::Response::from_bytes`.
pub fn from_bytes(bytes: &[u8]) -> Result<Response> {
    Ok(Response::new_with_typed_array(&Uint8Array::from(bytes))?)
}

/// `worker::Delay`.
pub async fn sleep(ms: f64) {
    let promise = Promise::new(&mut |resolve: Function, _| {
        let _ = set_timeout_with_ms_delay(resolve.unchecked_ref(), ms);
    });
    let _ = JsFuture::from(promise).await;
}

/// `worker::Request::text()` as an owned `String`.
pub async fn text(req: &Request) -> Result<String> {
    Ok(req.text().await?.into())
}

/// `worker::Response::text()` as an owned `String`.
pub async fn response_text(res: &Response) -> Result<String> {
    Ok(res.text().await?.into())
}

/// `worker::Response::from_websocket`.
pub fn from_websocket(websocket: &WebSocket) -> Result<Response> {
    Ok(Response::new_with_null_and_init(
        &Null::NULL,
        &ResponseInit::builder()
            .status(101.0)
            .web_socket(websocket)
            .build(),
    )?)
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
        Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            (handler.borrow_mut())(WebsocketEvent::Message(event.unchecked_into()));
        })
    };
    let on_close = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
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
pub fn message_text(event: &MessageEvent) -> Option<String> {
    match event.data() {
        DataKind::String(s) => Some(s),
        DataKind::ArrayBuffer(_) => None,
    }
}

/// `worker::Storage::get`: `undefined` is `None`, anything else deserializes.
pub async fn storage_get<T: serde::de::DeserializeOwned>(
    storage: &DurableObjectStorage,
    key: &str,
) -> Result<Option<T>> {
    let value = storage.get(key).await?;
    if value.is_undefined() {
        Ok(None)
    } else {
        Ok(Some(
            serde_wasm_bindgen::from_value(value).map_err(|e| e.to_string())?,
        ))
    }
}

/// `worker::Storage::put`.
pub async fn storage_put<T: Serialize>(
    storage: &DurableObjectStorage,
    key: &str,
    value: T,
) -> Result<()> {
    storage
        .put(
            key,
            &serde_wasm_bindgen::to_value(&value).map_err(|e| e.to_string())?,
        )
        .await?;
    Ok(())
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
    let gated = state.block_concurrency_while_promise(&Function::from_closure(callback));
    async move {
        JsFuture::from(gated?).await?;
        let value = output.borrow_mut().take();
        value.ok_or_else(|| "block_concurrency_while callback did not run".into())
    }
}
