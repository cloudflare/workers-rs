use std::{future::Future, pin::Pin};

use hyper::{
    client::connect::{Connected, Connection},
    Uri,
};
use worker::SecureTransport;

pub struct Socket {
    inner: worker::Socket,
}

#[derive(Clone)]
pub struct CloudflareConnector;

async fn connect_uri(uri: Uri) -> Result<Socket, String> {
    let tls = match uri.scheme_str() {
        None => false,
        Some("http") => false,
        Some("https") => true,
        Some(_) => false,
    };

    let transport = if tls {
        SecureTransport::On
    } else {
        SecureTransport::Off
    };

    let port = match uri.port_u16() {
        Some(port) => port,
        None => {
            if tls {
                443
            } else {
                80
            }
        }
    };

    let hostname = if let Some(host) = uri.host() {
        host.to_string()
    } else {
        return Err(format!("No host provided in URI: '{:?}'", uri));
    };

    let inner = worker::Socket::builder()
        .secure_transport(transport)
        .connect(hostname, port)
        .map_err(|e| format!("{:?}", e))?;

    Ok(Socket { inner })
}

impl hyper::service::Service<hyper::Uri> for CloudflareConnector {
    type Response = Socket;
    type Error = String;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, String>> + Send>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), String>> {
        // This connector is always ready, but others might not be.
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: hyper::Uri) -> Self::Future {
        Box::pin(connect_uri(uri))
    }
}

impl Connection for Socket {
    fn connected(&self) -> Connected {
        // TODO: We don't know our peer or local address, so cant provide
        // anything useful here.
        Connected::new()
    }
}

impl tokio::io::AsyncRead for Socket {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        {
            let socket = Pin::new(&mut self.inner);
            socket.poll_read(cx, buf)
        }
    }
}

impl tokio::io::AsyncWrite for Socket {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        {
            let socket = Pin::new(&mut self.inner);
            socket.poll_write(cx, buf)
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        {
            let socket = Pin::new(&mut self.inner);
            socket.poll_flush(cx)
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        {
            let socket = Pin::new(&mut self.inner);
            socket.poll_shutdown(cx)
        }
    }
}

pub struct CloudflareExecutor;

impl hyper::rt::Executor<Pin<Box<dyn Future<Output = ()> + Send>>> for CloudflareExecutor {
    fn execute(&self, fut: Pin<Box<dyn Future<Output = ()> + Send>>) {
        wasm_bindgen_futures::spawn_local(fut)
    }
}
