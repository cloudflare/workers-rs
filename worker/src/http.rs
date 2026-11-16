use std::fmt::Display;

#[cfg(feature = "http")]
pub mod body;
#[cfg(feature = "http")]
mod header;
#[cfg(feature = "http")]
mod redirect;
#[cfg(feature = "http")]
pub mod request;
#[cfg(feature = "http")]
pub mod response;

/// A [`Method`](https://developer.mozilla.org/en-US/docs/Web/API/Request/method) representation
/// used on Request objects.
#[derive(Default, Debug, Clone, PartialEq, Hash, Eq)]
pub enum Method {
    Head = 0,
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Connect,
    Trace,
    Report,
    /// A safe, idempotent request with content, defined by RFC 10008.
    Query,
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
            Method::Report,
            Method::Query,
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
            "REPORT" => Method::Report,
            "QUERY" => Method::Query,
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
            Method::Report => "REPORT",
            Method::Query => "QUERY",
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let s: String = (*self).clone().into();
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::Method;

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn query_method_round_trips() {
        for token in ["QUERY", "query", "QuErY"] {
            let method = Method::from(token.to_owned());
            assert_eq!(method, Method::Query);
            assert_eq!(method.as_ref(), "QUERY");
            assert_eq!(String::from(method.clone()), "QUERY");
            assert_eq!(method.to_string(), "QUERY");
            assert!(Method::all().contains(&method));
        }
    }
}
