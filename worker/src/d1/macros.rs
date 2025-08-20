/// **Requires** `d1` feature. Prepare a D1 query from the provided D1Database, query string, and optional query parameters.
///
/// Any parameter provided is required to implement [`serde::Serialize`] to be used.
///
/// Using [`query`](crate::query) is equivalent to using db.prepare('').bind('') in Javascript.
///
/// # Example
///
/// ```
/// let query = worker::query!(
///   &d1,
///   "SELECT * FROM things WHERE num > ?1 AND num < ?2",
///   &min,
///   &max,
/// )?;
/// ```
#[macro_export]
macro_rules! query {
    // rule for simple queries
    ($db:expr, $query:expr) => {
        $crate::d1::D1Database::prepare($db, $query)
    };
    // rule for parameterized queries
    ($db:expr, $query:expr, $($args:expr),* $(,)?) => {{
        || -> $crate::Result<$crate::d1::D1PreparedStatement> {
            let prepared = $crate::d1::D1Database::prepare($db, $query);

            // D1 doesn't support taking in undefined values, so we translate these missing values to NULL.
            let serializer = $crate::d1::serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true);
            let bindings = &[$(
                ::serde::ser::Serialize::serialize(&$args, &serializer)
                    .map_err(|e| $crate::Error::Internal(e.into()))?
            ),*];

            $crate::d1::D1PreparedStatement::bind(prepared, bindings)
        }()
    }};
}
