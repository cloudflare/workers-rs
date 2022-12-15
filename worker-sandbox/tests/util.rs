use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    time::Duration,
};

use http::Method;
use reqwest::blocking::{Client, RequestBuilder, Response};

/// How long we'll attempt to connect to wrangler before we assume it's not running.
const WAIT_FOR_WRANGLER: Duration = Duration::from_secs(1);

/// Ensures that the [Wrangler](https://github.com/cloudflare/wrangler) dev server is running so we
/// can make requests to the worker it's previewing.
pub fn expect_wrangler() {
    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8787).into();

    // Try to connect to wrangler's dev server, if it can't be reached assume the user isn't running it.
    if TcpStream::connect_timeout(&addr, WAIT_FOR_WRANGLER).is_err() {
        panic!("Unable to verify wrangler is running");
    }
}

pub fn get(endpoint: &str, builder_fn: impl FnOnce(RequestBuilder) -> RequestBuilder) -> Response {
    expect_wrangler();

    let builder = Client::new().get(format!("http://127.0.0.1:8787/{endpoint}"));
    let builder = builder_fn(builder);

    builder
        .send()
        .expect("could not make request to wrangler")
        .error_for_status()
        .unwrap_or_else(|_| panic!("received invalid status code for {}", endpoint))
}

pub fn post(endpoint: &str, builder_fn: impl FnOnce(RequestBuilder) -> RequestBuilder) -> Response {
    expect_wrangler();

    let builder = Client::new().post(format!("http://127.0.0.1:8787/{endpoint}"));
    let builder = builder_fn(builder);

    builder
        .send()
        .expect("could not make request to wrangler")
        .error_for_status()
        .unwrap_or_else(|_| panic!("received invalid status code for {}", endpoint))
}

#[allow(dead_code)]
pub fn put(endpoint: &str, builder_fn: impl FnOnce(RequestBuilder) -> RequestBuilder) -> Response {
    expect_wrangler();

    let builder = Client::new().put(format!("http://127.0.0.1:8787/{endpoint}"));
    let builder = builder_fn(builder);

    builder
        .send()
        .expect("could not make request to wrangler")
        .error_for_status()
        .unwrap_or_else(|_| panic!("received invalid status code for {}", endpoint))
}

#[allow(dead_code)]
pub fn options(
    endpoint: &str,
    builder_fn: impl FnOnce(RequestBuilder) -> RequestBuilder,
) -> Response {
    expect_wrangler();
    let builder =
        Client::new().request(Method::OPTIONS, format!("http://127.0.0.1:8787/{endpoint}"));
    let builder = builder_fn(builder);

    builder
        .send()
        .expect("could not make request to wrangler")
        .error_for_status()
        .unwrap_or_else(|_| panic!("received invalid status code for {}", endpoint))
}
