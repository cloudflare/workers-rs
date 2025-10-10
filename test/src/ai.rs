use worker::{
    models::llama_4_scout_17b_16e_instruct::Llama4Scout17b16eInstruct,
    worker_sys::AiTextGenerationInput, Env, Request, Response, Result,
};

use crate::SomeSharedData;

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
