use worker::{
    worker_sys::{AiTextGenerationInput, AiTextGenerationOutput},
    Env, Model, Request, Response, Result, StreamableModel,
};

use crate::SomeSharedData;

pub struct Llama4Scout17b16eInstruct;

impl Model for Llama4Scout17b16eInstruct {
    const MODEL_NAME: &str = "@cf/meta/llama-4-scout-17b-16e-instruct";

    type Input = AiTextGenerationInput;

    type Output = AiTextGenerationOutput;
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
        .run::<Llama4Scout17b16eInstruct>(
            AiTextGenerationInput::new()
                .set_prompt("What is the answer to life the universe and everything?"),
        )
        .await?;
    Response::ok(ai.get_response().unwrap_or_default())
}

#[worker::send]
pub async fn streaming_ai_text_generation(
    _: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let stream = env
        .ai(AI_TEST)?
        .run_streaming::<Llama4Scout17b16eInstruct>(
            AiTextGenerationInput::new()
                .set_prompt("What is the answer to life the universe and everything?"),
        )
        .await?;

    Response::from_stream(stream)
}
