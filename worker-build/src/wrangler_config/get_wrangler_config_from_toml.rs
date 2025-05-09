use crate::wrangler_config::WranglerConfig;
use anyhow::{anyhow, Result};
use std::{fs::read_to_string, path::Path};

pub fn get_wrangler_config_from_toml(path: &Path) -> Result<WranglerConfig> {
    let content = read_to_string(&path)
        .map_err(|e| anyhow!("Failed to read wrangler.toml at {:?}: {}", path, e))?;

    let config: WranglerConfig =
        toml::from_str(&content).map_err(|e| anyhow!("Failed to parse wrangler.toml: {}", e))?;

    Ok(config)
}
