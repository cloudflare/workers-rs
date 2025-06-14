use super::DurableObjectBinding;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct DurableObjectsConfig {
    #[serde(default)] // Handles case where 'bindings' might be missing
    pub bindings: Vec<DurableObjectBinding>,
}
