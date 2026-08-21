use crate::SomeSharedData;
use serde_json::json;
use worker::{
    vectorize::{
        VectorizeMetadataRetrievalLevel, VectorizeQueryOptions, VectorizeVector,
        VectorizeVectorMetadataFilter,
    },
    Env, Request, Response, Result,
};

/// Dimensionality of the bound `test-index`. Must match the index created with
/// `wrangler vectorize create test-index --dimensions=768`.
const INDEX_DIMENSIONS: usize = 768;

/// A deterministic 768-dim vector, so the handler runs against a real index
/// without needing an embedding model. Real workers embed text via Workers AI;
/// see `examples/vectorize`.
fn demo_vector(seed: f32) -> Vec<f32> {
    (0..INDEX_DIMENSIONS)
        .map(|i| ((i as f32 + seed) * 0.001).sin())
        .collect()
}

// Exercises the full Vectorize binding surface so it stays compiling against
// real worker code (including the `#[worker::send]` async path). Vectorize has
// no local Miniflare emulator, so this isn't wired into a vitest spec. Pointed
// at a real 768-dim index (`remote = true` in wrangler.toml) it returns the JSON
// summary below; otherwise each step reports its error instead of panicking.
#[worker::send]
pub async fn handle_vectorize(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let index = match env.vectorize("VECTORIZE") {
        Ok(index) => index,
        Err(err) => return Response::error(format!("no Vectorize binding: {err}"), 500),
    };

    // Build vectors from `f32` embeddings with typed metadata. `doc-2` goes
    // through the generated `&[f64]` slice builder to cover that path too.
    let first = VectorizeVector::from_f32("doc-1", &demo_vector(1.0));
    first.set_metadata_from(&json!({ "genre": "comedy", "year": 2021 }))?;
    let second_values: Vec<f64> = demo_vector(2.0).into_iter().map(f64::from).collect();
    let second = VectorizeVector::builder_with_slice("doc-2", &second_values)
        .namespace("library")
        .build();

    let insert = index.insert(&[first, second]).await;
    let upsert = index
        .upsert(&[VectorizeVector::from_f32("doc-1", &demo_vector(1.5))])
        .await;

    // Query by vector with the full option surface, including a metadata filter
    // and the typed retrieval level (passed by value).
    let options = VectorizeQueryOptions::builder()
        .top_k(3.0)
        .namespace("library")
        .return_values(true)
        .return_metadata_with_vectorize_metadata_retrieval_level(
            VectorizeMetadataRetrievalLevel::All,
        )
        .filter(&VectorizeVectorMetadataFilter::from_serde(
            &json!({ "genre": { "$in": ["comedy", "drama"] } }),
        )?)
        .build();
    let matches = index.query_f32(&demo_vector(1.0), &options).await;

    let scored: Vec<_> = matches
        .as_ref()
        .map(|m| {
            m.matches()
                .into_iter()
                .map(|hit| {
                    let metadata: Option<serde_json::Value> = hit.metadata_into().ok().flatten();
                    json!({ "id": hit.id(), "score": hit.score(), "metadata": metadata })
                })
                .collect()
        })
        .unwrap_or_default();

    let ids = [String::from("doc-1"), String::from("doc-2")];
    let by_id = index.query_by_id_with_options("doc-1", &options).await;
    let fetched = index.get_by_ids(&ids).await;
    let deleted = index.delete_by_ids(&ids[1..]).await;
    let described = index.describe().await;

    Response::from_json(&json!({
        "insert": insert.map(|m| m.mutation_id()).map_err(|e| e.to_string()).ok(),
        "upsert": upsert.map(|m| m.mutation_id()).map_err(|e| e.to_string()).ok(),
        "matchCount": matches.as_ref().map(|m| m.count()).unwrap_or(0.0),
        "matches": scored,
        "queryByIdCount": by_id.map(|m| m.count()).unwrap_or(0.0),
        "fetched": fetched.map(|arr| arr.length()).unwrap_or(0),
        "deleted": deleted.map(|m| m.mutation_id()).map_err(|e| e.to_string()).ok(),
        "dimensions": described.map(|info| info.dimensions()).unwrap_or(0.0),
    }))
}
