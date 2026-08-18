// Regression test for https://github.com/cloudflare/workers-rs/issues/380.
// `Url::parse` returns `Result<Url, ParseError>`, and `Error` has a
// `From<url::ParseError>` impl, so the error type must be nameable through
// `::worker::` without `url` as a direct dependency.
use worker::{url::ParseError, Url};

fn parse(input: &str) -> Result<Url, ParseError> {
    Url::parse(input)
}

fn main() {
    let _ = parse("https://example.com");
}
