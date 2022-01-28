use crate::ws_events::{CloseEvent, ErrorEvent, MessageEvent};
use crate::{Error, Result};
use serde::Serialize;
use std::future::Future;
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};

/// Struct holding the values for a JavaScript `WebSocketPair`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketPair {
    pub client: WebSocket,
    pub server: WebSocket,
}

impl WebSocketPair {
    /// Creates a new `WebSocketPair`.
    pub fn new() -> Result<Self> {
        let mut pair = worker_sys::WebSocketPair::new();
        let client = pair.client()?.into();
        let server = pair.server()?.into();
        Ok(Self { client, server })
    }
}

/// Wrapper struct for underlying worker-sys `WebSocket`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocket {
    socket: worker_sys::WebSocket,
}

impl WebSocket {
    /// Accept the server-side of a websocket connection
    pub fn accept(&self) -> Result<()> {
        self.socket.accept().map_err(Error::from)
    }

    /// Serialize data into a string using serde and send it through the `WebSocket`
    pub fn send<T: Serialize>(&self, data: &T) -> Result<()> {
        let value = serde_json::to_string(data)?;
        self.send_with_str(value.as_str())
    }

    /// Sends a raw string through the `WebSocket`
    pub fn send_with_str<S: AsRef<str>>(&self, data: S) -> Result<()> {
        self.socket
            .send_with_str(data.as_ref())
            .map_err(Error::from)
    }

    /// Sends raw binary data through the `WebSocket`.
    pub fn send_with_binary<D: AsRef<[u8]>>(&self, bytes: D) -> Result<()> {
        let slice = bytes.as_ref();
        let array = js_sys::Uint8Array::new_with_length(slice.len() as u32);
        array.copy_from(slice);
        self.socket.send_with_u8_array(array).map_err(Error::from)
    }

    /// Closes this channel.
    /// This method translates to three different underlying method calls based of the
    /// parameters passed.
    ///
    /// If the following parameters are Some:
    /// * `code` and `reason` -> `close_with_code_and_reason`
    /// * `code`              -> `close_with_code`
    /// * `reason` or `none`  -> `close`
    ///
    /// Effectively, if only `reason` is `Some`, the `reason` argument will be ignored.
    pub fn close<S: AsRef<str>>(&self, code: Option<u16>, reason: Option<S>) -> Result<()> {
        if let Some((code, reason)) = code.zip(reason) {
            self.socket
                .close_with_code_and_reason(code, reason.as_ref())
        } else if let Some(code) = code {
            self.socket.close_with_code(code)
        } else {
            self.socket.close()
        }
        .map_err(Error::from)
    }

