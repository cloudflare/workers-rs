use wasm_bindgen::prelude::wasm_bindgen;
use worker::*;

#[durable_object]
pub struct RPCServer {
    state: State,
    counter: u32,
}

#[wasm_bindgen]
impl RPCServer {
    #[wasm_bindgen]
    pub fn add(&mut self, a: u32, b: u32) -> u32 {
        console_error_panic_hook::set_once();
        self.counter += 1;
        a + b + self.counter
    }
}

#[durable_object]
impl DurableObject for RPCServer {
    fn new(state: State, _env: Env) -> Self {
        Self { state, counter: 0 }
    }

    // URL /{left}+{right}
    async fn fetch(&mut self, req: Request) -> Result<Response> {
        console_error_panic_hook::set_once();
        let url = req.url()?;
        let path = url.path();

        let path = path.trim_start_matches('/');

        let path_parts: Vec<&str> = path.split('+').collect();
        if path_parts.len() != 2 {
            return Response::error("Invalid URL", 400);
        }

        if let (Ok(left), Ok(right)) = (path_parts[0].parse::<u32>(), path_parts[1].parse::<u32>())
        {
            let result = self.add(left, right);
            Response::ok(&format!("{} + {} = {}", left, right, result))
        } else {
            return Response::error("Invalid operands", 400);
        }
    }
}
