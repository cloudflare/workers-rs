use serde::{Deserialize, Serialize};

use crate::models::scoped_chat::RoleScopedChatInput;

pub mod llama_4_scout_17b_16e_instruct;

pub mod scoped_chat {
    use serde::Serialize;

    #[derive(Default, Serialize)]
    #[serde(rename_all = "lowercase", untagged)]
    pub enum Role {
        #[default]
        User,
        Assistant,
        System,
        Tool,
        Any(String),
    }

    #[derive(Default, Serialize)]
    pub struct RoleScopedChatInput {
        pub role: Role,
        pub content: String,
        pub name: Option<String>,
    }

    pub fn user(content: &str) -> RoleScopedChatInput {
        RoleScopedChatInput {
            role: Role::User,
            content: content.to_owned(),
            name: None,
        }
    }

    pub fn assistant(content: &str) -> RoleScopedChatInput {
        RoleScopedChatInput {
            role: Role::Assistant,
            content: content.to_owned(),
            name: None,
        }
    }

    pub fn system(content: &str) -> RoleScopedChatInput {
        RoleScopedChatInput {
            role: Role::System,
            content: content.to_owned(),
            name: None,
        }
    }

    pub fn tool(content: &str) -> RoleScopedChatInput {
        RoleScopedChatInput {
            role: Role::Tool,
            content: content.to_owned(),
            name: None,
        }
    }
}

/// Default input object for text generating Ai
///
/// The type implements default so you do not have to specify all fields.
///
/// like so
///# fn main() {
/// AiTextGenerationInput {
///  prompt: Some("What is the answer to life the universe and everything?".to_owned()),
///  ..default()
/// }
///# ;}
///
// TODO add response_json, tool calling and function calling to the input
#[derive(Default, Serialize)]
pub struct AiTextGenerationInput {
    pub prompt: Option<String>,
    pub raw: Option<bool>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub seed: Option<u32>,
    pub repetition_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub messages: Option<Vec<RoleScopedChatInput>>,
}

/// Default output object for text generating Ai
// TODO add tool call output support
#[derive(Default, Deserialize)]
pub struct UsageTags {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Default, Deserialize)]
pub struct AiTextGenerationOutput {
    pub response: Option<String>,
    pub usage: Option<UsageTags>,
}
