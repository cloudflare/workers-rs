// Cargo compiles each integration test individually so we can run into scenarios like this where
// cargo emits a warning in `util.rs` saying that some of the exported functions aren't used. This
// warning is raised because the websocket specifically doesn't use those functions, it doesn't
// consider their usage in `requests.rs` when compiling this test.
#![allow(unused)]

use reqwest::Url;
use tungstenite::{connect, Message};

mod util;

#[test]
fn websocket() {
    util::expect_wrangler();

    let (mut socket, _) =
        connect(Url::parse("ws://127.0.0.1:8787/websocket").unwrap()).expect("Can't connect");

    socket
        .write_message(Message::Text("Hello, world!".into()))
        .unwrap();

    let msg = socket
        .read_message()
        .and_then(|msg| msg.into_text())
        .unwrap();

    assert_eq!(&msg, "Hello, world!");

    // Drop the socket, causing the connection to get closed.
    drop(socket);

    println!("Dropped socket");

    let got_close_event: bool = util::get("websocket/closed", |r| r)
        .text()
        .expect("could not deserialize as text")
        .parse()
        .expect("body was not boolean");
    assert!(got_close_event)
}
