use crate::{Error, Method, Request, Result};
use futures_channel::mpsc::UnboundedReceiver;
use futures_util::Stream;
use js_sys::Uint8Array;
use serde::Serialize;
use url::Url;
use worker_sys::ext::WebSocketExt;

#[cfg(not(feature = "http"))]
use crate::Fetch;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
#[cfg(feature = "http")]
use wasm_bindgen_futures::JsFuture;

pub use crate::ws_events::*;

/// Struct holding the values for a JavaScript `WebSocketPair`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketPair {
    pub client: WebSocket,
    pub server: WebSocket,
}

unsafe impl Send for WebSocketPair {}
unsafe impl Sync for WebSocketPair {}

impl WebSocketPair {
    /// Creates a new `WebSocketPair`.
    pub fn new() -> Result<Self> {
        let mut pair = worker_sys::WebSocketPair::new()?;
        let client = pair.client()?.into();
        let server = pair.server()?.into();
        Ok(Self { client, server })
    }
}

/// Wrapper struct for underlying worker-sys `WebSocket`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocket {
    socket: web_sys::WebSocket,
}

unsafe impl Send for WebSocket {}
unsafe impl Sync for WebSocket {}

impl WebSocket {
    /// Attempts to establish a [`WebSocket`] connection to the provided [`Url`].
    ///
    /// # Example:
    /// ```rust,ignore
    /// let ws = WebSocket::connect("wss://echo.zeb.workers.dev/".parse()?).await?;
    ///
    /// // It's important that we call this before we send our first message, otherwise we will
    /// // not have any event listeners on the socket to receive the echoed message.
    /// let mut event_stream = ws.events()?;
    ///
    /// ws.accept()?;
    /// ws.send_with_str("Hello, world!")?;
    ///
    /// while let Some(event) = event_stream.next().await {
    ///     let event = event?;
    ///
    ///     if let WebsocketEvent::Message(msg) = event {
    ///         if let Some(text) = msg.text() {
    ///             return Response::ok(text);
    ///         }
    ///     }
    /// }
    ///
    /// Response::error("never got a message echoed back :(", 500)
    /// ```
    pub async fn connect(url: Url) -> Result<WebSocket> {
        WebSocket::connect_with_protocols(url, None).await
    }

    /// Attempts to establish a [`WebSocket`] connection to the provided [`Url`] and protocol.
    ///
    /// # Example:
    /// ```rust,ignore
    /// let ws = WebSocket::connect_with_protocols("wss://echo.zeb.workers.dev/".parse()?, Some(vec!["GiggleBytes"])).await?;
    ///
    /// ```
    pub async fn connect_with_protocols(
        mut url: Url,
        protocols: Option<Vec<&str>>,
    ) -> Result<WebSocket> {
        let scheme: String = match url.scheme() {
            "ws" => "http".into(),
            "wss" => "https".into(),
            scheme => scheme.into(),
        };

        // With fetch we can only make requests to http(s) urls, but Workers will allow us to upgrade
        // those connections into websockets if we use the `Upgrade` header.
        url.set_scheme(&scheme).unwrap();

        let mut req = Request::new(url.as_str(), Method::Get)?;
        req.headers_mut()?.set("upgrade", "websocket")?;

        match protocols {
            None => {}
            Some(v) => {
                req.headers_mut()?
                    .set("Sec-WebSocket-Protocol", v.join(",").as_str())?;
            }
        }

        #[cfg(not(feature = "http"))]
        let res = Fetch::Request(req).send().await?;
        #[cfg(feature = "http")]
        let res: crate::Response = fetch_with_request_raw(req).await?.into();

        match res.websocket() {
            Some(ws) => Ok(ws),
            None => Err(Error::RustError("server did not accept".into())),
        }
    }

