use std::task::Poll;

use crate::r2::js_object;
use crate::Result;
use futures_util::FutureExt;
use js_sys::{Boolean as JsBoolean, JsString, Number as JsNumber, Object as JsObject, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::console_log;

enum Reading {
    None,
    Pending(JsFuture),
    Ready(Vec<u8>),
}

impl Default for Reading {
    fn default() -> Self {
        Self::None
    }
}

enum Writing {
    Pending(JsFuture, usize),
    None,
}

impl Default for Writing {
    fn default() -> Self {
        Self::None
    }
}

enum Closing {
    Pending(JsFuture),
    None,
}

impl Default for Closing {
    fn default() -> Self {
        Self::None
    }
}

/// Represents an outbound TCP connection from your Worker.
pub struct Socket {
    inner: worker_sys::Socket,
    // TODO: Box this stuff so it can be sync / send?
    _writable: worker_sys::WritableStream,
    writer: worker_sys::WritableStreamDefaultWriter,
    _readable: worker_sys::ReadableStream,
    reader: worker_sys::ReadableStreamDefaultReader,
    write: Option<Writing>,
    read: Option<Reading>,
    close: Option<Closing>,
}

impl Socket {
    fn new(inner: worker_sys::Socket) -> Self {
        let _writable = inner.writable();
        let writer = _writable.get_writer();
        let _readable = inner.readable();
        let reader = _readable.get_reader();
        Socket {
            inner,
            _writable,
            writer,
            _readable,
            reader,
            read: None,
            write: None,
            close: None,
        }
    }

    /// Closes the TCP socket. Both the readable and writable streams are forcibly closed.
    pub async fn close(&mut self) -> Result<()> {
        JsFuture::from(self.inner.close()).await?;
        Ok(())
    }

    /// This Future is resolved when the socket is closed
    /// and is rejected if the socket encounters an error.
    pub async fn closed(&self) -> Result<()> {
        JsFuture::from(self.inner.closed()).await?;
        Ok(())
    }

    /// Upgrades an insecure socket to a secure one that uses TLS,
    /// returning a new Socket. Note that in order to call this method,
    /// you must set [`secure_transport`](SocketOptions::secure_transport)
    /// to [`StartTls`](SecureTransport::StartTls) when initially
    /// calling [`connect`](connect) to create the socket.
    pub async fn start_tls(self) -> Socket {
        let inner = self.inner.start_tls();
        Socket::new(inner)
    }

    pub fn builder() -> ConnectionBuilder {
        ConnectionBuilder::default()
    }
}

fn js_value_to_std_io_error(value: JsValue) -> std::io::Error {
    let s = if value.is_string() {
        value.as_string().unwrap()
    } else if let Some(value) = value.dyn_ref::<js_sys::Error>() {
        value.to_string().into()
    } else {
        format!("Error interpreting JsError: {:?}", value)
    };
    std::io::Error::new(std::io::ErrorKind::Other, s)
}
impl tokio::io::AsyncRead for Socket {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // Writes as much as possible to buf, and stores the rest in internal buffer
        fn handle_data(
            buf: &mut tokio::io::ReadBuf<'_>,
            data: Vec<u8>,
        ) -> (Reading, Poll<std::io::Result<()>>) {
            let idx = buf.remaining().min(data.len());
            console_log!("Ready {}/{} bytes", data.len(), idx);
            buf.put_slice(&data[..idx]);
            if idx == data.len() {
                console_log!("Read to end");
                (Reading::None, Poll::Ready(Ok(())))
            } else {
                console_log!("Storing {} in internal buffer", data.len() - idx);
                let text = std::str::from_utf8(&data[idx..]).unwrap();
                console_log!("Text: {}", &text);
                (Reading::Ready(data[idx..].to_vec()), Poll::Ready(Ok(())))
            }
        }

        fn handle_future(
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
            mut fut: JsFuture,
        ) -> (Reading, Poll<std::io::Result<()>>) {
            match fut.poll_unpin(cx) {
                Poll::Pending => (Reading::Pending(fut), Poll::Pending),
                Poll::Ready(res) => match res {
                    Ok(value) => {
                        let arr: js_sys::Uint8Array =
                            js_sys::Reflect::get(&value, &JsValue::from("value"))
                                .unwrap()
                                .into();
                        let data = arr.to_vec();
                        handle_data(buf, data)
                    }
                    Err(e) => (Reading::None, Poll::Ready(Err(js_value_to_std_io_error(e)))),
                },
            }
        }

        let (new_reading, poll) = match self.read.take().unwrap_or_default() {
            Reading::None => handle_future(cx, buf, JsFuture::from(self.reader.read())),
            Reading::Pending(fut) => handle_future(cx, buf, fut),
            Reading::Ready(data) => handle_data(buf, data),
        };
        self.read = Some(new_reading);
        poll
    }
}

