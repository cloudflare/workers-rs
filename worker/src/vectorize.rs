use std::collections::HashMap;

use crate::{send::SendFuture, EnvBinding, Result};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::types::VectorizeIndex as VectorizeIndexSys;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Supported distance metrics for an index.
/// Distance metrics determine how other "similar" vectors are determined.
pub enum VectorizeDistanceMetric {
    Euclidean,
    Cosine,
    DotProduct,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
/// Information about the configuration of an index.
pub enum VectorizeIndexConfig {
    Preset {
        preset: String,
    },
    Custom {
        dimensions: u16,
        metric: VectorizeDistanceMetric,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Metadata about an existing index.
///
/// This type is exclusively for the Vectorize **beta** and will be deprecated once Vectorize RC is released.
pub struct VectorizeIndexDetails {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub config: VectorizeIndexConfig,
    pub vectors_count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Results of an operation that performed a mutation on a set of vectors.
/// Here, `ids` is a list of vectors that were successfully processed.
///
/// This type is exclusively for the Vectorize **beta** and will be deprecated once Vectorize RC is released.
pub struct VectorizeVectorMutation {
    /// List of ids of vectors that were successfully processed.
    pub ids: Vec<String>,
    /// Total count of the number of processed vectors.
    pub count: u64,
}

#[derive(Debug, Serialize)]
/// Represents a single vector value set along with its associated metadata.
pub struct VectorizeVector<'a> {
    /// The ID for the vector. This can be user-defined, and must be unique. It should uniquely identify the object, and is best set based on the ID of what the vector represents.
    id: String,
    /// The vector values.
    values: &'a [f32],
    /// The namespace this vector belongs to.
    namespace: Option<String>,
    /// Metadata associated with the vector. Includes the values of other fields and potentially additional details.
    metadata: serde_json::Map<String, serde_json::Value>,
}

impl<'a> VectorizeVector<'a> {
    pub fn new(id: &str, values: &'a [f32]) -> Self {
        Self {
            id: id.to_owned(),
            values,
            namespace: None,
            metadata: serde_json::Map::new(),
        }
    }

    pub fn with_namespace(mut self, namespace: String) -> Self {
        self.namespace = Some(namespace);
        self
    }

    pub fn with_metadata_entry<V: Serialize>(mut self, key: &str, value: V) -> Result<Self> {
        self.metadata
            .insert(key.to_owned(), serde_json::to_value(value)?);
        Ok(self)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Metadata return levels for a Vectorize query.
pub enum VectorizeMetadataRetrievalLevel {
    /// Full metadata for the vector return set, including all fields (including those un-indexed) without truncation. This is a more expensive retrieval, as it requires additional fetching & reading of un-indexed data.
    All,
    /// Return all metadata fields configured for indexing in the vector return set. This level of retrieval is "free" in that no additional overhead is incurred returning this data. However, note that indexed metadata is subject to truncation (especially for larger strings).
    Indexed,
    /// No indexed metadata will be returned.
    None,
}

#[derive(Debug, Serialize, Hash, PartialEq, Eq)]
/// Comparison logic/operation to use for metadata filtering.
///
/// This list is expected to grow as support for more operations are released.
pub enum VectorizeVectorMetadataFilterOp {
    #[serde(rename = "$eq")]
    Eq,
    #[serde(rename = "$ne")]
    Neq,
}

/// Filter criteria for vector metadata used to limit the retrieved query result set.
type VectorizeVectorMetadataFilter =
    HashMap<String, HashMap<VectorizeVectorMetadataFilterOp, serde_json::Value>>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VectorizeQueryOptions {
    // Default 3, max 20
    top_k: u8,
    namespace: Option<String>,
    /// Return vector values. Default `false`.
    return_values: bool,
    /// Return vector metadata. Default `false`.
    return_metadata: bool,
    /// Default `none`.
    filter: Option<VectorizeVectorMetadataFilter>,
}

impl VectorizeQueryOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_top_k(mut self, top_k: u8) -> Self {
        self.top_k = top_k;
        self
    }

    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_owned());
        self
    }

    pub fn with_return_values(mut self, return_values: bool) -> Self {
        self.return_values = return_values;
        self
    }

    pub fn with_return_metadata(mut self, return_metadata: bool) -> Self {
        self.return_metadata = return_metadata;
        self
    }

    pub fn with_filter_entry<T: Serialize>(
        mut self,
        key: &str,
        op: VectorizeVectorMetadataFilterOp,
        value: T,
    ) -> Result<Self> {
        let mut filter = self.filter.unwrap_or_default();
        let inner = filter.entry(key.to_owned()).or_default();
        inner.insert(op, serde_json::to_value(value)?);
        self.filter = Some(filter);
        Ok(self)
    }
}

impl Default for VectorizeQueryOptions {
    fn default() -> Self {
        Self {
            top_k: 3,
            namespace: None,
            return_values: false,
            return_metadata: false,
            filter: None,
        }
    }
}

#[derive(Debug, Deserialize)]
/// Represents a single vector value set along with its associated metadata.
pub struct VectorizeVectorResult {
    /// The ID for the vector. This can be user-defined, and must be unique. It should uniquely identify the object, and is best set based on the ID of what the vector represents.
    pub id: String,
    /// The vector values.
    pub values: Option<Vec<f32>>,
    /// Metadata associated with the vector. Includes the values of other fields and potentially additional details.
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct VectorizeMatchVector {
    #[serde(flatten)]
    pub vector: VectorizeVectorResult,
    /// The score or rank for similarity, when returned as a result
    pub score: Option<f64>,
}

#[derive(Debug, Deserialize)]
/// A set of matching {@link VectorizeMatch} for a particular query.
pub struct VectorizeMatches {
    pub matches: Vec<VectorizeMatchVector>,
    pub count: u64,
}

/// A Vectorize Vector Search Index for querying vectors/embeddings.
///
/// This type is exclusively for the Vectorize **beta** and will be deprecated once Vectorize RC is released.
pub struct VectorizeIndex(VectorizeIndexSys);

unsafe impl Send for VectorizeIndex {}
unsafe impl Sync for VectorizeIndex {}

impl EnvBinding for VectorizeIndex {
    const TYPE_NAME: &'static str = "VectorizeIndexImpl";
}

impl VectorizeIndex {
    /// Get information about the currently bound index.
    pub async fn describe(&self) -> Result<VectorizeIndexDetails> {
        let promise = self.0.describe()?;
        let fut = SendFuture::new(JsFuture::from(promise));
        let details = fut.await?;
        Ok(serde_wasm_bindgen::from_value(details)?)
    }

