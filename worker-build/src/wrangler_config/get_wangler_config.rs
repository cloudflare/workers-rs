use super::get_wrangler_config_from_toml;
use crate::wrangler_config::WranglerConfig;
use anyhow::Result;
use std::env::current_dir;

pub fn get_wrangler_config() -> Result<WranglerConfig> {
    let root = current_dir()?;

    let toml = root.join("wrangler.toml");
    if toml.exists() {
        get_wrangler_config_from_toml(&toml)
    } else {
        println!("[worker-build] wrangler.toml not found at {:?}", root);
        Ok(WranglerConfig::default())
    }
}
