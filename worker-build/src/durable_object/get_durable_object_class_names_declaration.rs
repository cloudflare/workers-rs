use anyhow::Result;

pub fn get_durable_object_class_names_declaration(names: &Vec<String>) -> Result<String> {
    Ok(format!(
        "const __WORKER_BUILD_DO_NAMES__ = [{}];",
        names
            .into_iter()
            .map(|n| format!("\"{}\"", n))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}
