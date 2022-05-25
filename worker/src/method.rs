/// A [`Method`](https://developer.mozilla.org/en-US/docs/Web/API/Request/method) representation
/// used on Request objects.
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Method {
    Head = 0,
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Connect,
    Trace,
}

impl Method {
    pub fn all() -> Vec<Method> {
        vec![
            Method::Head,
            Method::Get,
            Method::Post,
            Method::Put,
            Method::Patch,
            Method::Delete,
            Method::Options,
            Method::Connect,
            Method::Trace,
        ]
    }
}

impl From<String> for Method {
    fn from(m: String) -> Self {
        match m.to_ascii_uppercase().as_str() {
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

impl From<Method> for String {
    fn from(val: Method) -> Self {
        val.as_ref().to_string()
    }
}

impl AsRef<str> for Method {
    fn as_ref(&self) -> &'static str {
        match self {
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
            Method::Options => "OPTIONS",
            Method::Connect => "CONNECT",
            Method::Trace => "TRACE",
            Method::Get => "GET",
        }
    }
}

impl ToString for Method {
    fn to_string(&self) -> String {
        (*self).clone().into()
    }
}

impl Default for Method {
    fn default() -> Self {
        Method::Get
    }
}
