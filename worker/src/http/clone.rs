use crate::{
    body::{Body, BodyInner},
    AbortSignal, WebSocket,
};

use super::{request, response, RequestRedirect};

mod sealed {
    use crate::body::Body;

    pub trait Sealed {}

    impl Sealed for http::Request<Body> {}
    impl Sealed for http::Response<Body> {}
}

/// Extension trait for cloning [`Request`] and [`Response`] types.
///
/// [`Request`]: http::Request
/// [`Response`]: http::Response
pub trait HttpClone: sealed::Sealed
where
    Self: Sized,
{
    /// The JS equivalent of this type.
    type JsValue;

    /// Returns a copy of the value.
    ///
    /// Cloning does not copy over all [`Extensions`] of the value.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use worker::body::Body;
    /// use worker::http::HttpClone;
    ///
    /// let mut req = http::Request::get("https://www.rust-lang.org/")
    ///     .body(Body::empty())
    ///     .unwrap();
    ///
    /// let clone = req.clone();
    /// ```
    ///
    /// [`Extensions`]: http::Extensions
    fn clone(&mut self) -> Self;

    /// Returns a copy of the value as its JS equivalent.
    fn clone_raw(&mut self) -> Self::JsValue;

    /// Returns a copy of the inner value.
    ///
    /// This function is faster than [`clone()`] as it does not have to translate the `http` value into its JS equivalent first.
    /// However, any changes made to the `http` value will not be reflected in the copy.
    ///
    /// This function returns `None` if the value did not originate from JS or if the body has already been accessed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use worker::body::Body;
    /// use worker::http::{request, HttpClone};
    ///
    /// let req = web_sys::Request::new_with_str("flowers.jpg").unwrap();
    /// let req = request::from_wasm(req);
    ///
    /// let clone = req.clone_inner().unwrap();
    /// ```
    ///
    /// [`clone()`]: Self::clone()
    fn clone_inner(&self) -> Option<Self>;

    /// Returns a copy of the inner value as its JS equivalent.
    fn clone_inner_raw(&self) -> Option<Self::JsValue>;
}

impl HttpClone for http::Request<Body> {
    type JsValue = web_sys::Request;

    fn clone(&mut self) -> Self {
        let mut clone = request::from_wasm(self.clone_raw());
        *clone.version_mut() = self.version();
        clone
    }

    fn clone_raw(&mut self) -> Self::JsValue {
        // Take the request temporarily
        let mut req = std::mem::take(self);

        // Track original values that are lost in the JS type
        let version = req.version();
        let extensions = std::mem::take(req.extensions_mut());

        // Insert clones of extensions used by into_wasm
        if let Some(signal) = extensions.get::<AbortSignal>() {
            req.extensions_mut().insert(signal.clone());
        }

        if let Some(redirect) = extensions.get::<RequestRedirect>() {
            req.extensions_mut().insert(*redirect);
        }

        // Do the conversion
        let req = request::into_wasm(req);

        // Should never panic as the request body has not been accessed yet
        let clone = req.clone().unwrap();

        // Put back the original request
        *self = request::from_wasm(req);
        *self.version_mut() = version;
        *self.extensions_mut() = extensions;

        clone
    }

    fn clone_inner(&self) -> Option<Self> {
        self.clone_inner_raw().map(request::from_wasm)
    }

    fn clone_inner_raw(&self) -> Option<Self::JsValue> {
        match self.body().inner() {
            // Should never panic as the request body has not been accessed yet
            BodyInner::Request(req) => Some(req.clone().unwrap()),
            _ => None,
        }
    }
}

impl HttpClone for http::Response<Body> {
    type JsValue = web_sys::Response;

    fn clone(&mut self) -> Self {
        let mut clone = response::from_wasm(self.clone_raw());
        *clone.version_mut() = self.version();
        clone
    }

    fn clone_raw(&mut self) -> Self::JsValue {
        // Take the response temporarily
        let mut res = std::mem::take(self);

        // Track original values that are lost in the JS type
        let version = res.version();
        let extensions = std::mem::take(res.extensions_mut());

        // Insert clones of extensions used by into_wasm
        if let Some(websocket) = extensions.get::<WebSocket>() {
            res.extensions_mut().insert(websocket.clone());
        }

        // Do the conversion
        let res = response::into_wasm(res);

        // Should never panic as the response body has not been accessed yet
        let clone = res.clone().unwrap();

        // Put back the original response
        *self = response::from_wasm(res);
        *self.version_mut() = version;
        *self.extensions_mut() = extensions;

        clone
    }

    fn clone_inner(&self) -> Option<Self> {
        self.clone_inner_raw().map(response::from_wasm)
    }

    fn clone_inner_raw(&self) -> Option<Self::JsValue> {
        match self.body().inner() {
            // Should never panic as the response body has not been accessed yet
            BodyInner::Response(res) => Some(res.clone().unwrap()),
            _ => None,
        }
    }
}
