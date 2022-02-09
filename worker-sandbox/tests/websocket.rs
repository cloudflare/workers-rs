use reqwest::Url;
use tungstenite::{connect, Message};

mod util;

#[test]
fn websocket() {
    util::expect_wrangler();

    let (mut socket, _) =
        connect(Url::parse("ws://localhost:8787/websocket").unwrap()).expect("Can't connect");

    socket
        .write_message(Message::Text("Hello, world!".into()))
        .unwrap();

    let msg = socket
        .read_message()
        .and_then(|msg| msg.into_text())
        .unwrap();

    assert_eq!(&msg, "Hello, world!")
}
