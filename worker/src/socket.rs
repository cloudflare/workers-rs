use std::task::Poll;

use crate::r2::js_object;
use crate::Result;
use futures_util::FutureExt;
use js_sys::{Boolean as JsBoolean, JsString, Number as JsNumber, Object as JsObject, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// Represents an outbound TCP connection from your Worker.
pub struct Socket {
    inner: worker_sys::Socket,
    _writable: worker_sys::WritableStream,
    writer: worker_sys::WritableStreamDefaultWriter,
    _readable: worker_sys::ReadableStream,
    reader: worker_sys::ReadableStreamDefaultReader,
    // This seems weird, but we don't want to keep writing on each poll,
    // Not sure how this would handle two writes without awaiting first.
    write_fut: Option<JsFuture>,
    read_fut: Option<JsFuture>,
    close_fut: Option<JsFuture>,
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
            write_fut: None,
            read_fut: None,
            close_fut: None,
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
        let mut fut = match self.read_fut.take() {
            Some(fut) => fut,
            None => JsFuture::from(self.reader.read()),
        };

        match fut.poll_unpin(cx) {
            Poll::Pending => {
                self.read_fut = Some(fut);
                Poll::Pending
            }
            Poll::Ready(res) => Poll::Ready(
                res.map(|value| {
                    let data = Uint8Array::from(value);
                    buf.put_slice(&data.to_vec());
                })
                .map_err(js_value_to_std_io_error),
            ),
        }
    }
}

impl tokio::io::AsyncWrite for Socket {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::result::Result<usize, std::io::Error>> {
        let mut fut = match self.write_fut.take() {
            Some(fut) => fut,
            None => {
                let obj = JsValue::from(Uint8Array::from(buf));
                JsFuture::from(self.writer.write(obj))
            }
        };

        match fut.poll_unpin(cx) {
            Poll::Pending => {
                self.write_fut = Some(fut);
                Poll::Pending
            }
            Poll::Ready(res) => {
                Poll::Ready(res.map(|_| buf.len()).map_err(js_value_to_std_io_error))
            }
        }
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
        let mut fut = match self.close_fut.take() {
            Some(fut) => fut,
            None => JsFuture::from(self.writer.close()),
        };
        match fut.poll_unpin(cx) {
            Poll::Pending => {
                self.close_fut = Some(fut);
                Poll::Pending
            }
            Poll::Ready(res) => Poll::Ready(res.map(|_| ()).map_err(js_value_to_std_io_error)),
        }
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
