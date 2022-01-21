use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    time::Duration,
};

use serde::de::DeserializeOwned;

/// How long we'll attempt to connect to wrangler before we assume it's not running.
const WAIT_FOR_WRANGLER: Duration = Duration::from_secs(1);

/// Ensures that the [Wrangler](https://github.com/cloudflare/wrangler) dev server is running so we
/// can make requests to the worker it's previewing.
pub fn expect_wrangler() {
    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8787).into();

    // Try to connect to wrangler's dev server, if it can't be reached assume the user isn't running it.
    if let Err(_) = TcpStream::connect_timeout(&addr, WAIT_FOR_WRANGLER) {
        panic!("Unable to verify wrangler is running");
    }
}

pub fn get(endpoint: &str) -> String {
    expect_wrangler();

    reqwest::blocking::get(&format!("http://127.0.0.1:8787/{endpoint}"))
        .expect("could not make request to wrangler")
        .error_for_status()
        .expect(&format!("received invalid status code for {endpoint}"))
        .text()
        .expect("body could not be decoded into valid UTF-8")
}

pub fn get_json<T: DeserializeOwned>(endpoint: &str) -> T {
    expect_wrangler();

    reqwest::blocking::get(&format!("http://127.0.0.1:8787/{endpoint}"))
        .expect("could not make request to wrangler")
        .error_for_status()
        .expect(&format!("received invalid status code for {endpoint}"))
        .json()
        .expect("body could not be decoded into valid UTF-8")
}
