use std::{
    convert::TryFrom,
    io::ErrorKind,
    pin::Pin,
    task::{Context, Poll},
};

use crate::Result;
use crate::{r2::js_object, Error};
use futures_util::FutureExt;
use js_sys::{
    Boolean as JsBoolean, Error as JsError, JsString, Number as JsNumber, Object as JsObject,
    Reflect, Uint8Array,
};
use std::convert::TryInto;
use std::io::Error as IoError;
use std::io::Result as IoResult;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    ReadableStream, ReadableStreamDefaultReader, WritableStream, WritableStreamDefaultWriter,
};

#[derive(Debug)]
pub struct SocketInfo {
    pub remote_address: Option<String>,
    pub local_address: Option<String>,
}

impl TryFrom<JsValue> for SocketInfo {
    type Error = Error;
    fn try_from(value: JsValue) -> Result<Self> {
        let remote_address_value =
            js_sys::Reflect::get(&value, &JsValue::from_str("remoteAddress"))?;
        let local_address_value = js_sys::Reflect::get(&value, &JsValue::from_str("localAddress"))?;
        Ok(Self {
            remote_address: remote_address_value.as_string(),
            local_address: local_address_value.as_string(),
        })
    }
}

#[derive(Default)]
enum Reading {
    #[default]
    None,
    Pending(JsFuture, ReadableStreamDefaultReader),
    Ready(Vec<u8>),
}

#[derive(Default)]
enum Writing {
    Pending(JsFuture, WritableStreamDefaultWriter, usize),
    #[default]
    None,
}

#[derive(Default)]
enum Closing {
    Pending(JsFuture),
    #[default]
    None,
}

/// Represents an outbound TCP connection from your Worker.
pub struct Socket {
    inner: worker_sys::Socket,
    writable: WritableStream,
    readable: ReadableStream,
    write: Option<Writing>,
    read: Option<Reading>,
    close: Option<Closing>,
}

// This can only be done because workers are single threaded.
unsafe impl Send for Socket {}
unsafe impl Sync for Socket {}

impl Socket {
    fn new(inner: worker_sys::Socket) -> Self {
        let writable = inner.writable().unwrap();
        let readable = inner.readable().unwrap();
        Socket {
            inner,
            writable,
            readable,
            read: None,
            write: None,
            close: None,
        }
    }

    /// Closes the TCP socket. Both the readable and writable streams are forcibly closed.
    pub async fn close(&mut self) -> Result<()> {
        JsFuture::from(self.inner.close()?).await?;
        Ok(())
    }

    /// This Future is resolved when the socket is closed
    /// and is rejected if the socket encounters an error.
    pub async fn closed(&self) -> Result<()> {
        JsFuture::from(self.inner.closed()?).await?;
        Ok(())
    }

    pub async fn opened(&self) -> Result<SocketInfo> {
        let value = JsFuture::from(self.inner.opened()?).await?;
        value.try_into()
    }

    /// Upgrades an insecure socket to a secure one that uses TLS,
    /// returning a new Socket. Note that in order to call this method,
    /// you must set [`secure_transport`](SocketOptions::secure_transport)
    /// to [`StartTls`](SecureTransport::StartTls) when initially
    /// calling [`connect`](connect) to create the socket.
    pub fn start_tls(self) -> Socket {
        let inner = self.inner.start_tls().unwrap();
        Socket::new(inner)
    }

    pub fn builder() -> ConnectionBuilder {
        ConnectionBuilder::default()
    }

    fn handle_write_future(
        cx: &mut Context<'_>,
        mut fut: JsFuture,
        writer: WritableStreamDefaultWriter,
        len: usize,
    ) -> (Writing, Poll<IoResult<usize>>) {
        match fut.poll_unpin(cx) {
            Poll::Pending => (Writing::Pending(fut, writer, len), Poll::Pending),
            Poll::Ready(res) => {
                writer.release_lock();
                match res {
                    Ok(_) => (Writing::None, Poll::Ready(Ok(len))),
                    Err(e) => (Writing::None, Poll::Ready(Err(js_value_to_std_io_error(e)))),
                }
            }
        }
    }
}

