//! Example: semantic search with Vectorize + Workers AI embeddings.
//!
//! Embeds text with the `@cf/baai/bge-base-en-v1.5` model (768-dim) through the
//! AI binding, stores the vectors in a Vectorize index with the source text as
//! metadata, and answers natural-language queries by embedding the query and
//! searching the index, the retrieval half of a RAG pipeline.
//!
//! Both Workers AI and Vectorize run remotely (Vectorize has no local
//! emulator):
//!
//! ```sh
//! wrangler vectorize create demo-index --dimensions=768 --metric=cosine
//! # Filtering needs a metadata index on the property, created before inserting:
//! wrangler vectorize create-metadata-index demo-index --property-name=category --type=string
//! wrangler dev
//! curl -X POST localhost:8787/insert
//! curl "localhost:8787/query?q=famous+towers+in+Europe"
//! curl "localhost:8787/query?q=tall+places&category=geography"
//! ```

use serde::{Deserialize, Serialize};
use serde_json::{json, Map};
use worker::{
    event,
    vectorize::{
        VectorizeMetadataRetrievalLevel, VectorizeQueryOptions, VectorizeVector,
        VectorizeVectorMetadataFilter, VectorizeVectorMetadataFilterCollectionOp,
    },
    Context, Env, Request, Response, Result, RouteContext, Router,
};

/// Text-embedding model; produces the 768-dim vectors the index is sized for.
const EMBEDDING_MODEL: &str = "@cf/baai/bge-base-en-v1.5";

/// Sample corpus: `(id, category, text)`.
const CORPUS: &[(&str, &str, &str)] = &[
    (
        "eiffel",
        "landmark",
        "The Eiffel Tower is a wrought-iron lattice tower in Paris, France.",
    ),
    (
        "colosseum",
        "landmark",
        "The Colosseum is an ancient amphitheatre in the centre of Rome, Italy.",
    ),
    (
        "everest",
        "geography",
        "Mount Everest is Earth's highest mountain above sea level.",
    ),
    (
        "pacific",
        "geography",
        "The Pacific Ocean is the largest and deepest of Earth's oceans.",
    ),
];

#[derive(Serialize)]
struct EmbeddingRequest {
    text: Vec<String>,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    /// One embedding vector per input text.
    data: Vec<Vec<f32>>,
}

#[derive(Serialize, Deserialize)]
struct DocMetadata {
    text: String,
    category: String,
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .post_async("/insert", insert)
        .get_async("/query", query)
        .get_async("/describe", describe)
        .run(req, env)
        .await
}

/// Embed the corpus with Workers AI and upsert it into the index, keeping the
/// source text and category as metadata. Mutations are async, so the returned
/// id identifies the changeset rather than a completed write.
async fn insert(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let ai = ctx.env.ai("AI")?;
    let index = ctx.env.vectorize("VECTORIZE")?;

    let text = CORPUS.iter().map(|&(_, _, t)| t.to_string()).collect();
    let embeddings: EmbeddingResponse = ai.run(EMBEDDING_MODEL, EmbeddingRequest { text }).await?;

    let mut vectors = Vec::with_capacity(CORPUS.len());
    for (&(id, category, text), embedding) in CORPUS.iter().zip(&embeddings.data) {
        let vector = VectorizeVector::from_f32(id, embedding);
        vector.set_metadata_from(&DocMetadata {
            text: text.to_string(),
            category: category.to_string(),
        })?;
        vectors.push(vector);
    }

    let mutation = index.upsert(&vectors).await?;
    Response::ok(format!("queued mutation {}", mutation.mutation_id()))
}

/// Embed the query text and return the closest documents.
///
/// `GET /query?q=<text>&category=<a,b>`. The optional `category` filter is
/// built with the typed `$in` operator.
async fn query(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let ai = ctx.env.ai("AI")?;
    let index = ctx.env.vectorize("VECTORIZE")?;

    let url = req.url()?;
    let question = url
        .query_pairs()
        .find(|(k, _)| k == "q")
        .map(|(_, v)| v.into_owned())
        .unwrap_or_else(|| "famous towers in Europe".to_string());

    let embeddings: EmbeddingResponse = ai
        .run(
            EMBEDDING_MODEL,
            EmbeddingRequest {
                text: vec![question.clone()],
            },
        )
        .await?;
    let query_vector = embeddings.data.into_iter().next().unwrap_or_default();

    let mut options = VectorizeQueryOptions::builder()
        .top_k(3.0)
        .return_metadata_with_vectorize_metadata_retrieval_level(
            VectorizeMetadataRetrievalLevel::All,
        );

    // Optional `?category=landmark,geography`
    if let Some((_, categories)) = url.query_pairs().find(|(k, _)| k == "category") {
        let mut clause = Map::new();
        clause.insert(
            VectorizeVectorMetadataFilterCollectionOp::In
                .as_str()
                .to_string(),
            json!(categories.split(',').collect::<Vec<_>>()),
        );
        let filter = VectorizeVectorMetadataFilter::from_serde(&json!({ "category": clause }))?;
        options = options.filter(&filter);
    }

    let results = index.query_f32(&query_vector, &options.build()).await?;

    let hits: Vec<_> = results
        .matches()
        .into_iter()
        .map(|hit| {
            let metadata: Option<DocMetadata> = hit.metadata_into().ok().flatten();
            json!({
                "id": hit.id(),
                "score": hit.score(),
                "text": metadata.map(|m| m.text),
            })
        })
        .collect();

    Response::from_json(&json!({ "query": question, "matches": hits }))
}

/// Report the index dimensions and current vector count.
async fn describe(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let info = ctx.env.vectorize("VECTORIZE")?.describe().await?;
    Response::from_json(&json!({
        "dimensions": info.dimensions(),
        "vectorCount": info.vector_count(),
    }))
}
