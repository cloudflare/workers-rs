use std::task::Poll;

use crate::r2::js_object;
use crate::Result;
use futures_util::FutureExt;
use js_sys::{Boolean as JsBoolean, JsString, Number as JsNumber, Object as JsObject, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

enum Reading {
    None,
    Pending(JsFuture, web_sys::ReadableStreamDefaultReader),
    Ready(Vec<u8>),
}

impl Default for Reading {
    fn default() -> Self {
        Self::None
    }
}

enum Writing {
    Pending(JsFuture, web_sys::WritableStreamDefaultWriter, usize),
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
    writable: web_sys::WritableStream,
    readable: web_sys::ReadableStream,
    write: Option<Writing>,
    read: Option<Reading>,
    close: Option<Closing>,
}

// This can only be done because workers are single threaded.
unsafe impl Send for Socket {}
unsafe impl Sync for Socket {}

impl Socket {
    fn new(inner: worker_sys::Socket) -> Self {
        let writable = inner.writable();
        let readable = inner.readable();
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
    pub fn start_tls(self) -> Socket {
        let inner = self.inner.start_tls();
        Socket::new(inner)
    }

    pub fn builder() -> ConnectionBuilder {
        ConnectionBuilder::default()
    }

    fn handle_write_future(
        cx: &mut std::task::Context<'_>,
        mut fut: JsFuture,
        writer: web_sys::WritableStreamDefaultWriter,
        len: usize,
    ) -> (Writing, Poll<std::io::Result<usize>>) {
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
        fn handle_future(
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
            mut fut: JsFuture,
            reader: web_sys::ReadableStreamDefaultReader,
        ) -> (Reading, Poll<std::io::Result<()>>) {
            match fut.poll_unpin(cx) {
                Poll::Pending => (Reading::Pending(fut, reader), Poll::Pending),
                Poll::Ready(res) => match res {
                    Ok(value) => {
                        reader.release_lock();
                        let done: js_sys::Boolean = match js_sys::Reflect::get(
                            &value,
                            &JsValue::from("done"),
                        ) {
                            Ok(value) => value.into(),
                            Err(error) => {
                                let msg = format!("Unable to interpret field 'done' in ReadableStreamDefaultReader.read(): {:?}", error);
                                return (
                                    Reading::None,
                                    Poll::Ready(Err(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        msg,
                                    ))),
                                );
                            }
                        };
                        if done.is_truthy() {
                            (Reading::None, Poll::Ready(Ok(())))
                        } else {
                            let arr: js_sys::Uint8Array = match js_sys::Reflect::get(
                                &value,
                                &JsValue::from("value"),
                            ) {
                                Ok(value) => value.into(),
                                Err(error) => {
                                    let msg = format!("Unable to interpret field 'value' in ReadableStreamDefaultReader.read(): {:?}", error);
                                    return (
                                        Reading::None,
                                        Poll::Ready(Err(std::io::Error::new(
                                            std::io::ErrorKind::Other,
                                            msg,
                                        ))),
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
                let reader: web_sys::ReadableStreamDefaultReader =
                    match self.readable.get_reader().dyn_into() {
                        Ok(reader) => reader,
                        Err(error) => {
                            let msg = format!(
                                "Unable to cast JsObject to ReadableStreamDefaultReader: {:?}",
                                error
                            );
                            return Poll::Ready(Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                msg,
                            )));
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

impl tokio::io::AsyncWrite for Socket {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::result::Result<usize, std::io::Error>> {
        let (new_writing, poll) = match self.write.take().unwrap_or_default() {
            Writing::None => {
                let obj = JsValue::from(Uint8Array::from(buf));
                let writer: web_sys::WritableStreamDefaultWriter = match self.writable.get_writer()
                {
                    Ok(writer) => writer,
                    Err(error) => {
                        let msg = format!("Could not retrieve Writer: {:?}", error);
                        return Poll::Ready(Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            msg,
                        )));
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

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
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

        let inner = worker_sys::connect(address, options);
        Ok(Socket::new(inner))
    }
}

// Writes as much as possible to buf, and stores the rest in internal buffer
fn handle_data(
    buf: &mut tokio::io::ReadBuf<'_>,
    mut data: Vec<u8>,
) -> (Reading, Poll<std::io::Result<()>>) {
    let idx = buf.remaining().min(data.len());
    let store = data.split_off(idx);
    buf.put_slice(&data);
    if store.is_empty() {
        (Reading::None, Poll::Ready(Ok(())))
    } else {
        (Reading::Ready(store), Poll::Ready(Ok(())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_handle_data() {
        let mut arr = vec![0u8; 32];
        let mut buf = tokio::io::ReadBuf::new(&mut arr);
        let data = vec![1u8; 32];
        let (reading, _) = handle_data(&mut buf, data);

        assert!(matches!(reading, Reading::None));
        assert_eq!(buf.remaining(), 0);
        assert_eq!(buf.filled().len(), 32);
    }

    #[test]
    fn test_handle_large_data() {
        let mut arr = vec![0u8; 32];
        let mut buf = tokio::io::ReadBuf::new(&mut arr);
        let data = vec![1u8; 64];
        let (reading, _) = handle_data(&mut buf, data);

        assert!(matches!(reading, Reading::Ready(store) if store.len() == 32));
        assert_eq!(buf.remaining(), 0);
        assert_eq!(buf.filled().len(), 32);
    }

    #[test]
    fn test_handle_small_data() {
        let mut arr = vec![0u8; 32];
        let mut buf = tokio::io::ReadBuf::new(&mut arr);
        let data = vec![1u8; 16];
        let (reading, _) = handle_data(&mut buf, data);

        assert!(matches!(reading, Reading::None));
        assert_eq!(buf.remaining(), 16);
        assert_eq!(buf.filled().len(), 16);
    }
}
