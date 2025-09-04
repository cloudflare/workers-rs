use std::fmt;

/// Represents the set of CLI tools wasm-pack uses
pub enum Tool {
    /// wasm-bindgen CLI tools
    WasmBindgen,
    /// wasm-opt CLI tool
    WasmOpt,
}

impl fmt::Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Tool::WasmBindgen => "wasm-bindgen",
            Tool::WasmOpt => "wasm-opt",
        };
        write!(f, "{}", s)
    }
}
