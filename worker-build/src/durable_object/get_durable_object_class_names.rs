use crate::wrangler_config::{get_wrangler_config, WranglerConfig};
use anyhow::Result;

pub fn get_durable_object_class_names() -> Result<Vec<String>> {
    let config: WranglerConfig = get_wrangler_config()?;

    let mut names = Vec::new();

    if let Some(do_config) = config.durable_objects {
        for binding in do_config.bindings {
            if !binding.class_name.is_empty() {
                if !names.contains(&binding.class_name) {
                    names.push(binding.class_name);
                }
            }
        }
    }

    if !names.is_empty() {
        println!(
            "[worker-build] Found DO class names in wrangler.toml: {:?}",
            names
        );
    } else {
        println!("[worker-build] No Durable Object class names found in wrangler.toml [durable_objects].bindings");
    }

    Ok(names)
}
