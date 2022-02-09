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

    assert_eq!(&msg, "Hello, world!");

    // Drop the socket, causing the connection to get closed.
    drop(socket);

    println!("Dropped socket");

    let got_close_event: bool = util::get("got-close-event", |r| r)
        .text()
        .expect("could not deserialize as text")
        .parse()
        .expect("body was not boolean");
    assert!(got_close_event)
}
