use crate::ws_events::MessageEvent;
use crate::{Error, Result};
use std::convert::TryFrom;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use worker_sys::console_log;

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
    pub fn new(url: &str) -> Result<Self> {
        worker_sys::WebSocket::new(url)
            .map(|ws| ws.into())
            .map_err(|err| Error::from(err))
    }

    pub fn new_with_protocols(url: &str, protocols: &str) -> Result<Self> {
        worker_sys::WebSocket::new_with_str(url, protocols)
            .map(|ws| ws.into())
            .map_err(|err| Error::from(err))
    }

    pub fn accept(&self) -> Result<()> {
        self.socket.accept().map_err(|err| Error::from(err))
    }

    pub fn url(&self) -> String {
        self.socket.url()
    }

    pub fn ready_state(&self) -> Result<ReadyState> {
        let value = self.socket.ready_state();
        ReadyState::try_from(value)
    }

    pub fn buffered_amount(&self) -> u32 {
        self.socket.buffered_amount()
    }

    pub fn extensions(&self) -> String {
        self.socket.extensions()
    }

    pub fn protocol(&self) -> String {
        self.socket.protocol()
    }

    pub fn send_with_str(&self, data: &str) -> Result<()> {
        self.socket
            .send_with_str(data)
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

    pub fn on_message<F: Fn(MessageEvent) + 'static>(&self, fun: F) {
        let js_callback = Closure::wrap(Box::new(move |e: worker_sys::MessageEvent| fun(e.into()))
            as Box<dyn FnMut(worker_sys::MessageEvent)>);
        self.socket
            .set_onmessage(Some(js_callback.as_ref().unchecked_ref()));
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

pub enum ReadyState {
    Connecting,
    Open,
    Closing,
    Closed,
}

impl TryFrom<u16> for ReadyState {
    type Error = Error;

    fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
        match value {
            worker_sys::WebSocket::CONNECTING => Ok(Self::Connecting),
            worker_sys::WebSocket::OPEN => Ok(Self::Open),
            worker_sys::WebSocket::CLOSING => Ok(Self::Closing),
            worker_sys::WebSocket::CLOSED => Ok(Self::Closed),
            _ => Err(Error::from("Invalid ready state")),
        }
    }
}

pub mod ws_events {
    pub struct MessageEvent {
        event: worker_sys::MessageEvent,
    }

    impl From<worker_sys::MessageEvent> for MessageEvent {
        fn from(event: worker_sys::MessageEvent) -> Self {
            Self { event }
        }
    }
}
