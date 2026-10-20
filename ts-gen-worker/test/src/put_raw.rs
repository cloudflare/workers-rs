use crate::Env;
use js_sys::Uint8Array;
use ts_gen_worker::*;
use wasm_bindgen::prelude::*;

use crate::SomeSharedData;

#[durable_object]
pub struct PutRawTestObject {
    state: DurableObjectState,
}

impl PutRawTestObject {
    async fn put_raw(&self) -> Result<()> {
        let storage = self.state.storage();
        let bytes = Uint8Array::new_with_length(3);
        bytes.copy_from(b"123");
        storage.put("bytes", &bytes).await?;
        let bytes: Vec<u8> = storage
            .get_into("bytes")
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
        let entries = js_sys::Object::from_iter([("foo", Uint8Array::from(BAR))]);
        storage.put_with_entries(&entries).await?;

        assert_eq!(
            storage
                .get_into::<Vec<u8>>("foo")
                .await?
                .expect("get('foo') yielded nothing"),
            BAR,
            "Didn't get the right thing with put_multiple_raw"
        );

        Ok(())
    }
}

impl DurableObject for PutRawTestObject {
    type Env = Env;

    fn new(state: DurableObjectState, _env: Env) -> PutRawTestObject {
        Self { state }
    }

    async fn fetch(&self, _req: Request) -> Result<Response> {
        self.put_raw().await?;
        self.put_multiple_raw().await?;

        Response::builder().ok("ok")
    }
}

pub(crate) async fn handle_put_raw(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let namespace = env.put_raw_test_object();
    let id = namespace.new_unique_id()?;
    let stub = namespace.get(&id)?;
    stub.fetch(&req).await
}
