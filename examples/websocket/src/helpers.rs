use crate::{error::Error, user::Users};

use std::rc::Rc;
use std::sync::RwLock;
use wasm_bindgen::prelude::*;
use worker::{wasm_bindgen, Date, Response};

#[wasm_bindgen(inline_js = "export function random_color() {
    return `#${Math.floor(Math.random() * 16777215).toString(16)}`;
  }")]
extern "C" {
    /// A random generated color used for css style on the frontend
    pub fn random_color() -> String;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(closure: &JsValue, millis: i32) -> Option<i32>;
}

pub trait IntoResponse {
    fn into_response(self) -> worker::Result<Response>;
}

impl IntoResponse for Result<Response, Error> {
    fn into_response(self) -> worker::Result<Response> {
        match self {
            Ok(res) => Ok(res),
            Err(err) => Ok(err.into_response()?),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> worker::Result<Response> {
        let (msg, status) = self.take();
        let mut res = Response::error(msg, status)?;
        let headers = res.headers_mut();

        let _ = headers.set("content-type", "application/json;charset=UTF-8");
        let _ = headers.set("date", &Date::now().to_string());

        Ok(res)
    }
}

pub trait HeartBeat {
    fn ping(self) -> Closure<dyn FnMut()>;
    fn pong(self) -> Closure<dyn FnMut()>;
    fn run(&mut self);
    fn set_ping_timeout(&mut self);
    fn set_pong_timeout(&mut self);
}

#[derive(Default)]
struct Timeout {
    ping_token: Option<i32>,
    pong_token: Option<i32>,
}

#[derive(Clone)]
pub struct TimeoutHandle<T> {
    handle: Rc<RwLock<Timeout>>,
    item: T,
}

impl<T> TimeoutHandle<T>
where
    Self: HeartBeat,
{
    pub fn run(item: T) {
        Self {
            handle: Rc::new(RwLock::new(Timeout::default())),
            item,
        }
        .run();
    }
}

impl HeartBeat for TimeoutHandle<(Users, Box<str>)> {
    fn ping(mut self) -> Closure<dyn FnMut()> {
        Closure::once(move || {
            let (users, id) = &self.item;
            if users.ping(id).is_some() {
                self.set_pong_timeout();
            }
            // the above is equivalent to the below in javascript
            // clearTimeoutHandle(this.ping_token);
            // this.pong_token = setTimeoutHandle(closure, 5000);
            // function closure() { ... } // this is self.pong() method
            // if (users.ping(id) === undefined) {
            //  clearTimeoutHandle(this.pong_token);
            // }
        })
    }

    fn pong(mut self) -> Closure<dyn FnMut()> {
        Closure::once(move || {
            if let Some(pong) = self.item.0.hb(&self.item.1) {
                if pong == 5 {
                    self.item.0.close(&self.item.1);
                    return;
                }
            } else {
                return;
            }
            self.set_ping_timeout();

            // the above is equivalent to the below in javascript
            // clearTimeoutHandle(this.pong_token);
            // if (users.hb(id) === undefined) {
            //  clearTimeoutHandle(this.ping_token);
            //  return;
            // } else if (users.hb(id) === 5) {
            //  users.close(id);
            //  clearTimeoutHandle(this.ping_token);
            //  return;
            // }
            // this.ping_token = setTimeoutHandle(closure, 5000); // closure if self.ping() method
        })
    }

    fn run(&mut self) {
        self.set_ping_timeout();
    }

    fn set_ping_timeout(&mut self) {
        let closure = set_timeout(&self.clone().ping().into_js_value(), 5000);
        self.handle.write().unwrap().ping_token = closure;
        // the code above is equivalent to the below in javascript
        // this.ping_token = setTimeoutHandle(closure, 5000); // self.ping_token
        // function closure() { ... } // this is self.ping() method
    }

    fn set_pong_timeout(&mut self) {
        let closure = set_timeout(&self.clone().pong().into_js_value(), 5000);
        self.handle.write().unwrap().pong_token = closure;
        // the code above is equivalent to the below in javascript
        // this.pong_token = setTimeoutHandle(closure, 5000); // self.pong_token
        // function closure() { ... } // this is self.pong() method
    }
}
