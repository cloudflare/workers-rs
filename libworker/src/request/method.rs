#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Method {
    Head,
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Connect,
    Trace,
}

impl<T: AsRef<str>> From<T> for Method {
    fn from(string: T) -> Self {
        match string.as_ref().to_ascii_uppercase().as_str() {
            "HEAD" => Method::Head,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "PATCH" => Method::Patch,
            "DELETE" => Method::Delete,
            "OPTIONS" => Method::Options,
            "CONNECT" => Method::Connect,
            "TRACE" => Method::Trace,
            _ => Method::Get,
        }
    }
}
