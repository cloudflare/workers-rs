use std::fmt;
use std::str::FromStr;

use crate::{Error, Request, RequestInit, URL};

/// HTTP method of a `Request`, parsed from the generated `method() -> String`.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Head,
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
}

impl Method {
    pub fn all() -> [Method; 10] {
        [
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
        ]
    }
}

impl FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(match s.to_ascii_uppercase().as_str() {
            "HEAD" => Method::Head,
            "GET" => Method::Get,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "PATCH" => Method::Patch,
            "DELETE" => Method::Delete,
            "OPTIONS" => Method::Options,
            "CONNECT" => Method::Connect,
            "TRACE" => Method::Trace,
            "REPORT" => Method::Report,
            other => return Err(Error::Rust(format!("unknown HTTP method: {other}"))),
        })
    }
}

impl AsRef<str> for Method {
    fn as_ref(&self) -> &'static str {
        match self {
            Method::Head => "HEAD",
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
            Method::Options => "OPTIONS",
            Method::Connect => "CONNECT",
            Method::Trace => "TRACE",
            Method::Report => "REPORT",
        }
    }
}

impl From<Method> for String {
    fn from(method: Method) -> Self {
        method.as_ref().to_owned()
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl Request {
    /// `new Request(url, { method })`.
    pub fn new_with_method(url: &str, method: Method) -> crate::Result<Request> {
        let init = RequestInit::builder().method(method.as_ref()).build();
        Ok(Request::new_with_str_and_init(url, &init)?)
    }

    fn parsed_url(&self) -> crate::Result<URL> {
        URL::new(self.url())
    }

    /// The path component of the request URL; `/` if the URL does not parse.
    pub fn path(&self) -> String {
        self.parsed_url()
            .map(|url| url.pathname())
            .unwrap_or_else(|_| "/".to_owned())
    }

    /// Deserializes the query string.
    #[cfg(feature = "serde")]
    pub fn query<Q: serde::de::DeserializeOwned>(&self) -> crate::Result<Q> {
        let search = self.parsed_url()?.search();
        Ok(serde_urlencoded::from_str(
            search.strip_prefix('?').unwrap_or(&search),
        )?)
    }

    /// The request URL as a [`url::Url`].
    #[cfg(feature = "url")]
    pub fn url_parsed(&self) -> crate::Result<url::Url> {
        Ok(self.url().parse()?)
    }
}
