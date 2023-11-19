use crate::user::UserInfo;

use worker::WebsocketEvent;

pub enum Data {
    Binary(Box<[u8]>),
    None,
    Text(String),
}

pub enum MessageEvent {
    Close,
    Message(Data),
    Ping,
    Pong,
}

impl MessageEvent {
    pub fn new(event: WebsocketEvent) -> Self {
        match event {
            WebsocketEvent::Message(msg) => {
                if let Some(msg) = msg.text() {
                    return Self::message(msg);
                }
                if let Some(binary) = msg.bytes() {
                    return Self::Message(Data::Binary(binary.into()));
                }

                Self::Message(Data::None)
            }
            WebsocketEvent::Close(_) => Self::Close,
        }
    }

    fn message(msg: String) -> Self {
        if msg.len() == 4 {
            let ping_or_pong = msg.to_lowercase();
            if ping_or_pong == "ping" {
                return Self::Ping;
            }
            if ping_or_pong == "pong" {
                return Self::Pong;
            }
        }

        Self::Message(Data::Text(msg))
    }
}

/// Generic response for all messages.
#[derive(Debug)]
pub struct MsgResponse<'a> {
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

impl<'a> MsgResponse<'a> {
    pub fn new(info: &'a UserInfo, msg: &'a WsMessage) -> Self {
        let UserInfo { color, name, .. } = info;
        match *msg {
            WsMessage::Conn(count) => Self {
                color: Some(color),
                count,
                message: None,
                name,
            },
            WsMessage::Send(message) => Self {
                color: Some(color),
                count: None,
                message,
                name,
            },
        }
    }

    pub fn as_str(&self) -> Box<str> {
        let mut str_ = String::new();
        str_.insert(0, '{');
        str_.insert_str(str_.len(), &v("color", self.color));
        str_.insert_str(str_.len(), &v("count", self.count));
        str_.insert_str(str_.len(), &v("message", self.message));
        str_.insert_str(str_.len(), &format!("{:?}:{:?}", "name", self.name));
        str_.insert(str_.len(), '}');
        str_.into()
    }
}

fn v<T: std::fmt::Debug + Sized>(k: &str, v: Option<T>) -> Box<str> {
    format!("{:?}:{},", k, v.map_or("null".into(), |s| format!("{s:?}"))).into()
}
