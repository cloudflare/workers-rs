use crate::wasm_pack::install::Tool;
use anyhow::Result;
use serde::Deserialize;
const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
pub struct Krate {
    pub max_version: String,
}

#[derive(Debug, Deserialize)]
pub struct KrateResponse {
    #[serde(rename = "crate")]
    pub krate: Krate,
}

impl Krate {
    pub fn new(name: &Tool) -> Result<Krate> {
        let krate_address = format!("https://crates.io/api/v1/crates/{}", name);
        let mut config = ureq::Agent::config_builder();
        if let Some(proxy) = ureq::Proxy::try_from_env() {
            config = config.proxy(Some(proxy));
        }
        let agent = config.build().new_agent();
        let mut res = agent
            .get(&krate_address)
            .header(
                "user-agent",
                &format!("wasm-pack/{}", VERSION.unwrap_or("unknown")),
            )
            .call()?;

        let kr: KrateResponse = res.body_mut().read_json()?;
        Ok(kr.krate)
    }
}
