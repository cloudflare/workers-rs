use std::convert::TryFrom;
use worker::{
    durable_object,
    js_sys::{self, Uint8Array},
    wasm_bindgen::{self, JsValue},
    DurableObject, Env, Request, Response, Result, State,
};

use crate::SomeSharedData;

#[durable_object]
pub struct PutRawTestObject {
    state: State,
}

impl PutRawTestObject {
    async fn put_raw(&self) -> Result<()> {
        let storage = self.state.storage();
        let bytes = Uint8Array::new_with_length(3);
        bytes.copy_from(b"123");
        storage.put_raw("bytes", bytes).await?;
        let bytes = storage
            .get::<Vec<u8>>("bytes")
            .await?
            .expect("get after put yielded nothing");
        storage.delete("bytes").await?;
        assert_eq!(
            bytes, b"123",
            "eficient serialization of bytes is not preserved"
        );
        Ok(())
    }

    async fn put_multiple_raw(&self) -> Result<()> {
        const BAR: &[u8] = b"bar";
        let storage = self.state.storage();
        let obj = js_sys::Object::new();
        let value = Uint8Array::new_with_length(u32::try_from(BAR.len()).unwrap());
        value.copy_from(BAR);
        js_sys::Reflect::set(&obj, &JsValue::from_str("foo"), &value.into())?;
        storage.put_multiple_raw(obj).await?;

        assert_eq!(
            storage
                .get::<Vec<u8>>("foo")
                .await?
                .expect("get('foo') yielded nothing"),
            BAR,
            "Didn't get the right thing with put_multiple_raw"
        );

        Ok(())
    }
}

impl DurableObject for PutRawTestObject {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&self, _: Request) -> Result<Response> {
        self.put_raw().await?;
        self.put_multiple_raw().await?;

        Response::ok("ok")
    }
}

pub(crate) async fn handle_put_raw(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let namespace = env.durable_object("PUT_RAW_TEST_OBJECT")?;
    let id = namespace.unique_id()?;
    let stub = id.get_stub()?;
    stub.fetch_with_request(req).await
}
