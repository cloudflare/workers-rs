use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct DurableObjectBinding {
    // pub name: String, // The binding name, e.g., "DO_RPC_SERVER" - not strictly needed for this task
    pub class_name: String, // The actual class name, e.g., "RPCServer"
}