    /// Registers an event-handler for the `message` event.
    ///
    /// This event will be fired when the `WebSocket` receives a payload.
    ///
    /// https://developers.cloudflare.com/workers/runtime-apis/websockets#events     
    pub fn on_message<F: Fn(MessageEvent) + 'static>(&self, fun: F) -> Result<()> {
        self.add_event_handler("message", move |event: worker_sys::MessageEvent| {
            fun(event.into());
        })
    }

    /// Registers an async event-handler for the `message` event.
    ///
    /// This event will be fired when the `WebSocket` receives a payload.
    ///
    /// https://developers.cloudflare.com/workers/runtime-apis/websockets#events    
    pub fn on_message_async<
        Fut: Future<Output = ()> + 'static,
        F: Fn(MessageEvent) -> Fut + 'static,
    >(
        &self,
        fun: F,
    ) -> Result<()> {
        self.add_event_handler("message", move |event: worker_sys::MessageEvent| {
            let fut = fun(event.into());
            wasm_bindgen_futures::spawn_local(async move { fut.await });
        })
    }

    /// Registers an event-handler for the `close` event.
    ///
    /// This event will be fired when the `WebSocket` closes.
    ///
    /// https://developers.cloudflare.com/workers/runtime-apis/websockets#events    
    pub fn on_close<F: Fn(CloseEvent) + 'static>(&self, fun: F) -> Result<()> {
        self.add_event_handler("close", move |event: worker_sys::CloseEvent| {
            fun(event.into());
        })
    }

    /// Registers an async event-handler for the `close` event.
    ///
    /// This event will be fired when the `WebSocket` closes.
    ///
    /// https://developers.cloudflare.com/workers/runtime-apis/websockets#events
    pub fn on_close_async<
        Fut: Future<Output = ()> + 'static,
        F: Fn(CloseEvent) -> Fut + 'static,
    >(
        &self,
        fun: F,
    ) -> Result<()> {
        self.add_event_handler("close", move |event: worker_sys::CloseEvent| {
            let fut = fun(event.into());
            wasm_bindgen_futures::spawn_local(async move { fut.await });
        })
    }

    /// Registers an event-handler for the `error` event.
    ///
    /// This event will be fired when the `WebSocket` caught an error.
    ///
    /// https://developers.cloudflare.com/workers/runtime-apis/websockets#events
    pub fn on_error<F: Fn(ErrorEvent) + 'static>(&self, fun: F) -> Result<()> {
        self.add_event_handler("error", move |event: worker_sys::ErrorEvent| {
            fun(event.into());
        })
    }

    /// Registers an async event-handler for the `error` event.
    ///
    /// This event will be fired when the `WebSocket` caught an error.
    ///
    /// https://developers.cloudflare.com/workers/runtime-apis/websockets#events
    pub fn on_error_async<
        Fut: Future<Output = ()> + 'static,
        F: Fn(ErrorEvent) -> Fut + 'static,
    >(
        &self,
        fun: F,
    ) -> Result<()> {
        self.add_event_handler("error", move |event: worker_sys::ErrorEvent| {
            let fut = fun(event.into());
            wasm_bindgen_futures::spawn_local(async move { fut.await });
        })
    }

    /// Internal utility method to avoid verbose code.
    /// This method registers a closure in the underlying JS environment, which calls back
    /// into the Rust/wasm environment.
    /// Since this is a 'long living closure', we need to actively "leak" memory in this,
    /// as we need to drop the reference to the `WasmClosure` or it will be destroyed with
    /// it's lifetime at the end of the function.
    fn add_event_handler<T: FromWasmAbi + 'static, F: FnMut(T) + 'static>(
        &self,
        r#type: &str,
        fun: F,
    ) -> Result<()> {
        let js_callback = Closure::wrap(Box::new(fun) as Box<dyn FnMut(T)>);
        let result = self
            .socket
            .add_event_listener(
                JsValue::from_str(r#type),
                Some(js_callback.as_ref().unchecked_ref()),
            )
            .map_err(Error::from);
        // we need to leak this closure, or it will be invalidated at the end of the function.
        js_callback.forget();
        result
    }
}

impl From<worker_sys::WebSocket> for WebSocket {
    fn from(socket: worker_sys::WebSocket) -> Self {
        Self { socket }
    }
}

impl AsRef<worker_sys::WebSocket> for WebSocket {
    fn as_ref(&self) -> &worker_sys::WebSocket {
        &self.socket
    }
}

pub mod ws_events {
    use wasm_bindgen::JsValue;

    /// Wrapper/Utility struct for the `web_sys::MessageEvent`
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct MessageEvent {
        event: worker_sys::MessageEvent,
    }

    impl From<worker_sys::MessageEvent> for MessageEvent {
        fn from(event: worker_sys::MessageEvent) -> Self {
            Self { event }
        }
    }

    impl AsRef<worker_sys::MessageEvent> for MessageEvent {
        fn as_ref(&self) -> &worker_sys::MessageEvent {
            &self.event
        }
    }

    impl MessageEvent {
        /// Gets the data/payload from the message.
        pub fn get_data(&self) -> JsValue {
            self.event.data()
        }
    }

    /// Wrapper/Utility struct for the `web_sys::CloseEvent`
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CloseEvent {
        event: worker_sys::CloseEvent,
    }

    impl From<worker_sys::CloseEvent> for CloseEvent {
        fn from(event: worker_sys::CloseEvent) -> Self {
            Self { event }
        }
    }

    impl AsRef<worker_sys::CloseEvent> for CloseEvent {
        fn as_ref(&self) -> &worker_sys::CloseEvent {
            &self.event
        }
    }

    /// Wrapper/Utility struct for the `web_sys::ErrorEvent`
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ErrorEvent {
        event: worker_sys::ErrorEvent,
    }

    impl From<worker_sys::ErrorEvent> for ErrorEvent {
        fn from(event: worker_sys::ErrorEvent) -> Self {
            Self { event }
        }
    }

    impl AsRef<worker_sys::ErrorEvent> for ErrorEvent {
        fn as_ref(&self) -> &worker_sys::ErrorEvent {
            &self.event
        }
    }
}
