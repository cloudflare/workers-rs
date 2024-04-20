mod sys {
    use ::wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(extends = ::worker::js_sys::Object)]
        pub type CalculatorSys;
        #[wasm_bindgen(method, catch, js_name = "add")]
        pub fn add(
            this: &CalculatorSys,
            a: u32,
            b: u32,
        ) -> std::result::Result<
            ::worker::js_sys::Promise,
            ::worker::wasm_bindgen::JsValue,
        >;
    }
}
#[async_trait::async_trait]
pub trait Calculator {
    async fn add(&self, a: u32, b: u32) -> ::worker::Result<u64>;
}
pub struct CalculatorService(::worker::send::SendWrapper<sys::CalculatorSys>);
#[async_trait::async_trait]
impl Calculator for CalculatorService {
    async fn add(&self, a: u32, b: u32) -> ::worker::Result<u64> {
        let promise = self.0.add(a, b)?;
        let fut = ::worker::send::SendFuture::new(
            ::worker::wasm_bindgen_futures::JsFuture::from(promise),
        );
        let output = fut.await?;
        Ok(::serde_wasm_bindgen::from_value(output)?)
    }
}
impl From<::worker::Fetcher> for CalculatorService {
    fn from(fetcher: ::worker::Fetcher) -> Self {
        Self(::worker::send::SendWrapper::new(fetcher.into_rpc()))
    }
}