    /// Insert a list of vectors into the index dataset. If a provided id exists, an error will be thrown.
    pub async fn insert<'a>(
        &self,
        vectors: &[VectorizeVector<'a>],
    ) -> Result<VectorizeVectorMutation> {
        let promise = self
            .0
            .insert(serde_wasm_bindgen::to_value(&vectors)?.into())?;
        let fut = SendFuture::new(JsFuture::from(promise));
        let mutation = fut.await?;
        Ok(serde_wasm_bindgen::from_value(mutation)?)
    }

    /// Upsert a list of vectors into the index dataset. If a provided id exists, it will be replaced with the new values.
    pub async fn upsert<'a>(
        &self,
        vectors: &[VectorizeVector<'a>],
    ) -> Result<VectorizeVectorMutation> {
        let promise = self
            .0
            .upsert(serde_wasm_bindgen::to_value(&vectors)?.into())?;
        let fut = SendFuture::new(JsFuture::from(promise));
        let mutation = fut.await?;
        Ok(serde_wasm_bindgen::from_value(mutation)?)
    }

    /// Use the provided vector to perform a similarity search across the index.
    pub async fn query(
        &self,
        vector: &[f32],
        options: VectorizeQueryOptions,
    ) -> Result<VectorizeMatches> {
        let opts = serde_wasm_bindgen::to_value(&options)?;
        let promise = self.0.query(vector, opts.into())?;
        let fut = SendFuture::new(JsFuture::from(promise));
        let matches = fut.await?;
        Ok(serde_wasm_bindgen::from_value(matches)?)
    }

    /// Delete a list of vectors with a matching id.
    pub async fn delete_by_ids<'a, T>(&self, ids: T) -> Result<VectorizeVectorMutation>
    where
        T: IntoIterator<Item = &'a str>,
    {
        // TODO: Can we avoid this allocation?
        let ids: Vec<String> = ids.into_iter().map(|id| id.to_string()).collect();
        let arg = serde_wasm_bindgen::to_value(&ids)?;
        let promise = self.0.delete_by_ids(arg)?;
        let fut = SendFuture::new(JsFuture::from(promise));
        let mutation = fut.await?;
        Ok(serde_wasm_bindgen::from_value(mutation)?)
    }

    /// Get a list of vectors with a matching id.
    pub async fn get_by_ids<'a, T>(&self, ids: T) -> Result<Vec<VectorizeVectorResult>>
    where
        T: IntoIterator<Item = &'a str>,
    {
        let ids: Vec<String> = ids.into_iter().map(|id| id.to_string()).collect();
        let arg = serde_wasm_bindgen::to_value(&ids)?;
        let promise = self.0.get_by_ids(arg)?;
        let fut = SendFuture::new(JsFuture::from(promise));
        let vectors = fut.await?;
        Ok(serde_wasm_bindgen::from_value(vectors)?)
    }
}

impl JsCast for VectorizeIndex {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<VectorizeIndex>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self(val.into())
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl From<VectorizeIndex> for JsValue {
    fn from(index: VectorizeIndex) -> Self {
        JsValue::from(index.0)
    }
}

impl AsRef<JsValue> for VectorizeIndex {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}
