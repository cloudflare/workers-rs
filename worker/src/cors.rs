use crate::{Error, Headers, Method, Result};

/// Cors struct, holding cors configuration
#[derive(Debug, Clone)]
pub struct Cors {
    credentials: bool,
    max_age: Option<u32>,
    origins: Vec<String>,
    methods: Vec<Method>,
    allowed_headers: Vec<String>,
    exposed_headers: Vec<String>,
    request_origin: RequestOrigin,
}

#[derive(Debug, Clone)]
enum RequestOrigin {
    /// `with_request_origin` was never called.
    Unset,
    /// `with_request_origin` was called, but the request had no `Origin` header.
    Absent,
    /// `with_request_origin` was called and the request had an `Origin` header.
    Present(String),
}

/// Creates a default cors configuration, which will do nothing.
impl Default for Cors {
    fn default() -> Self {
        Self {
            credentials: false,
            max_age: None,
            origins: vec![],
            methods: vec![],
            allowed_headers: vec![],
            exposed_headers: vec![],
            request_origin: RequestOrigin::Unset,
        }
    }
}

impl Cors {
    /// `new` constructor for convenience; does the same as `Self::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures whether cors is allowed to share credentials or not.
    pub fn with_credentials(mut self, credentials: bool) -> Self {
        self.credentials = credentials;
        self
    }

    /// Configures how long cors is allowed to cache a preflight-response.
    pub fn with_max_age(mut self, max_age: u32) -> Self {
        self.max_age = Some(max_age);
        self
    }

    /// Configures which origins are allowed for cors.
    ///
    /// `Access-Control-Allow-Origin` may only contain a single origin or `*`.
    /// When several origins are given, also call [`Cors::with_request_origin`]
    /// so the matching one can be selected; otherwise [`Cors::apply_headers`]
    /// returns an error.
    pub fn with_origins<S: Into<String>, V: IntoIterator<Item = S>>(mut self, origins: V) -> Self {
        self.origins = origins
            .into_iter()
            .map(|item| item.into())
            .collect::<Vec<String>>();
        self
    }

    /// `Access-Control-Allow-Origin` may only contain a single origin, so when
    /// several origins are configured with [`Cors::with_origins`] the one to
    /// send is picked by matching it against this value. For several origins,
    /// [`Cors::apply_headers`] returns an error if this method was not called.
    pub fn with_request_origin<S: Into<String>>(mut self, origin: Option<S>) -> Self {
        self.request_origin = match origin {
            Some(origin) => RequestOrigin::Present(origin.into()),
            None => RequestOrigin::Absent,
        };
        self
    }

    /// Configures which methods are allowed for cors.
    pub fn with_methods<V: IntoIterator<Item = Method>>(mut self, methods: V) -> Self {
        self.methods = methods.into_iter().collect();
        self
    }

    /// Configures which headers are allowed for cors.
    pub fn with_allowed_headers<S: Into<String>, V: IntoIterator<Item = S>>(
        mut self,
        headers: V,
    ) -> Self {
        self.allowed_headers = headers
            .into_iter()
            .map(|item| item.into())
            .collect::<Vec<String>>();
        self
    }

    /// Configures which headers the client is allowed to access.
    pub fn with_exposed_headers<S: Into<String>, V: IntoIterator<Item = S>>(
        mut self,
        headers: V,
    ) -> Self {
        self.exposed_headers = headers
            .into_iter()
            .map(|item| item.into())
            .collect::<Vec<String>>();
        self
    }

