use super::SomeSharedData;
use uuid::Uuid;
use worker::{AnalyticsEngineDataPointBuilder, Env, Request, Response, Result};

#[worker::send]
pub async fn handle_analytics_event(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let dataset = match env.analytics_engine("HTTP_ANALYTICS") {
        Ok(dataset) => dataset,
        Err(err) => return Response::error(format!("Failed to get dataset: {err:?}"), 500),
    };

    let request_id = Uuid::new_v4();
    // Build the event and write it to analytics engine
    let point = AnalyticsEngineDataPointBuilder::new()
        .indexes(vec!["index1"].as_slice())
        .add_blob(req.method().as_ref()) // blob1
        .add_blob(request_id.as_bytes().as_ref()) // blob2
        .add_double(200)
        .build();
    dataset.write_data_point(&point)?;

    // Or write it directly from the builder using write_to
    AnalyticsEngineDataPointBuilder::new()
        .indexes(vec!["index1"].as_slice())
        .add_blob(req.method().as_ref()) // blob1
        .add_blob(req.method().as_ref()) // blob2
        .add_double(200)
        .write_to(&dataset)?;

    return Response::ok("Events sent");
}
