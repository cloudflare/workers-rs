use std::time::Duration;

use worker::*;

#[durable_object]
pub struct AlarmObject {
    state: State,
}

#[durable_object]
impl DurableObject for AlarmObject {
    fn new(state: State, _: Env) -> Self {
        Self { state }
    }

    async fn fetch(&mut self, _: http::Request<body::Body>) -> Result<http::Response<body::Body>> {
        let alarmed: bool = match self.state.storage().get("alarmed").await {
            Ok(alarmed) => alarmed,
            Err(e) if e.to_string() == "No such value in storage." => {
                // Trigger our alarm method in 100ms.
                self.state
                    .storage()
                    .set_alarm(Duration::from_millis(100))
                    .await?;

                false
            }
            Err(e) => return Err(e),
        };

        Ok(http::Response::new(alarmed.to_string().into()))
    }

    async fn alarm(&mut self) -> Result<http::Response<body::Body>> {
        self.state.storage().put("alarmed", true).await?;

        console_log!("Alarm has been triggered!");

        Ok(http::Response::new("ALARMED".into()))
    }
}
