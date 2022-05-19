use worker_sys::TextEncoder as EdgeTextEncoder;

/// A [TextEncoder](https://developer.mozilla.org/en-US/docs/Web/API/FormData) representation of the
/// request body, providing access to form encoded fields and files.
/// TODO: Finish docs
pub struct TextEncoder {
    inner: EdgeTextEncoder,
}

impl TextEncoder {
    pub fn new() -> Self {
        Self {
            inner: EdgeTextEncoder::new().unwrap(),
        }
    }

    pub fn encode(&self, input: String) -> Vec<u8> {
        self.inner.encode_with_input(input.as_str())
    }
}