    /// Accepts the connection, allowing for messages to be sent to and from the `WebSocket`.
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
    pub fn send_with_bytes<D: AsRef<[u8]>>(&self, bytes: D) -> Result<()> {
        // This clone to Uint8Array must happen, because workerd
        // will not clone the supplied buffer and will send it asynchronously.
        // Rust believes that the lifetime ends when `send` returns, and frees
        // the memory, causing corruption.
        let uint8_array = Uint8Array::from(bytes.as_ref());
        self.socket.send_with_array_buffer(&uint8_array.buffer())?;
        Ok(())
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

    /// Internal utility method to avoid verbose code.
    /// This method registers a closure in the underlying JS environment, which calls back into the
    /// Rust/wasm environment.
    ///
    /// Since this is a 'long living closure', we need to keep the lifetime of this closure until
    /// we remove any references to the closure. So the caller of this must not drop the closure
    /// until they call [`Self::remove_event_handler`].
    fn add_event_handler<T: FromWasmAbi + 'static, F: FnMut(T) + 'static>(
        &self,
        r#type: &str,
        fun: F,
    ) -> Result<Closure<dyn FnMut(T)>> {
        let js_callback = Closure::wrap(Box::new(fun) as Box<dyn FnMut(T)>);
        self.socket
            .add_event_listener_with_callback(r#type, js_callback.as_ref().unchecked_ref())
            .map_err(Error::from)?;

        Ok(js_callback)
    }

    /// Internal utility method to avoid verbose code.
    /// This method registers a closure in the underlying JS environment, which calls back
    /// into the Rust/wasm environment.
    fn remove_event_handler<T: FromWasmAbi + 'static>(
        &self,
        r#type: &str,
        js_callback: Closure<dyn FnMut(T)>,
    ) -> Result<()> {
        self.socket
            .remove_event_listener_with_callback(r#type, js_callback.as_ref().unchecked_ref())
            .map_err(Error::from)
    }

    /// Gets an implementation [`Stream`](futures::Stream) that yields events from the inner
    /// WebSocket.
    pub fn events(&self) -> Result<EventStream<&Self>> {
        EventStream::new(self)
    }
}

impl AsRef<WebSocket> for WebSocket {
    fn as_ref(&self) -> &WebSocket {
        self
    }

    pub fn serialize_attachment<T: Serialize>(&self, value: T) -> Result<()> {
        self.socket
            .serialize_attachment(serde_wasm_bindgen::to_value(&value)?)
            .map_err(Error::from)
    }

    pub fn deserialize_attachment<T: serde::de::DeserializeOwned>(&self) -> Result<Option<T>> {
        let value = self.socket.deserialize_attachment().map_err(Error::from)?;

        if value.is_null() || value.is_undefined() {
            return Ok(None);
        }

        serde_wasm_bindgen::from_value::<T>(value)
            .map(Some)
            .map_err(Error::from)
    }
}

type EvCallback<T> = Closure<dyn FnMut(T)>;

/// A [`Stream`](futures::Stream) that yields [`WebsocketEvent`](crate::ws_events::WebsocketEvent)s
/// emitted by the inner [`WebSocket`](crate::WebSocket). The stream is guaranteed to always yield a
/// `WebsocketEvent::Close` as the final non-none item.
///
/// # Example
/// ```rust,ignore
/// use futures::StreamExt;
///
/// let pair = WebSocketPair::new()?;
/// let server = pair.server;
///
/// server.accept()?;
///
/// // Spawn a future for handling the stream of events from the websocket.
/// wasm_bindgen_futures::spawn_local(async move {
///     let mut event_stream = server.events().expect("could not open stream");
///
///     while let Some(event) = event_stream.next().await {
///         match event.expect("received error in websocket") {
///             WebsocketEvent::Message(msg) => console_log!("{:#?}", msg),
///             WebsocketEvent::Close(event) => console_log!("Closed!"),
///         }
///     }
/// });
/// ```
#[pin_project::pin_project(PinnedDrop)]
pub struct EventStream<T: AsRef<WebSocket>> {
    ws: T,
    #[pin]
    rx: UnboundedReceiver<Result<WebsocketEvent>>,
    closed: bool,
    /// Once we have decided we need to finish the stream, we need to remove any listeners we
    /// registered with the websocket.
    closures: Option<(
        EvCallback<web_sys::MessageEvent>,
        EvCallback<web_sys::ErrorEvent>,
        EvCallback<web_sys::CloseEvent>,
    )>,
}

impl<T: AsRef<WebSocket>> EventStream<T> {
    /// Create EventStream from Websocket. The input can be [`WebSocket`](self::WebSocket),
    /// [`&WebSocket`](self::WebSocket), [`Arc<WebSocket>`](self::WebSocket), etc.
    pub fn new(ws: T) -> Result<EventStream<T>> {
        let ws_ref = ws.as_ref();
        let (tx, rx) = futures_channel::mpsc::unbounded::<Result<WebsocketEvent>>();
        let tx = Rc::new(tx);

        let close_closure = ws_ref.add_event_handler("close", {
            let tx = tx.clone();
            move |event: web_sys::CloseEvent| {
                tx.unbounded_send(Ok(WebsocketEvent::Close(event.into())))
                    .unwrap();
            }
        })?;
        let message_closure = ws_ref.add_event_handler("message", {
            let tx = tx.clone();
            move |event: web_sys::MessageEvent| {
                tx.unbounded_send(Ok(WebsocketEvent::Message(event.into())))
                    .unwrap();
            }
        })?;
        let error_closure =
            ws_ref.add_event_handler("error", move |event: web_sys::ErrorEvent| {
                let error = event.error();
                tx.unbounded_send(Err(error.into())).unwrap();
            })?;

        Ok(EventStream {
            ws,
            rx,
            closed: false,
            closures: Some((message_closure, error_closure, close_closure)),
        })
    }

    pub fn socket(&self) -> &WebSocket {
        self.ws.as_ref()
    }
}

impl<T: AsRef<WebSocket>> Stream for EventStream<T> {
    type Item = Result<WebsocketEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if *this.closed {
            return Poll::Ready(None);
        }

        // Poll the inner receiver to check if theres any events from our event callbacks.
        let item = futures_util::ready!(this.rx.poll_next(cx));

        // Mark the stream as closed if we get a close event and yield None next iteration.
        if let Some(item) = &item {
            if matches!(&item, Ok(WebsocketEvent::Close(_))) {
                *this.closed = true;
            }
        }

        Poll::Ready(item)
    }
}

// Because we don't want to receive messages once our stream is done we need to remove all our
// listeners when we go to drop the stream.
#[pin_project::pinned_drop]
impl<T: AsRef<WebSocket>> PinnedDrop for EventStream<T> {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();

