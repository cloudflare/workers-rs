use super::get_durable_objects_shim;
use anyhow::Result;

pub fn inject_durable_objects_shim(shim: String) -> Result<String> {
    Ok(shim.replace(
        "$DURABLE_OBJECTS_INJECTION_POINT",
        &get_durable_objects_shim()?,
    ))
}
