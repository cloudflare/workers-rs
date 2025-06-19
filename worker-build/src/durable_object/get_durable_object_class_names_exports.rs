pub fn get_durable_object_class_names_exports(names: Vec<String>) -> String {
    let mut exports = String::new();

    let prefix = "__DO_WRAPPED_";

    for name in names {
        exports.push_str(&format!(
            "\nexport const {} = globalThis.{}{};",
            name, prefix, name
        ));
    }

    exports
}