fn js_value_to_std_io_error(value: JsValue) -> IoError {
    let s = if value.is_string() {
        value.as_string().unwrap()
    } else if let Some(value) = value.dyn_ref::<JsError>() {
        value.to_string().into()
    } else {
        format!("Error interpreting JsError: {:?}", value)
    };
    IoError::new(ErrorKind::Other, s)
}
impl AsyncRead for Socket {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        fn handle_future(
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
            mut fut: JsFuture,
            reader: ReadableStreamDefaultReader,
        ) -> (Reading, Poll<IoResult<()>>) {
            match fut.poll_unpin(cx) {
                Poll::Pending => (Reading::Pending(fut, reader), Poll::Pending),
                Poll::Ready(res) => match res {
                    Ok(value) => {
                        reader.release_lock();
                        let done: JsBoolean = match Reflect::get(&value, &JsValue::from("done")) {
                            Ok(value) => value.into(),
                            Err(error) => {
                                let msg = format!("Unable to interpret field 'done' in ReadableStreamDefaultReader.read(): {:?}", error);
                                return (
                                    Reading::None,
                                    Poll::Ready(Err(IoError::new(ErrorKind::Other, msg))),
                                );
                            }
                        };
                        if done.is_truthy() {
                            (Reading::None, Poll::Ready(Ok(())))
                        } else {
                            let arr: Uint8Array = match Reflect::get(
                                &value,
                                &JsValue::from("value"),
                            ) {
                                Ok(value) => value.into(),
                                Err(error) => {
                                    let msg = format!("Unable to interpret field 'value' in ReadableStreamDefaultReader.read(): {:?}", error);
                                    return (
                                        Reading::None,
                                        Poll::Ready(Err(IoError::new(ErrorKind::Other, msg))),
                                    );
                                }
                            };
                            let data = arr.to_vec();
                            handle_data(buf, data)
                        }
                    }
                    Err(e) => (Reading::None, Poll::Ready(Err(js_value_to_std_io_error(e)))),
                },
            }
        }

        let (new_reading, poll) = match self.read.take().unwrap_or_default() {
            Reading::None => {
                let reader: ReadableStreamDefaultReader =
                    match self.readable.get_reader().dyn_into() {
                        Ok(reader) => reader,
                        Err(error) => {
                            let msg = format!(
                                "Unable to cast JsObject to ReadableStreamDefaultReader: {:?}",
                                error
                            );
                            return Poll::Ready(Err(IoError::new(ErrorKind::Other, msg)));
                        }
                    };

                handle_future(cx, buf, JsFuture::from(reader.read()), reader)
            }
            Reading::Pending(fut, reader) => handle_future(cx, buf, fut, reader),
            Reading::Ready(data) => handle_data(buf, data),
        };
        self.read = Some(new_reading);
        poll
    }
}

impl AsyncWrite for Socket {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<IoResult<usize>> {
        let (new_writing, poll) = match self.write.take().unwrap_or_default() {
            Writing::None => {
                let obj = JsValue::from(Uint8Array::from(buf));
                let writer: WritableStreamDefaultWriter = match self.writable.get_writer() {
                    Ok(writer) => writer,
                    Err(error) => {
                        let msg = format!("Could not retrieve Writer: {:?}", error);
                        return Poll::Ready(Err(IoError::new(ErrorKind::Other, msg)));
                    }
                };
                Self::handle_write_future(
                    cx,
                    JsFuture::from(writer.write_with_chunk(&obj)),
                    writer,
                    buf.len(),
                )
            }
            Writing::Pending(fut, writer, len) => Self::handle_write_future(cx, fut, writer, len),
        };
        self.write = Some(new_writing);
        poll
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        // Poll existing write future if it exists.
        let (new_writing, poll) = match self.write.take().unwrap_or_default() {
            Writing::Pending(fut, writer, len) => {
                let (writing, poll) = Self::handle_write_future(cx, fut, writer, len);
                // Map poll output to ()
                (writing, poll.map(|res| res.map(|_| ())))
            }
            writing => (writing, Poll::Ready(Ok(()))),
        };
        self.write = Some(new_writing);
        poll
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        fn handle_future(cx: &mut Context<'_>, mut fut: JsFuture) -> (Closing, Poll<IoResult<()>>) {
            match fut.poll_unpin(cx) {
                Poll::Pending => (Closing::Pending(fut), Poll::Pending),
                Poll::Ready(res) => match res {
                    Ok(_) => (Closing::None, Poll::Ready(Ok(()))),
                    Err(e) => (Closing::None, Poll::Ready(Err(js_value_to_std_io_error(e)))),
                },
            }
        }
        let (new_closing, poll) = match self.close.take().unwrap_or_default() {
            Closing::None => handle_future(cx, JsFuture::from(self.writable.close())),
            Closing::Pending(fut) => handle_future(cx, fut),
        };
        self.close = Some(new_closing);
        poll
    }
}

/// Secure transport options for outbound TCP connections.
pub enum SecureTransport {
    /// Do not use TLS.
    Off,
    /// Use TLS.
    On,
    /// Do not use TLS initially, but allow the socket to be upgraded to
    /// use TLS by calling [`Socket.start_tls`](Socket::start_tls).
    StartTls,
}

/// Used to configure outbound TCP connections.
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
    options: SocketOptions,
}

impl ConnectionBuilder {
    /// Create a new `ConnectionBuilder` with default settings.
    pub fn new() -> Self {
        ConnectionBuilder {
            options: SocketOptions::default(),
        }
    }

