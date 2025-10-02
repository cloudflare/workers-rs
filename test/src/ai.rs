use serde::{Deserialize, Serialize};
use worker::{Env, Model, Request, Response, Result, StreamableModel};

use crate::SomeSharedData;

pub struct Llama4Scout17b16eInstruct;

#[derive(Serialize)]
pub struct DefaultTextGenerationInput {
    pub prompt: String,
}

#[derive(Deserialize)]
pub struct DefaultTextGenerationOutput {
    pub response: String,
}

impl From<DefaultTextGenerationOutput> for Vec<u8> {
    fn from(value: DefaultTextGenerationOutput) -> Self {
        value.response.into_bytes()
    }
}

impl Model for Llama4Scout17b16eInstruct {
    const MODEL_NAME: &str = "@cf/meta/llama-4-scout-17b-16e-instruct";

    type Input = DefaultTextGenerationInput;

    type Output = DefaultTextGenerationOutput;
}

impl StreamableModel for Llama4Scout17b16eInstruct {}

const AI_TEST: &str = "AI_TEST";

#[worker::send]
pub async fn simple_ai_text_generation(
    _: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let ai = env
        .ai(AI_TEST)?
        .run::<Llama4Scout17b16eInstruct>(DefaultTextGenerationInput {
            prompt: "What is the answer to life the universe and everything?".to_owned(),
        })
        .await?;
    Response::ok(ai.response)
}

#[worker::send]
pub async fn streaming_ai_text_generation(
    _: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let stream = env
        .ai(AI_TEST)?
        .run_streaming::<Llama4Scout17b16eInstruct>(DefaultTextGenerationInput {
            prompt: "What is the answer to life the universe and everything?".to_owned(),
        })
        .await?;

    Response::from_stream(stream)
}
