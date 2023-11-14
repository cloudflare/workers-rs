use std::fmt::Display;

use crate::user::UserInfo;

use serde_json::json;

/// Generic response for all messages.
pub struct Message<'a> {
    /// Unique color of the user's name.
    color: Option<&'a str>,
    /// Number of connected clients, None if the type of message is not [WsMessage::Send].
    count: Option<u64>,
    /// Value of the message, None if the type of message is not [WsMessage::Conn]
    message: Option<&'a str>,
    /// User's name
    name: &'a str,
}

pub enum WsMessage<'a> {
    /// Connected or disconnected client with the number of connected clients
    Conn(Option<u64>),
    /// Value of the message.
    Send(Option<&'a str>),
}

impl<'a> Message<'a> {
    pub fn new(info: &'a UserInfo, msg: &'a WsMessage) -> Self {
        match *msg {
            WsMessage::Conn(count) => Self {
                color: Some(&info.color),
                count,
                message: None,
                name: &info.name,
            },
            WsMessage::Send(message) => Self {
                color: Some(&info.color),
                count: None,
                message,
                name: &info.name,
            },
        }
    }
}

// This trait allows calling to_string and used with worker::WebSocket::send_with_str.
// We can add Serialize to its derive method and call worker::WebSocket::send too.
impl<'a> Display for Message<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &json!({
                "color": self.color,
                "count": self.count,
                "message": self.message,
                "name": self.name
            })
            .to_string(),
        )
    }
}
