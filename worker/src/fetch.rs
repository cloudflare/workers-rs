use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::WorkerGlobalScope;

use crate::{
    body::Body,
    http::{request, response},
    Error, Result,
};

pub async fn fetch(req: http::Request<impl Into<Body>>) -> Result<http::Response<Body>> {
    let req = req.map(Into::into);
    let (tx, rx) = futures_channel::oneshot::channel();

    wasm_bindgen_futures::spawn_local(async move {
        let global = js_sys::global().unchecked_into::<WorkerGlobalScope>();

        let req = request::into_wasm(req);
        let promise = global.fetch_with_request(&req);

        let res = JsFuture::from(promise)
            .await
            .map(|res| response::from_wasm(res.unchecked_into()))
            .map_err(Error::from);

        tx.send(res).unwrap();
    });

    rx.await.unwrap()
}

fn _assert_send() {
    fn _assert_send(_: impl Send) {}

    _assert_send(fetch(http::Request::new(())));
}
