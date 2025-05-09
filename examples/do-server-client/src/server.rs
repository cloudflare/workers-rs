use worker::*;

#[durable_object]
pub struct Server {
    counter: u32,
}

#[durable_object]
impl DurableObject for Server {
    fn new(state: State, _env: Env) -> Self {
        Self { counter: 0 }
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
            let result = left + right + self.counter;
            self.counter += 1;
            Response::ok(&format!("{} + {} = {}", left, right, result))
        } else {
            return Response::error("Invalid operands", 400);
        }
    }
}
