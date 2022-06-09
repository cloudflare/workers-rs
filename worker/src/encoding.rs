use worker_sys::TextEncoder as EdgeTextEncoder;

/// A [TextEncoder](https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder) 
/// takes a stream of code points as input and emits a stream of UTF-8 bytes.
pub struct TextEncoder {
    inner: EdgeTextEncoder,
}

impl TextEncoder {
    /// Returns a newly constructed `TextEncoder` that will generate a byte stream with UTF-8 encoding.
    pub fn new() -> Self {
        Self {
            inner: EdgeTextEncoder::new().unwrap(),
        }
    }

    /// Takes a string as input, and returns a vector containing UTF-8 encoded text.
    pub fn encode(&self, input: String) -> Vec<u8> {
        self.inner.encode_with_input(input.as_str())
    }
}
