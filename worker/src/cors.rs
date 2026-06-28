use crate::{Error, Headers, Method, Result};

/// Cors struct, holding cors configuration
#[derive(Debug, Clone)]
pub struct Cors {
    credentials: bool,
    max_age: Option<u32>,
    origin: Option<String>,
    methods: Vec<Method>,
    allowed_headers: Vec<String>,
    exposed_headers: Vec<String>,
}

/// Creates a default cors configuration, which will do nothing.
impl Default for Cors {
    fn default() -> Self {
        Self {
            credentials: false,
            max_age: None,
            origin: None,
            methods: vec![],
            allowed_headers: vec![],
            exposed_headers: vec![],
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
    /// The actual request origin is also required.
    pub fn with_origins<S1: Into<String>, S2: AsRef<str>, V: IntoIterator<Item = S1>>(
        mut self,
        origins: V,
        req_origin: S2,
    ) -> Self {
        let req_origin_str = req_origin.as_ref();
        self.origin = origins
            .into_iter()
            .map(|item| item.into())
            .filter(|s| s == req_origin_str)
            .next();
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
        if let Some(origin) = &self.origin {
            headers.set("Access-Control-Allow-Origin", origin)?;
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
