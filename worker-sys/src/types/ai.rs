use js_sys::Promise;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=Ai)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Ai;

    #[wasm_bindgen(structural, method, js_class=Ai, js_name=run)]
    pub fn run(this: &Ai, model: &str, input: JsValue) -> Promise;
}

#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type AiTextGenerationInput;

    #[wasm_bindgen(constructor, js_class = Object)]
    pub fn new() -> AiTextGenerationInput;

    #[wasm_bindgen(method, setter = "prompt")]
    fn set_prompt_inner(this: &AiTextGenerationInput, prompt: &str);
    #[wasm_bindgen(method, getter = "prompt")]
    pub fn get_prompt(this: &AiTextGenerationInput) -> Option<String>;

    #[wasm_bindgen(method, setter = "raw")]
    fn set_raw_inner(this: &AiTextGenerationInput, raw: bool);
    #[wasm_bindgen(method, getter = "raw")]
    pub fn get_raw(this: &AiTextGenerationInput) -> Option<bool>;

    #[wasm_bindgen(method, setter = "max_tokens")]
    fn set_max_tokens_inner(this: &AiTextGenerationInput, max_tokens: u32);
    #[wasm_bindgen(method, getter = "max_tokens")]
    pub fn get_max_tokens(this: &AiTextGenerationInput) -> Option<u32>;

    #[wasm_bindgen(method, setter = "temperature")]
    fn set_temperature_inner(this: &AiTextGenerationInput, temperature: f32);
    #[wasm_bindgen(method, getter = "temperature")]
    pub fn get_temperature(this: &AiTextGenerationInput) -> Option<f32>;

    #[wasm_bindgen(method, setter = "top_p")]
    fn set_top_p_inner(this: &AiTextGenerationInput, top_p: f32);
    #[wasm_bindgen(method, getter = "top_p")]
    pub fn get_top_p(this: &AiTextGenerationInput) -> Option<f32>;

    #[wasm_bindgen(method, setter = "top_k")]
    fn set_top_k_inner(this: &AiTextGenerationInput, top_p: u32);
    #[wasm_bindgen(method, getter = "top_k")]
    pub fn get_top_k(this: &AiTextGenerationInput) -> Option<u32>;

    #[wasm_bindgen(method, setter = "seed")]
    fn set_seed_inner(this: &AiTextGenerationInput, seed: u64);
    #[wasm_bindgen(method, getter = "seed")]
    pub fn get_seed(this: &AiTextGenerationInput) -> Option<u64>;

    #[wasm_bindgen(method, setter = "repetition_penalty")]
    fn set_repetition_penalty_inner(this: &AiTextGenerationInput, repetition_penalty: f32);
    #[wasm_bindgen(method, getter = "repetition_penalty")]
    pub fn get_repetition_penalty(this: &AiTextGenerationInput) -> Option<f32>;

    #[wasm_bindgen(method, setter = "frequency_penalty")]
    fn set_frequency_penalty_inner(this: &AiTextGenerationInput, frequency_penalty: f32);
    #[wasm_bindgen(method, getter = "frequency_penalty")]
    pub fn get_frequency_penalty(this: &AiTextGenerationInput) -> Option<f32>;

    #[wasm_bindgen(method, setter = "presence_penalty")]
    fn set_presence_penalty_inner(this: &AiTextGenerationInput, presence_penalty: f32);
    #[wasm_bindgen(method, getter = "presence_penalty")]
    pub fn get_presence_penalty(this: &AiTextGenerationInput) -> Option<f32>;
}

impl AiTextGenerationInput {
    pub fn set_prompt(self, prompt: &str) -> Self {
        self.set_prompt_inner(prompt);
        self
    }

    pub fn set_raw(self, raw: bool) -> Self {
        self.set_raw_inner(raw);
        self
    }

    pub fn set_max_tokens(self, max_tokens: u32) -> Self {
        self.set_max_tokens_inner(max_tokens);
        self
    }

    pub fn set_temperature(self, temperature: f32) -> Self {
        self.set_temperature_inner(temperature);
        self
    }

    pub fn set_top_p(self, top_p: f32) -> Self {
        self.set_top_p_inner(top_p);
        self
    }

    pub fn set_top_k(self, top_k: u32) -> Self {
        self.set_top_k_inner(top_k);
        self
    }

    pub fn set_seed(self, seed: u64) -> Self {
        self.set_seed_inner(seed);
        self
    }

    pub fn set_repetition_penalty(self, repetition_penalty: f32) -> Self {
        self.set_repetition_penalty_inner(repetition_penalty);
        self
    }

    pub fn set_frequency_penalty(self, frequency_penalty: f32) -> Self {
        self.set_frequency_penalty_inner(frequency_penalty);
        self
    }

    pub fn set_presence_penalty(self, presence_penalty: f32) -> Self {
        self.set_presence_penalty_inner(presence_penalty);
        self
    }
}

#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type AiTextGenerationOutput;

    #[wasm_bindgen(constructor, js_class = Object)]
    pub fn new() -> AiTextGenerationOutput;

    #[wasm_bindgen(method, getter = "response")]
    pub fn get_response(this: &AiTextGenerationOutput) -> Option<String>;

}

impl From<AiTextGenerationOutput> for Vec<u8> {
    fn from(value: AiTextGenerationOutput) -> Self {
        value
            .get_response()
            .map(|text| text.into_bytes())
            .unwrap_or_default()
    }
}