impl tokio::io::AsyncWrite for Socket {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::result::Result<usize, std::io::Error>> {
        fn handle_future(
            cx: &mut std::task::Context<'_>,
            mut fut: JsFuture,
            len: usize,
        ) -> (Writing, Poll<std::io::Result<usize>>) {
            match fut.poll_unpin(cx) {
                Poll::Pending => (Writing::Pending(fut, len), Poll::Pending),
                Poll::Ready(res) => match res {
                    Ok(_) => (Writing::None, Poll::Ready(Ok(len))),
                    Err(e) => (Writing::None, Poll::Ready(Err(js_value_to_std_io_error(e)))),
                },
            }
        }

        let (new_writing, poll) = match self.write.take().unwrap_or_default() {
            Writing::None => {
                let obj = JsValue::from(Uint8Array::from(buf));
                handle_future(cx, JsFuture::from(self.writer.write(obj)), buf.len())
            }
            Writing::Pending(fut, len) => handle_future(cx, fut, len),
        };
        self.write = Some(new_writing);
        poll
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        // TODO: I don't think we have a flush operation available?
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        fn handle_future(
            cx: &mut std::task::Context<'_>,
            mut fut: JsFuture,
        ) -> (Closing, Poll<std::io::Result<()>>) {
            match fut.poll_unpin(cx) {
                Poll::Pending => (Closing::Pending(fut), Poll::Pending),
                Poll::Ready(res) => match res {
                    Ok(_) => (Closing::None, Poll::Ready(Ok(()))),
                    Err(e) => (Closing::None, Poll::Ready(Err(js_value_to_std_io_error(e)))),
                },
            }
        }
        let (new_closing, poll) = match self.close.take().unwrap_or_default() {
            Closing::None => handle_future(cx, JsFuture::from(self.writer.close())),
            Closing::Pending(fut) => handle_future(cx, fut),
        };
        self.close = Some(new_closing);
        poll
    }
}

pub enum SecureTransport {
    /// Do not use TLS.
    Off,
    /// Use TLS.
    On,
    /// Do not use TLS initially, but allow the socket to be upgraded to
    /// use TLS by calling [`Socket.start_tls`](Socket::start_tls).
    StartTls,
}

pub struct SocketOptions {
    /// Specifies whether or not to use TLS when creating the TCP socket.
    pub secure_transport: SecureTransport,
    /// Defines whether the writable side of the TCP socket will automatically
    /// close on end-of-file (EOF). When set to false, the writable side of the
    /// TCP socket will automatically close on EOF. When set to true, the
    /// writable side of the TCP socket will remain open on EOF.
    pub allow_half_open: bool,
}

impl Default for SocketOptions {
    fn default() -> Self {
        SocketOptions {
            secure_transport: SecureTransport::Off,
            allow_half_open: false,
        }
    }
}

/// The host and port that you wish to connect to.
pub struct SocketAddress {
    /// The hostname to connect to. Example: `cloudflare.com`.
    pub hostname: String,
    /// The port number to connect to. Example: `5432`.
    pub port: u16,
}

#[derive(Default)]
pub struct ConnectionBuilder {
    hostname: Option<String>,
    port: Option<u16>,
    options: SocketOptions,
}

impl ConnectionBuilder {
    /// Create a new `ConnectionBuilder` with default settings.
    pub fn new() -> Self {
        ConnectionBuilder {
            hostname: None,
            port: None,
            options: SocketOptions::default(),
        }
    }

    /// Set the hostname to connect to. Example: `cloudflare.com`.
    pub fn host(mut self, host: &str) -> Self {
        self.hostname = Some(host.to_string());
        self
    }

    /// Set the port to connect to. Example: `5432`.
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set whether the writable side of the TCP socket will automatically
    /// close on end-of-file (EOF).
    pub fn allow_half_open(mut self, allow_half_open: bool) -> Self {
        self.options.allow_half_open = allow_half_open;
        self
    }

    pub fn secure_transport(mut self, secure_transport: SecureTransport) -> Self {
        self.options.secure_transport = secure_transport;
        self
    }

    /// Open the connection, returning a [`Socket`](Socket).
    /// You must set both `hostname` and `port` before invoking this method.
    pub fn connect(self) -> Result<Socket> {
        let address: JsValue = js_object!(
            "hostname" => match self.hostname {
                Some(hostname) => JsObject::from(JsString::from(hostname)),
                None => return Err("No hostname configured!".into()),
            },
            "port" => match self.port {
                Some(port) => JsObject::from(JsNumber::from(port)),
                None => return Err("No port configured!".into()),
            }
        )
        .into();

        let options: JsValue = js_object!(
            "allowHalfOpen" => JsObject::from(JsBoolean::from(self.options.allow_half_open)),
            "secureTransport" => JsObject::from(JsString::from(match self.options.secure_transport {
                SecureTransport::On => "on",
                SecureTransport::Off => "off",
                SecureTransport::StartTls => "starttls",
            }))
        )
        .into();

        let inner = worker_sys::connect(address, options);
        Ok(Socket::new(inner))
    }
}
