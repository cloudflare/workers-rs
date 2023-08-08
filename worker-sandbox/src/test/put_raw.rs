use worker::{js_sys::Uint8Array, wasm_bindgen::JsValue, *};

#[durable_object]
pub struct PutRawTestObject {
    state: State,
}

impl PutRawTestObject {
    async fn put_raw(&mut self) -> Result<()> {
        let mut storage = self.state.storage();
        let bytes = Uint8Array::new_with_length(3);
        bytes.copy_from(b"123");
        storage.put_raw("bytes", bytes).await?;
        let bytes = storage.get::<Vec<u8>>("bytes").await?;
        storage.delete("bytes").await?;
        assert_eq!(
            bytes, b"123",
            "eficient serialization of bytes is not preserved"
        );
        Ok(())
    }

    async fn put_multiple_raw(&mut self) -> Result<()> {
        let mut storage = self.state.storage();
        let obj = js_sys::Object::new();
        const BAR: &[u8] = b"bar";
        let value = Uint8Array::new_with_length(BAR.len() as _);
        value.copy_from(BAR);
        js_sys::Reflect::set(&obj, &JsValue::from_str("foo"), &value.into())?;
        storage.put_multiple_raw(obj).await?;

        assert_eq!(
            storage.get::<Vec<u8>>("foo").await?,
            BAR,
            "Didn't the right thing with put_multiple_raw"
        );

        Ok(())
    }
}

#[durable_object]
impl DurableObject for PutRawTestObject {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&mut self, _: Request) -> Result<Response> {
        self.put_raw().await?;
        self.put_multiple_raw().await?;

        Response::ok("ok")
    }
}
