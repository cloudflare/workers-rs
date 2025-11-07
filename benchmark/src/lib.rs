use worker::*;

#[event(fetch)]
async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();

    match path {
        "/stream" => handle_stream().await,
        "/benchmark" => handle_benchmark(&url).await,
        _ => Response::error("Not Found", 404),
    }
}

/// Streams 1MB of data in chunks
async fn handle_stream() -> Result<Response> {
    use futures_util::stream;

    // Create 1MB of data (1024 * 1024 bytes)
    let chunk_size = 8192; // 8KB chunks
    let num_chunks = (1024 * 1024) / chunk_size; // 128 chunks
    let chunk = vec![b'x'; chunk_size];

    // Create a stream that yields the data
    let data_stream = stream::iter((0..num_chunks).map(move |_| {
        Ok::<Vec<u8>, worker::Error>(chunk.clone())
    }));

    Response::from_stream(data_stream)
}

/// Main benchmark handler that makes 10 parallel sub-requests
async fn handle_benchmark(url: &Url) -> Result<Response> {
    // Get the base URL from the request
    let base_url = format!("{}://{}", url.scheme(), url.host_str().unwrap_or("localhost"));
    let stream_url = format!("{}/stream", base_url);

    // Create 10 parallel sub-requests
    let mut tasks = Vec::new();

    for i in 0..10 {
        let stream_url = stream_url.clone();

        // Create a task for each sub-request
        let task = async move {
            // Make the sub-request to the streaming endpoint
            let mut response = Fetch::Url(stream_url.parse().unwrap())
                .send()
                .await
                .map_err(|e| format!("Fetch error on request {}: {:?}", i, e))?;

            // Consume the stream to ensure all data is read
            let body = response.bytes().await
                .map_err(|e| format!("Body read error on request {}: {:?}", i, e))?;

            let total_bytes = body.len() as u64;

            Ok::<u64, String>(total_bytes)
        };

        tasks.push(task);
    }

    // Execute all tasks in parallel
    let start = Date::now().as_millis();
    let results = futures_util::future::join_all(tasks).await;
    let end = Date::now().as_millis();
    let duration_ms = end - start;

    // Check for errors and sum up total bytes
    let mut total_bytes = 0u64;
    let mut errors = Vec::new();

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(bytes) => total_bytes += bytes,
            Err(e) => errors.push(format!("Request {}: {}", i, e)),
        }
    }

    // Return summary as JSON
    let summary = serde_json::json!({
        "success": errors.is_empty(),
        "duration_ms": duration_ms,
        "total_bytes": total_bytes,
        "num_requests": 10,
        "errors": errors,
    });

    Response::from_json(&summary)
}
