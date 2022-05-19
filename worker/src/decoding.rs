use crate::error::Error;
use worker_sys::{
    TextDecodeOptions as EdgeTextDecodeOptions, TextDecoder as EdgeTextDecoder,
    TextDecoderOptions as EdgeTextDecoderOptions,
};

/// A [TextDecoder](https://developer.mozilla.org/en-US/docs/Web/API/FormData) representation of the
/// request body, providing access to form encoded fields and files.
/// TODO: Finish docs
pub struct TextDecoder {
    inner: EdgeTextDecoder,
}

impl TextDecoder {
    pub fn new() -> Self {
        Self {
            inner: EdgeTextDecoder::new().unwrap(),
        }
    }

    pub fn with_label(label: String) -> Result<Self, Error> {
        let edge_text_decoder = EdgeTextDecoder::new_with_label(label.as_str());
        match edge_text_decoder {
            Ok(val) => Ok(Self { inner: val }),
            Err(js_err) => Err(Error::from(js_err)),
        }
    }

    pub fn with_label_and_options(
        label: String,
        options: TextDecoderOptions,
    ) -> Result<Self, Error> {
        let edge_text_decoder =
            EdgeTextDecoder::new_with_label_and_options(label.as_str(), &options.inner);
        match edge_text_decoder {
            Ok(val) => Ok(Self { inner: val }),
            Err(js_err) => Err(Error::from(js_err)),
        }
    }

    pub fn encoding(&self) -> String {
        self.inner.encoding()
    }

    pub fn decode(&self) -> Result<String, Error> {
        match self.inner.decode() {
            Ok(val) => Ok(val),
            Err(js_err) => Err(Error::from(js_err)),
        }
    }

    pub fn decode_with_input(&self, input: &mut [u8]) -> Result<String, Error> {
        match self.inner.decode_with_u8_array(input) {
            Ok(val) => Ok(val),
            Err(js_err) => Err(Error::from(js_err)),
        }
    }

    pub fn decode_with_input_and_options(
        &self,
        input: &mut [u8],
        options: TextDecodeOptions,
    ) -> Result<String, Error> {
        match self
            .inner
            .decode_with_u8_array_and_options(input, &options.inner)
        {
            Ok(val) => Ok(val),
            Err(js_err) => Err(Error::from(js_err)),
        }
    }
}

pub struct TextDecoderOptions {
    inner: EdgeTextDecoderOptions,
}

impl TextDecoderOptions {
    pub fn new(fatal: bool) -> Self {
        let mut ret = EdgeTextDecoderOptions::default();
        Self {
            inner: ret.fatal(fatal).clone(),
        }
    }
}

pub struct TextDecodeOptions {
    inner: EdgeTextDecodeOptions,
}

impl TextDecodeOptions {
    pub fn new(stream: bool) -> Self {
        let mut ret = EdgeTextDecodeOptions::default();
        Self {
            inner: ret.stream(stream).clone(),
        }
    }
}
