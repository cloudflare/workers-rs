use crate::{
    fetch, fetch_with_init, fetch_with_str, fetch_with_str_and_init, AbortController, AbortSignal,
    Request, RequestInit, Response, Result,
};

/// A fetch call built from a URL or a `Request`; `send` runs it. A thin convenience
/// over the generated `fetch` / `fetch_with_str` for code written against `worker`.
#[derive(Debug)]
pub enum Fetch {
    Url(String),
    Request(Request),
}

impl Fetch {
    pub async fn send(&self) -> Result<Response> {
        match self {
            Fetch::Url(url) => fetch_with_str(url).await,
            Fetch::Request(req) => fetch(req).await,
        }
    }

    pub async fn send_with_signal(&self, signal: &AbortSignal) -> Result<Response> {
        let init = RequestInit::builder().signal(signal).build();
        match self {
            Fetch::Url(url) => fetch_with_str_and_init(url, &init).await,
            Fetch::Request(req) => fetch_with_init(req, &init).await,
        }
    }
}

impl From<String> for Fetch {
    fn from(url: String) -> Self {
        Fetch::Url(url)
    }
}

impl From<&str> for Fetch {
    fn from(url: &str) -> Self {
        Fetch::Url(url.to_owned())
    }
}

impl From<Request> for Fetch {
    fn from(req: Request) -> Self {
        Fetch::Request(req)
    }
}

#[cfg(feature = "url")]
impl From<url::Url> for Fetch {
    fn from(url: url::Url) -> Self {
        Fetch::Url(url.into())
    }
}

impl Default for AbortController {
    fn default() -> Self {
        Self::new()
    }
}
