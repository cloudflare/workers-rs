use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{Headers, RequestInit, Result};
use flate2::read::GzDecoder;
#[cfg(feature = "http")]
use std::convert::TryInto;

#[derive(Serialize)]
pub struct TextEmbeddingInput {
    pub text: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct TextEmbeddingOutput {
    pub shape: Vec<usize>,
    pub data: Vec<Vec<f64>>,
}

pub struct TextEmbeddingModel;

impl ModelKind for TextEmbeddingModel {
    type Input = TextEmbeddingInput;
    type Output = TextEmbeddingOutput;
}

pub trait ModelKind {
    type Input: Serialize;
    type Output: DeserializeOwned;
}

/// Object to interact with Workers AI binding.
///
/// ```rust
/// let ai = env.ai("AI")?;
///
/// let input = TextEmbeddingInput {
///     text: "This is a test embedding".to_owned(),
/// };
///
/// let result = ai
///     .run::<TextEmbeddingModel>("@cf/baai/bge-base-en-v1.5", input)
///     .await?;
/// ```
pub struct Ai {
    inner: crate::Fetcher,
}

#[derive(Serialize)]
struct AiRequestOptions {
    debug: bool,
}

#[derive(Serialize)]
struct AiRequest<Input: serde::Serialize> {
    inputs: Input,
    options: AiRequestOptions,
}

impl Ai {
    pub(crate) fn new(inner: crate::Fetcher) -> Self {
        Self { inner }
    }

    pub async fn run<M>(&self, model: &str, inputs: M::Input) -> Result<M::Output>
    where
        M: ModelKind,
    {
        use std::io::prelude::*;
        let request = AiRequest::<M::Input> {
            inputs,
            options: AiRequestOptions { debug: false },
        };
        let payload = serde_json::to_string(&request)?;
        let mut init = RequestInit::new();
        init.with_body(Some(payload.into()));
        let mut headers = Headers::new();
        headers.append("content-encoding", "application/json")?;
        headers.append("cf-consn-model-id", model)?;
        init.with_headers(headers);
        init.with_method(crate::Method::Post);
        let response = self
            .inner
            .fetch("http://workers-binding.ai/run?version=2", Some(init))
            .await?;

        #[cfg(feature = "http")]
        let mut resp: crate::Response = response.try_into()?;
        #[cfg(not(feature = "http"))]
        let mut resp = response;

        let data = resp.bytes().await?.to_owned();
        let mut d = GzDecoder::new(&data[..]);
        let mut text = String::new();
        d.read_to_string(&mut text).unwrap();
        let body: M::Output = serde_json::from_str(&text)?;
        Ok(body)
    }
}
