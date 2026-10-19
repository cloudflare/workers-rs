use super::SomeSharedData;
use crate::Env;
use ts_gen_worker::*;
use uuid::Uuid;

pub async fn handle_analytics_event(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let dataset = env.http_analytics();

    let request_id = Uuid::new_v4();
    let point = AnalyticsEngineDataPoint::builder()
        .indexes_with_str_slice(&["index1"])
        .blobs_with_js_value_slice(&[
            req.method().into(),
            js_sys::ArrayBuffer::from(&request_id.as_bytes()[..]).into(),
        ])
        .doubles(&[200.0])
        .build();
    dataset.write_data_point_with_event(&point)?;

    dataset.write_data_point_with_event(
        &AnalyticsEngineDataPoint::builder()
            .indexes_with_str_slice(&["index1"])
            .blobs_with_str_slice(&[req.method(), req.method()])
            .doubles(&[200.0])
            .build(),
    )?;

    Response::builder().ok("Events sent")
}
