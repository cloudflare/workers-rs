use crate::{Headers, Method, Response, Result};

/// CORS response headers, applied to a `Headers` or a built `Response`. The default
/// configuration sets nothing.
#[derive(Debug, Clone, Default)]
pub struct Cors {
    credentials: bool,
    max_age: Option<u32>,
    origins: Vec<String>,
    methods: Vec<Method>,
    allowed_headers: Vec<String>,
    exposed_headers: Vec<String>,
}

impl Cors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_credentials(mut self, credentials: bool) -> Self {
        self.credentials = credentials;
        self
    }

    /// How long a preflight response may be cached, in seconds.
    pub fn with_max_age(mut self, max_age: u32) -> Self {
        self.max_age = Some(max_age);
        self
    }

    pub fn with_origins<S: Into<String>, V: IntoIterator<Item = S>>(mut self, origins: V) -> Self {
        self.origins = origins.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_methods<V: IntoIterator<Item = Method>>(mut self, methods: V) -> Self {
        self.methods = methods.into_iter().collect();
        self
    }

    pub fn with_allowed_headers<S: Into<String>, V: IntoIterator<Item = S>>(
        mut self,
        headers: V,
    ) -> Self {
        self.allowed_headers = headers.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_exposed_headers<S: Into<String>, V: IntoIterator<Item = S>>(
        mut self,
        headers: V,
    ) -> Self {
        self.exposed_headers = headers.into_iter().map(Into::into).collect();
        self
    }

    pub fn apply_headers(&self, headers: &Headers) -> Result<()> {
        if self.credentials {
            headers.set("Access-Control-Allow-Credentials", "true")?;
        }
        if let Some(max_age) = self.max_age {
            headers.set("Access-Control-Max-Age", max_age.to_string())?;
        }
        if !self.origins.is_empty() {
            headers.set("Access-Control-Allow-Origin", join(&self.origins))?;
        }
        if !self.methods.is_empty() {
            headers.set("Access-Control-Allow-Methods", join(&self.methods))?;
        }
        if !self.allowed_headers.is_empty() {
            headers.set("Access-Control-Allow-Headers", join(&self.allowed_headers))?;
        }
        if !self.exposed_headers.is_empty() {
            headers.set("Access-Control-Expose-Headers", join(&self.exposed_headers))?;
        }
        Ok(())
    }
}

fn join<S: AsRef<str>>(values: &[S]) -> String {
    values
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>()
        .join(",")
}

impl Response {
    /// Applies `cors` to this response's headers. Fails on an immutable headers guard,
    /// such as a response received from `fetch`.
    pub fn with_cors(self, cors: &Cors) -> Result<Self> {
        cors.apply_headers(&self.headers())?;
        Ok(self)
    }
}
