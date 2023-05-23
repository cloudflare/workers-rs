use crate::r2::js_object;
use crate::Result;
use js_sys::{Boolean as JsBoolean, JsString, Number as JsNumber, Object as JsObject};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
pub struct SocketReader {}

pub struct SocketWriter {}

/// Represents an outbound TCP connection from your Worker.
pub struct Socket {
    /// Returns the readable side of the TCP socket.
    // pub readable: SocketReader,
    /// Returns the writable side of the TCP socket.
    // pub writable: SocketWriter,
    inner: worker_sys::Socket,
}

impl Socket {
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
        Socket { inner }
    }

    pub fn builder() -> ConnectionBuilder {
        ConnectionBuilder::default()
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

        Ok(Socket { inner })
    }
}
