# Vectorize example

Semantic search over a small corpus using [Cloudflare Vectorize](https://developers.cloudflare.com/vectorize/) and [Workers AI](https://developers.cloudflare.com/workers-ai/) from a Rust Cloudflare Worker. It's the retrieval half of a [RAG](https://developers.cloudflare.com/workers-ai/guides/tutorials/build-a-retrieval-augmented-generation-ai/) pipeline.

The Worker embeds text with the `@cf/baai/bge-base-en-v1.5` model (768-dim) through the `worker::Ai` binding and stores it in a Vectorize index, keeping the source text as metadata. To search, it embeds the natural-language query and looks for the nearest vectors.

## Routes

| Route | Description |
|---|---|
| `POST /insert` | Embeds the built-in corpus and upserts it, keeping `{ text, category }` as metadata. Returns the async mutation id. |
| `GET /query?q=<text>&category=<a,b>` | Embeds `q`, returns the `topK` nearest documents with full metadata. The optional `category` filter uses the typed `$in` operator. |
| `GET /describe` | Returns the index dimensions and current vector count. |

## Setup

Both Workers AI and Vectorize run remotely (Vectorize has no local emulator), so this example must run with `wrangler dev`.

1. Create a 768-dimensional index. The dimensions must match the embedding model:

   ```sh
   npx wrangler vectorize create demo-index --dimensions=768 --metric=cosine
   ```

   The name must match `index_name` under `[[vectorize]]` in `wrangler.toml`.

2. Create a metadata index on `category` so the `?category=` filter works.
   Vectorize only filters on properties that have a metadata index, and it only
   indexes vectors written after the index exists, so create it before
   inserting:

   ```sh
   npx wrangler vectorize create-metadata-index demo-index --property-name=category --type=string
   ```

3. Start the Worker against the live index + AI:

   ```sh
   npx wrangler dev
   ```

## Testing

```sh
# Embed and store the corpus (mutations are async — give it a moment to process)
curl -X POST localhost:8787/insert

# Semantic search
curl "localhost:8787/query?q=famous+towers+in+Europe"

# Restrict to a metadata category (typed `$in` operator)
curl "localhost:8787/query?q=tall+places&category=geography"

# Index stats
curl localhost:8787/describe
```

> Vectorize processes mutations (`insert` / `upsert` / `deleteByIds`) asynchronously.
> The response carries a `mutationId` that identifies the changeset rather than a
> completed write, so a `query` run right after an `insert` may not see the new
> vectors yet.
