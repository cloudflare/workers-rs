use super::{
    get_durable_object_class_names, get_durable_object_class_names_declaration,
    get_durable_object_class_names_exports,
};
use anyhow::Result;

const SHIM_TEMPLATE: &str = include_str!("../js/durable_objects_shim.js");

pub fn get_durable_objects_shim() -> Result<String> {
    let names = get_durable_object_class_names()?;

    if names.is_empty() {
        return Ok("const successfullyWrappedDONames = [];".to_string());
    }

    let mut shim = get_durable_object_class_names_declaration(&names)?;

    shim.push_str(SHIM_TEMPLATE);

    shim.push_str(&get_durable_object_class_names_exports(names));

    Ok(shim)
}
