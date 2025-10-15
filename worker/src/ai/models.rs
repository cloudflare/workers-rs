pub mod llama_4_scout_17b_16e_instruct;

pub mod scoped_chat {
    use serde::Serialize;

    #[derive(Default, Debug, Serialize)]
    #[serde(rename_all = "lowercase", untagged)]
    pub enum Role {
        #[default]
        User,
        Assistant,
        System,
        Tool,
        Any(String),
    }

    #[derive(Default, Debug, Serialize)]
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
