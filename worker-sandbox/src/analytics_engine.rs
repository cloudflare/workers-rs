use super::SomeSharedData;
use worker::{
    AnalyticsEngineDataPoint, AnalyticsEngineDataPointBuilder, Env, Request, Response, Result,
};

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

    // String blobs
    let event: AnalyticsEngineDataPoint<String> = AnalyticsEngineDataPointBuilder::new()
        .indexes(vec!["http".into()])
        .blobs(vec![req.method().to_string()])
        .doubles(vec![200.into()])
        .build();
    dataset.write_data_point(&event)?;

    // Binary blobs
    let event: AnalyticsEngineDataPoint<Vec<u8>> = AnalyticsEngineDataPointBuilder::new()
        .indexes(vec!["http".into()])
        .blobs(vec![req.method().to_string().as_bytes().to_vec()])
        .doubles(vec![200.into()])
        .build();
    dataset.write_data_point(&event)?;

    return Response::ok("Events sent");
}