    /// Set whether the writable side of the TCP socket will automatically
    /// close on end-of-file (EOF).
    pub fn allow_half_open(mut self, allow_half_open: bool) -> Self {
        self.options.allow_half_open = allow_half_open;
        self
    }

    // Specify whether or not to use TLS when creating the TCP socket.
    pub fn secure_transport(mut self, secure_transport: SecureTransport) -> Self {
        self.options.secure_transport = secure_transport;
        self
    }

    /// Open the connection to `hostname` on port `port`, returning a [`Socket`](Socket).
    pub fn connect(self, hostname: impl Into<String>, port: u16) -> Result<Socket> {
        let address: JsValue = js_object!(
            "hostname" => JsObject::from(JsString::from(hostname.into())),
            "port" => JsNumber::from(port)
        )
        .into();

        let options: JsValue = js_object!(
            "allowHalfOpen" => JsBoolean::from(self.options.allow_half_open),
            "secureTransport" => JsString::from(match self.options.secure_transport {
                SecureTransport::On => "on",
                SecureTransport::Off => "off",
                SecureTransport::StartTls => "starttls",
            })
        )
        .into();

        let inner = worker_sys::connect(address, options)?;
        Ok(Socket::new(inner))
    }
}

// Writes as much as possible to buf, and stores the rest in internal buffer
fn handle_data(buf: &mut ReadBuf<'_>, mut data: Vec<u8>) -> (Reading, Poll<IoResult<()>>) {
    let idx = buf.remaining().min(data.len());
    let store = data.split_off(idx);
    buf.put_slice(&data);
    if store.is_empty() {
        (Reading::None, Poll::Ready(Ok(())))
    } else {
        (Reading::Ready(store), Poll::Ready(Ok(())))
    }
}

#[cfg(feature = "tokio-postgres")]
/// Implements [`TlsConnect`](tokio_postgres::TlsConnect) for
/// [`Socket`](crate::Socket) to enable `tokio_postgres` connections
/// to databases using TLS.
pub mod postgres_tls {
    use super::Socket;
    use futures_util::future::{ready, Ready};
    use std::error::Error;
    use std::fmt::{self, Display, Formatter};
    use tokio_postgres::tls::{ChannelBinding, TlsConnect, TlsStream};

    /// Supply this to `connect_raw` in place of `NoTls` to specify TLS
    /// when using Workers.
    ///
    /// ```rust
    /// let config = tokio_postgres::config::Config::new();
    /// let socket = Socket::builder()
    ///     .secure_transport(SecureTransport::StartTls)
    ///     .connect("database_url", 5432)?;
    /// let _ = config.connect_raw(socket, PassthroughTls).await?;
    /// ```
    pub struct PassthroughTls;

    #[derive(Debug)]
    /// Error type for PassthroughTls.
    /// Should never be returned.
    pub struct PassthroughTlsError;

    impl Error for PassthroughTlsError {}

    impl Display for PassthroughTlsError {
        fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
            fmt.write_str("PassthroughTlsError")
        }
    }

    impl TlsConnect<Socket> for PassthroughTls {
        type Stream = Socket;
        type Error = PassthroughTlsError;
        type Future = Ready<Result<Socket, PassthroughTlsError>>;

        fn connect(self, s: Self::Stream) -> Self::Future {
            let tls = s.start_tls();
            ready(Ok(tls))
        }
    }

    impl TlsStream for Socket {
        fn channel_binding(&self) -> ChannelBinding {
            ChannelBinding::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_handle_data() {
        let mut arr = vec![0u8; 32];
        let mut buf = ReadBuf::new(&mut arr);
        let data = vec![1u8; 32];
        let (reading, _) = handle_data(&mut buf, data);

        assert!(matches!(reading, Reading::None));
        assert_eq!(buf.remaining(), 0);
        assert_eq!(buf.filled().len(), 32);
    }

    #[test]
    fn test_handle_large_data() {
        let mut arr = vec![0u8; 32];
        let mut buf = ReadBuf::new(&mut arr);
        let data = vec![1u8; 64];
        let (reading, _) = handle_data(&mut buf, data);

        assert!(matches!(reading, Reading::Ready(store) if store.len() == 32));
        assert_eq!(buf.remaining(), 0);
        assert_eq!(buf.filled().len(), 32);
    }

    #[test]
    fn test_handle_small_data() {
        let mut arr = vec![0u8; 32];
        let mut buf = ReadBuf::new(&mut arr);
        let data = vec![1u8; 16];
        let (reading, _) = handle_data(&mut buf, data);

        assert!(matches!(reading, Reading::None));
        assert_eq!(buf.remaining(), 16);
        assert_eq!(buf.filled().len(), 16);
    }
}
