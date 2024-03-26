#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum RequestRedirect {
    Error,
    #[default]
    Follow,
    Manual,
}

unsafe impl Send for RequestRedirect {}
unsafe impl Sync for RequestRedirect {}

impl From<RequestRedirect> for &str {
    fn from(redirect: RequestRedirect) -> Self {
        match redirect {
            RequestRedirect::Error => "error",
            RequestRedirect::Follow => "follow",
            RequestRedirect::Manual => "manual",
        }
    }
}

impl From<RequestRedirect> for web_sys::RequestRedirect {
    fn from(redir: RequestRedirect) -> Self {
        match redir {
            RequestRedirect::Error => web_sys::RequestRedirect::Error,
            RequestRedirect::Follow => web_sys::RequestRedirect::Follow,
            RequestRedirect::Manual => web_sys::RequestRedirect::Manual,
        }
    }
}

impl From<web_sys::RequestRedirect> for RequestRedirect {
    fn from(redir: web_sys::RequestRedirect) -> Self {
        match redir {
            web_sys::RequestRedirect::Error => RequestRedirect::Error,
            web_sys::RequestRedirect::Follow => RequestRedirect::Follow,
            web_sys::RequestRedirect::Manual => RequestRedirect::Manual,
            _ => panic!("unknown redirect"),
        }
    }
}
