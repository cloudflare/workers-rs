use crate::ws_events::{CloseEvent, ErrorEvent, MessageEvent};
use crate::{Error, Result};
use serde::Serialize;
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketPair {
    pub client: WebSocket,
    pub server: WebSocket,
}

impl WebSocketPair {
    pub fn new() -> Result<Self> {
        let mut pair = worker_sys::WebSocketPair::new();
        let client = pair.client()?.into();
        let server = pair.server()?.into();
        Ok(Self { client, server })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocket {
    socket: worker_sys::WebSocket,
}

impl WebSocket {
    pub fn accept(&self) -> Result<()> {
        self.socket.accept().map_err(|err| Error::from(err))
    }

    pub fn send<T: Serialize>(&self, data: &T) -> Result<()> {
        let value = serde_json::to_string(data)?;
        self.send_with_str(value.as_str())
    }

    pub fn send_with_str<S: AsRef<str>>(&self, data: S) -> Result<()> {
        self.socket
            .send_with_str(data.as_ref())
            .map_err(|err| Error::from(err))
    }

    pub fn send_with_binary(&self, bytes: Vec<u8>) -> Result<()> {
        let array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
        array.copy_from(&bytes);
        self.socket
            .send_with_u8_array(array)
            .map_err(|err| Error::from(err))
    }

    pub fn close<S: AsRef<str>>(&self, code: Option<u16>, reason: Option<S>) -> Result<()> {
        if let Some((code, reason)) = code.zip(reason) {
            self.socket
                .close_with_code_and_reason(code, reason.as_ref())
        } else if let Some(code) = code {
            self.socket.close_with_code(code)
        } else {
            self.socket.close()
        }
        .map_err(|err| Error::from(err))
    }

    pub fn on_message<F: Fn(MessageEvent) + 'static>(&self, fun: F) -> Result<()> {
        self.add_event_listener("message", move |event: worker_sys::MessageEvent| {
            fun(event.into());
        })
    }

    pub fn on_close<F: Fn(CloseEvent) + 'static>(&self, fun: F) -> Result<()> {
        self.add_event_listener("close", move |event: worker_sys::CloseEvent| {
            fun(event.into());
        })
    }

    pub fn on_error<F: Fn(ErrorEvent) + 'static>(&self, fun: F) -> Result<()> {
        self.add_event_listener("error", move |event: worker_sys::ErrorEvent| {
            fun(event.into());
        })
    }

    fn add_event_listener<T: FromWasmAbi + 'static, F: FnMut(T) + 'static>(
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
            .map_err(|err| Error::from(err));
        // we need to leak this closure, or it will be invalidated at the end of the function.
        js_callback.forget();
        return result;
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
        pub fn get_data(&self) -> JsValue {
            self.event.data()
        }
    }

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
