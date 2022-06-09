use crate::error::Error;
use worker_sys::{
    TextDecodeOptions as EdgeTextDecodeOptions, TextDecoder as EdgeTextDecoder,
    TextDecoderOptions as EdgeTextDecoderOptions,
};

/// A [TextDecoder](https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder) 
/// represents a decoder for a specific text encoding, such as UTF-8, ISO-8859-2, KOI8-R, GBK, etc.
/// A decoder takes a stream of bytes as input and emits a stream of code points.
/// TODO: Finish docs
pub struct TextDecoder {
    inner: EdgeTextDecoder,
}

impl TextDecoder {

    /// Returns a newly constructed TextDecoder that will generate a code point stream with the default
    /// decoding label (UTF-8).
    pub fn new() -> Self {
        Self {
            inner: EdgeTextDecoder::new().unwrap(),
        }
    }

    /// Returns a newly constructed TextDecoder that will generate a code point stream with the given
    /// decoding [label](https://developer.mozilla.org/en-US/docs/Web/API/Encoding_API/Encodings).
    pub fn with_label(label: String) -> Result<Self, Error> {
        let edge_text_decoder = EdgeTextDecoder::new_with_label(label.as_str());
        match edge_text_decoder {
            Ok(val) => Ok(Self { inner: val }),
            Err(js_err) => Err(Error::from(js_err)),
        }
    }

    /// Returns a newly constructed TextDecoder that will generate a code point stream with the given
    /// decoding [label](https://developer.mozilla.org/en-US/docs/Web/API/Encoding_API/Encodings) and options.
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

    /// Returns a string containing the name of the decoding algorithm used by the specific decoder.
    pub fn encoding(&self) -> String {
        self.inner.encoding()
    }

    /// Returns a string containing the decoded text.
    pub fn decode(&self) -> Result<String, Error> {
        match self.inner.decode() {
            Ok(val) => Ok(val),
            Err(js_err) => Err(Error::from(js_err)),
        }
    }

    /// Returns a string containing the decoded text of the given encoded input.
    pub fn decode_with_input(&self, input: &mut [u8]) -> Result<String, Error> {
        match self.inner.decode_with_u8_array(input) {
            Ok(val) => Ok(val),
            Err(js_err) => Err(Error::from(js_err)),
        }
    }

    /// Returns a string containing the decoded text of the given encoded input.
    pub fn decode_with_input_u16(&self, input: &mut [u16]) -> Result<String, Error> {
        match self.inner.decode_with_u16_array(input) {
            Ok(val) => Ok(val),
            Err(js_err) => Err(Error::from(js_err)),
        }
    }

    /// Returns a string containing the decoded text of the given encoded input and options.
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

/// A struct with a boolean flag indicating if the TextDecoder.decode() method must throw a TypeError 
/// when an coding error is found. It defaults to false.
pub struct TextDecoderOptions {
    inner: EdgeTextDecoderOptions,
}

impl TextDecoderOptions {
    /// Returns a newly constructed TextDecoderOptions with the given fatal value.
    pub fn new(fatal: bool) -> Self {
        let mut ret = EdgeTextDecoderOptions::default();
        Self {
            inner: ret.fatal(fatal).clone(),
        }
    }
}

/// A struct with a boolean flag indicating that additional data will follow in subsequent calls to decode(). 
/// Set to true if processing the data in chunks, and false for the final chunk or if the data is not chunked. It defaults to false.
pub struct TextDecodeOptions {
    inner: EdgeTextDecodeOptions,
}

impl TextDecodeOptions {
    /// Returns a newly constructed TextDecodeOptions with the given stream value.
    pub fn new(stream: bool) -> Self {
        let mut ret = EdgeTextDecodeOptions::default();
        Self {
            inner: ret.stream(stream).clone(),
        }
    }
}
