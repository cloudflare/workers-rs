use crate::{
    models::{AiTextGenerationInput, AiTextGenerationOutput},
    Model, StreamableModel,
};

pub struct Llama4Scout17b16eInstruct;

impl Model for Llama4Scout17b16eInstruct {
    const MODEL_NAME: &str = "@cf/meta/llama-4-scout-17b-16e-instruct";

    type Input = AiTextGenerationInput;

    type Output = AiTextGenerationOutput;
}

impl StreamableModel for Llama4Scout17b16eInstruct {}
