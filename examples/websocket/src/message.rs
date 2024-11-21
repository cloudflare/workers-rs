use crate::user::UserInfo;

use worker::WebsocketEvent;

pub enum Data {
    Binary(Box<[u8]>),
    None,
    Text(String),
}

pub enum SendData {
    Text(Box<str>),
    Binary(Box<[u8]>),
}

pub enum MessageEvent {
    Close(u16, String),
    Message(Data),
    Ping,
    Pong,
}

pub enum WsMessage<'a> {
    /// Connected or disconnected client with the number of connected clients
    Conn(Option<u64>),
    /// Value of the message.
    Send(Option<&'a str>),
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
            WebsocketEvent::Close(close) => Self::Close(close.code(), close.reason()),
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
    name: Option<&'a str>,
}

trait ToValue {
    fn to_value(self, k: &str, sep: Option<char>) -> String;
}

impl<T: std::fmt::Debug> ToValue for Option<T> {
    fn to_value(self, k: &str, sep: Option<char>) -> String {
        if self.is_none() {
            format!("{:?}:{}{}", k, "null", sep.unwrap_or('}'))
        } else {
            format!("{:?}:{:?}{}", k, self.unwrap(), sep.unwrap_or('}'))
        }
    }
}

impl<'a> MsgResponse<'a> {
    pub fn new(info: &'a UserInfo, msg: &'a WsMessage) -> Self {
        let UserInfo { color, name, .. } = info;
        match *msg {
            WsMessage::Conn(count) => Self {
                color: Some(color),
                count,
                message: None,
                name: Some(name),
            },
            WsMessage::Send(message) => Self {
                color: Some(color),
                count: None,
                message,
                name: Some(name),
            },
        }
    }

    pub fn as_string(&self) -> String {
        let mut str_ = String::from('{');
        str_.push_str(&self.color.to_value("color", Some(',')));
        str_.push_str(&self.count.to_value("count", Some(',')));
        str_.push_str(&self.message.to_value("message", Some(',')));
        str_.push_str(&self.name.to_value("name", None));

        str_
    }
}
