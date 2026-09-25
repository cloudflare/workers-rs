//! Ergonomics layered over the ts-gen-generated Vectorize bindings.
//!
//! The raw `#[wasm_bindgen]` surface (the [`Vectorize`] index handle plus the
//! [`VectorizeVector`], [`VectorizeQueryOptions`], [`VectorizeMatches`] types
//! and their builders) is generated from `types/vectorize.d.ts` into
//! [`crate::bindings::vectorize`] and re-exported here unchanged. This module
//! adds only what ts-gen can't infer:
//!
//! * the [`EnvBinding`] impl that backs [`Env::vectorize`](crate::Env::vectorize), and
//! * a handful of `f32`/serde conveniences. Embeddings are almost always
//!   `f32`, while the generated `number[]` overloads land on `f64`, and the
//!   freeform metadata/filter types come through as opaque JS objects.

use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::{JsCast, JsValue};

pub use crate::bindings::vectorize::*;
use crate::{env::EnvBinding, Result};

impl EnvBinding for Vectorize {
    const TYPE_NAME: &'static str = "VectorizeIndexImpl";

    // The runtime hands back an instance of workerd's internal
    // `VectorizeIndexImpl` class, but that name is an implementation detail
    // that has moved across Vectorize versions (the beta exposed
    // `VectorizeIndex`). As with `SendEmail`, we trust the object the runtime
    // provides for the configured binding and skip the `constructor.name`
    // check the default `EnvBinding::get` performs.
    fn get(val: JsValue) -> Result<Self> {
        Ok(val.unchecked_into())
    }
}

impl Vectorize {
    /// Similarity search using an `f32` embedding, the representation almost
    /// every model emits.
    ///
    /// Convenience over [`Vectorize::query_with_float32_array_and_options`]
    /// that converts the slice to a `Float32Array` for you. Build `options`
    /// with [`VectorizeQueryOptions::builder`] (or pass
    /// `&VectorizeQueryOptions::new()` for defaults).
    pub async fn query_f32(
        &self,
        vector: &[f32],
        options: &VectorizeQueryOptions,
    ) -> Result<VectorizeMatches> {
        let array = js_sys::Float32Array::from(vector);
        Ok(self
            .query_with_float32_array_and_options(&array, options)
            .await?)
    }
}

impl VectorizeVector {
    /// Build a vector from an `f32` embedding slice, the `f32` counterpart to
    /// the generated [`VectorizeVector::new_with_slice`] (which takes `&[f64]`).
    pub fn from_f32(id: &str, values: &[f32]) -> Self {
        Self::new(id, &js_sys::Float32Array::from(values))
    }

    /// Attach metadata serialized from any [`Serialize`] value, rather than
    /// hand-building the underlying JS object.
    pub fn set_metadata_from<T: Serialize>(&self, metadata: &T) -> Result<()> {
        self.set_metadata(&to_js_object(metadata)?.unchecked_into());
        Ok(())
    }
}

impl VectorizeMatch {
    /// Deserialize this match's metadata into a typed value, when the query
    /// requested metadata (`returnMetadata`).
    pub fn metadata_into<T: DeserializeOwned>(&self) -> Result<Option<T>> {
        match self.metadata() {
            Some(obj) => Ok(Some(serde_wasm_bindgen::from_value(obj.into())?)),
            None => Ok(None),
        }
    }
}

impl VectorizeVectorMetadataFilter {
    /// Build a metadata filter from any [`Serialize`] value (e.g. a
    /// `serde_json::json!({ "genre": { "$in": ["comedy", "drama"] } })`).
    ///
    /// The upstream filter type is a TypeScript mapped type, which ts-gen
    /// can't translate into fields, so it surfaces as an opaque JS object;
    /// this is the ergonomic way to populate it. The operator keys can be
    /// written as string literals or via [`VectorizeVectorMetadataFilterOp::as_str`]
    /// / [`VectorizeVectorMetadataFilterCollectionOp::as_str`].
    pub fn from_serde<T: Serialize>(filter: &T) -> Result<Self> {
        Ok(to_js_object(filter)?.unchecked_into())
    }
}

/// Serialize a value to a JS value, rendering Rust maps (`serde_json::Map`,
/// `HashMap`, etc.) as plain objects rather than `Map`s.
///
/// `serde_wasm_bindgen::to_value` defaults to JS `Map`s for map types, but
/// Vectorize metadata and filters must be plain objects. A `Map`-shaped filter
/// is silently ignored by the runtime (the query returns unfiltered results).
/// Structs are unaffected (they always serialize to objects); this only matters
/// for `json!(...)` / map inputs.
fn to_js_object<T: Serialize>(value: &T) -> Result<JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    Ok(value.serialize(&serializer)?)
}

impl VectorizeVectorMetadataFilterOp {
    /// The operator as its Vectorize filter key (e.g. `"$eq"`).
    ///
    /// wasm-bindgen emits a `to_str` for string enums but keeps it private, so
    /// we re-expose the mapping here for building filters with typed operators.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Eq => "$eq",
            Self::Ne => "$ne",
            Self::Lt => "$lt",
            Self::Lte => "$lte",
            Self::Gt => "$gt",
            Self::Gte => "$gte",
            // `__Invalid` (wasm-bindgen's hidden catch-all for unknown JS
            // strings) is unreachable for a locally-constructed operator.
            _ => "",
        }
    }
}

impl VectorizeVectorMetadataFilterCollectionOp {
    /// The collection operator as its Vectorize filter key (`"$in"` / `"$nin"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::In => "$in",
            Self::Nin => "$nin",
            _ => "",
        }
    }
}

#[cfg(test)]
mod send_check {
    // `Vectorize` is a raw `#[wasm_bindgen]` extern type, which wasm-bindgen
    // makes `Send + Sync`. This compile-time check guards against an upstream
    // regression, since the binding is awaited across `#[worker::send]` boundaries.
    use super::Vectorize;
    fn _assert_send<T: Send>() {}
    #[allow(dead_code)]
    fn _check() {
        _assert_send::<Vectorize>();
    }
}
