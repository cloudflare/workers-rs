use crate::{
    body::{Body, BodyInner},
    AbortSignal, WebSocket,
};

use super::{request, response};

pub trait HttpClone
where
    Self: Sized,
{
    type JsValue;

    fn clone(&mut self) -> Self;
    fn clone_raw(&mut self) -> Self::JsValue;
    fn clone_inner(&self) -> Option<Self>;
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
        req.extensions_mut()
            .insert(extensions.get::<AbortSignal>().cloned());

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
        res.extensions_mut()
            .insert(extensions.get::<WebSocket>().cloned());

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