    /// Applies the cors configuration to response headers.
    pub fn apply_headers(&self, headers: &mut Headers) -> Result<()> {
        if self.credentials {
            headers.set("Access-Control-Allow-Credentials", "true")?;
        }
        if let Some(ref max_age) = self.max_age {
            headers.set("Access-Control-Max-Age", format!("{max_age}").as_str())?;
        }
        let selection = select_allow_origin(&self.origins, &self.request_origin)?;
        if let Some(origin) = selection.allow_origin {
            headers.set("Access-Control-Allow-Origin", origin.as_str())?;
        }
        if selection.vary && !vary_covers_origin(headers.get("Vary")?.as_deref()) {
            headers.append("Vary", "Origin")?;
        }
        if !self.methods.is_empty() {
            headers.set(
                "Access-Control-Allow-Methods",
                concat_vec_to_string(self.methods.as_slice())?.as_str(),
            )?;
        }
        if !self.allowed_headers.is_empty() {
            headers.set(
                "Access-Control-Allow-Headers",
                concat_vec_to_string(self.allowed_headers.as_slice())?.as_str(),
            )?;
        }
        if !self.exposed_headers.is_empty() {
            headers.set(
                "Access-Control-Expose-headers",
                concat_vec_to_string(self.exposed_headers.as_slice())?.as_str(),
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
struct OriginSelection {
    allow_origin: Option<String>,
    vary: bool,
}

fn select_allow_origin(origins: &[String], request: &RequestOrigin) -> Result<OriginSelection> {
    let (allow_origin, vary) = match origins {
        [] => (None, false),
        _ if origins.iter().any(|o| o == "*") => (Some("*".to_string()), false),
        [origin] => (Some(origin.clone()), false),
        _ => match request {
            RequestOrigin::Unset => {
                return Err(Error::RustError(
                    "Cors has several origins, call `with_request_origin` with the request's `Origin` header to select one."
                        .to_string(),
                ))
            }
            RequestOrigin::Absent => (None, true),
            RequestOrigin::Present(req) => (origins.iter().find(|o| *o == req).cloned(), true),
        },
    };
    Ok(OriginSelection { allow_origin, vary })
}

fn vary_covers_origin(vary: Option<&str>) -> bool {
    vary.is_some_and(|vary| {
        vary.split(',')
            .map(str::trim)
            .any(|field| field == "*" || field.eq_ignore_ascii_case("origin"))
    })
}

fn concat_vec_to_string<S: AsRef<str>>(vec: &[S]) -> Result<String> {
    let str = vec.iter().fold("".to_owned(), |mut init, item| {
        init.push(',');
        init.push_str(item.as_ref());
        init
    });
    if !str.is_empty() {
        Ok(str[1..].to_string())
    } else {
        Err(Error::RustError(
            "Tried to concat header values without values.".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_origins_emits_nothing() {
        let want = OriginSelection {
            allow_origin: None,
            vary: false,
        };
        assert_eq!(
            select_allow_origin(&[], &RequestOrigin::Unset).unwrap(),
            want
        );
    }

    #[test]
    fn wildcard_is_sent_without_vary() {
        let want = OriginSelection {
            allow_origin: Some("*".into()),
            vary: false,
        };
        assert_eq!(
            select_allow_origin(&["*".into()], &RequestOrigin::Unset).unwrap(),
            want
        );
        assert_eq!(
            select_allow_origin(
                &["*".into()],
                &RequestOrigin::Present("https://a.com".into())
            )
            .unwrap(),
            want
        );
        assert_eq!(
            select_allow_origin(
                &["https://a.com".into(), "*".into()],
                &RequestOrigin::Absent
            )
            .unwrap(),
            want
        );
    }

    #[test]
    fn single_origin_is_static_and_sent_without_vary() {
        let want = OriginSelection {
            allow_origin: Some("https://a.com".into()),
            vary: false,
        };
        assert_eq!(
            select_allow_origin(&["https://a.com".into()], &RequestOrigin::Unset).unwrap(),
            want
        );
        assert_eq!(
            select_allow_origin(
                &["https://a.com".into()],
                &RequestOrigin::Present("https://evil.com".into())
            )
            .unwrap(),
            want
        );
    }

    #[test]
    fn several_origins_echo_the_matching_one() {
        let origins = ["https://a.com".to_string(), "https://b.com".to_string()];
        assert_eq!(
            select_allow_origin(&origins, &RequestOrigin::Present("https://b.com".into())).unwrap(),
            OriginSelection {
                allow_origin: Some("https://b.com".into()),
                vary: true,
            }
        );
    }

    #[test]
    fn several_origins_omit_header_when_nothing_matches() {
        let origins = ["https://a.com".to_string(), "https://b.com".to_string()];
        let want = OriginSelection {
            allow_origin: None,
            vary: true,
        };
        assert_eq!(
            select_allow_origin(&origins, &RequestOrigin::Present("https://evil.com".into()))
                .unwrap(),
            want
        );
        assert_eq!(
            select_allow_origin(&origins, &RequestOrigin::Present("https://A.com".into())).unwrap(),
            want
        );
        assert_eq!(
            select_allow_origin(&origins, &RequestOrigin::Absent).unwrap(),
            want
        );
    }

    #[test]
    fn null_origin_is_echoed_only_when_configured() {
        let with_null = ["https://a.com".to_string(), "null".to_string()];
        assert_eq!(
            select_allow_origin(&with_null, &RequestOrigin::Present("null".into())).unwrap(),
            OriginSelection {
                allow_origin: Some("null".into()),
                vary: true,
            }
        );

        let without_null = ["https://a.com".to_string(), "https://b.com".to_string()];
        assert_eq!(
            select_allow_origin(&without_null, &RequestOrigin::Present("null".into())).unwrap(),
            OriginSelection {
                allow_origin: None,
                vary: true,
            }
        );
    }

    #[test]
    fn duplicate_origins_count_as_several() {
        let origins = ["https://a.com".to_string(), "https://a.com".to_string()];
        assert_eq!(
            select_allow_origin(&origins, &RequestOrigin::Present("https://a.com".into())).unwrap(),
            OriginSelection {
                allow_origin: Some("https://a.com".into()),
                vary: true,
            }
        );
        assert!(select_allow_origin(&origins, &RequestOrigin::Unset).is_err());
    }

    #[test]
    fn vary_origin_is_added_unless_already_covered() {
        assert!(!vary_covers_origin(None));
        assert!(!vary_covers_origin(Some("")));
        assert!(!vary_covers_origin(Some("Accept-Encoding")));
        assert!(!vary_covers_origin(Some("Accept-Encoding, Originator")));

        assert!(vary_covers_origin(Some("Origin")));
        assert!(vary_covers_origin(Some("origin")));
        assert!(vary_covers_origin(Some(" Accept-Encoding ,  ORIGIN ")));
        assert!(vary_covers_origin(Some("*")));
        assert!(vary_covers_origin(Some("Accept-Encoding, *")));
    }

    #[test]
    fn several_origins_without_request_origin_is_an_error() {
        let origins = ["https://a.com".to_string(), "https://b.com".to_string()];
        assert!(select_allow_origin(&origins, &RequestOrigin::Unset).is_err());
    }
}