        // remove_event_handler takes an owned closure, so we'll do this little hack and wrap our
        // closures in an Option. This should never panic because we should never call drop twice.
        let (message_closure, error_closure, close_closure) =
            std::mem::take(this.closures).expect("double drop on worker::EventStream");

        let ws = this.ws.as_ref();

        ws.remove_event_handler("message", message_closure)
            .expect("could not remove message handler");
        ws.remove_event_handler("error", error_closure)
            .expect("could not remove error handler");
        ws.remove_event_handler("close", close_closure)
            .expect("could not remove close handler");
    }
}

impl From<web_sys::WebSocket> for WebSocket {
    fn from(socket: web_sys::WebSocket) -> Self {
        Self { socket }
    }
}

impl AsRef<web_sys::WebSocket> for WebSocket {
    fn as_ref(&self) -> &web_sys::WebSocket {
        &self.socket
    }
}

pub mod ws_events {
    use serde::de::DeserializeOwned;
    use wasm_bindgen::JsValue;

    use crate::Error;

    /// Events that can be yielded by a [`EventStream`](crate::EventStream).
    #[derive(Debug, Clone)]
    pub enum WebsocketEvent {
        Message(MessageEvent),
        Close(CloseEvent),
    }

    /// Wrapper/Utility struct for the `web_sys::MessageEvent`
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct MessageEvent {
        event: web_sys::MessageEvent,
    }

    impl From<web_sys::MessageEvent> for MessageEvent {
        fn from(event: web_sys::MessageEvent) -> Self {
            Self { event }
        }
    }

    impl AsRef<web_sys::MessageEvent> for MessageEvent {
        fn as_ref(&self) -> &web_sys::MessageEvent {
            &self.event
        }
    }

    impl MessageEvent {
        /// Gets the data/payload from the message.
        fn data(&self) -> JsValue {
            self.event.data()
        }

        pub fn text(&self) -> Option<String> {
            let value = self.data();
            value.as_string()
        }

        pub fn bytes(&self) -> Option<Vec<u8>> {
            let value = self.data();
            if value.is_object() {
                Some(js_sys::Uint8Array::new(&value).to_vec())
            } else {
                None
            }
        }

        pub fn json<T: DeserializeOwned>(&self) -> crate::Result<T> {
            let text = match self.text() {
                Some(text) => text,
                None => return Err(Error::from("data of message event is not text")),
            };

            serde_json::from_str(&text).map_err(Error::from)
        }
    }

    /// Wrapper/Utility struct for the `web_sys::CloseEvent`
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CloseEvent {
        event: web_sys::CloseEvent,
    }

    impl CloseEvent {
        pub fn reason(&self) -> String {
            self.event.reason()
        }

        pub fn code(&self) -> u16 {
            self.event.code()
        }

        pub fn was_clean(&self) -> bool {
            self.event.was_clean()
        }
    }

    impl From<web_sys::CloseEvent> for CloseEvent {
        fn from(event: web_sys::CloseEvent) -> Self {
            Self { event }
        }
    }

    impl AsRef<web_sys::CloseEvent> for CloseEvent {
        fn as_ref(&self) -> &web_sys::CloseEvent {
            &self.event
        }
    }
}

/// TODO: Convert WebSocket to use `http` types and `reqwest`.
#[cfg(feature = "http")]
async fn fetch_with_request_raw(request: crate::Request) -> Result<web_sys::Response> {
    let req = request.inner();
    let fut = {
        let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
        crate::send::SendFuture::new(JsFuture::from(worker.fetch_with_request(req)))
    };
    let resp = fut.await?;
    Ok(resp.dyn_into()?)
}
