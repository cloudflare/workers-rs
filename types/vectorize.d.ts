/*
 * Vectorize (V2) types from @cloudflare/workers-types. Valid as of 22/06/2026.
 * This file builds worker/src/bindings/vectorize.rs as auto-generated bindings using ts-gen.
 *
 * NOTE: All hand edits to the @cloudflare/worker-types are marked with an "EDIT:" comment.
 *
 * EDIT: This is a V2-only subset of upstream's vectorize.d.ts. The deprecated
 * beta `VectorizeIndex` class and its beta-only companions
 * (`VectorizeIndexDetails`, `VectorizeVectorMutation`, `VectorizeIndexConfig`,
 * `VectorizeDistanceMetric`, `VectorizeError`) have been dropped, since the
 * current `Vectorize` binding never surfaces them and generating them would
 * only emit dead code.
 */

/**
 * Data types supported for holding vector metadata.
 */
type VectorizeVectorMetadataValue = string | number | boolean | string[];
/**
 * Additional information to associate with a vector.
 */
type VectorizeVectorMetadata =
  | VectorizeVectorMetadataValue
  | Record<string, VectorizeVectorMetadataValue>;

type VectorFloatArray = Float32Array | Float64Array;

/**
 * Comparison logic/operation to use for metadata filtering.
 *
 * This list is expected to grow as support for more operations are released.
 */
type VectorizeVectorMetadataFilterOp =
  | '$eq'
  | '$ne'
  | '$lt'
  | '$lte'
  | '$gt'
  | '$gte';
type VectorizeVectorMetadataFilterCollectionOp = '$in' | '$nin';

/**
 * Filter criteria for vector metadata used to limit the retrieved query result set.
 */
type VectorizeVectorMetadataFilter = {
  [field: string]:
    | Exclude<VectorizeVectorMetadataValue, string[]>
    | null
    | {
        [Op in VectorizeVectorMetadataFilterOp]?: Exclude<
          VectorizeVectorMetadataValue,
          string[]
        > | null;
      }
    | {
        [Op in VectorizeVectorMetadataFilterCollectionOp]?: Exclude<
          VectorizeVectorMetadataValue,
          string[]
        >[];
      };
};

/**
 * Metadata return levels for a Vectorize query.
 *
 * Default to "none".
 *
 * @property all      Full metadata for the vector return set, including all fields (including those un-indexed) without truncation. This is a more expensive retrieval, as it requires additional fetching & reading of un-indexed data.
 * @property indexed  Return all metadata fields configured for indexing in the vector return set. This level of retrieval is "free" in that no additional overhead is incurred returning this data. However, note that indexed metadata is subject to truncation (especially for larger strings).
 * @property none     No indexed metadata will be returned.
 */
type VectorizeMetadataRetrievalLevel = "all" | "indexed" | "none";

interface VectorizeQueryOptions {
  topK?: number;
  namespace?: string;
  returnValues?: boolean;
  returnMetadata?: boolean | VectorizeMetadataRetrievalLevel;
  filter?: VectorizeVectorMetadataFilter;
}

/**
 * Metadata about an existing index.
 */
interface VectorizeIndexInfo {
  /** The number of records containing vectors within the index. */
  vectorCount: number;
  /** Number of dimensions the index has been configured for. */
  dimensions: number;
  /** ISO 8601 datetime of the last processed mutation on in the index. All changes before this mutation will be reflected in the index state. */
  processedUpToDatetime: number;
  /** UUIDv4 of the last mutation processed by the index. All changes before this mutation will be reflected in the index state. */
  processedUpToMutation: number;
}

/**
 * Represents a single vector value set along with its associated metadata.
 */
interface VectorizeVector {
  /** The ID for the vector. This can be user-defined, and must be unique. It should uniquely identify the object, and is best set based on the ID of what the vector represents. */
  id: string;
  /** The vector values */
  values: VectorFloatArray | number[];
  /** The namespace this vector belongs to. */
  namespace?: string;
  /** Metadata associated with the vector. Includes the values of other fields and potentially additional details. */
  metadata?: Record<string, VectorizeVectorMetadata>;
}

// EDIT: upstream defines this as
//   type VectorizeMatch = Pick<Partial<VectorizeVector>, "values"> &
//     Omit<VectorizeVector, "values"> & { score: number };
// ts-gen can't resolve the Pick/Omit intersection into concrete fields, so it
// collapses the whole type to an opaque `JsValue` (and `VectorizeMatches.matches`
// to `Vec<JsValue>`). Rewritten here as a plain interface with the identical
// runtime shape so ts-gen emits a real struct with typed getters.
/**
 * Represents a matched vector for a query along with its score and (if specified) the matching vector information.
 */
interface VectorizeMatch {
  /** The ID for the vector. */
  id: string;
  /** The vector values, when `returnValues` was requested. */
  values?: VectorFloatArray | number[];
  /** The namespace this vector belongs to. */
  namespace?: string;
  /** Metadata associated with the vector, when `returnMetadata` was requested. */
  metadata?: Record<string, VectorizeVectorMetadata>;
  /** The score or rank for similarity, when returned as a result */
  score: number;
}

/**
 * A set of matching {@link VectorizeMatch} for a particular query.
 */
interface VectorizeMatches {
  matches: VectorizeMatch[];
  count: number;
}

/**
 * Result type indicating a mutation on the Vectorize Index.
 * Actual mutations are processed async where the `mutationId` is the unique identifier for the operation.
 */
interface VectorizeAsyncMutation {
  /** The unique identifier for the async mutation operation containing the changeset. */
  mutationId: string;
}

/**
 * A Vectorize Vector Search Index for querying vectors/embeddings.
 *
 * Mutations in this version are async, returning a mutation id.
 */
declare abstract class Vectorize {
  /**
   * Get information about the currently bound index.
   * @returns A promise that resolves with information about the current index.
   */
  public describe(): Promise<VectorizeIndexInfo>;
  /**
   * Use the provided vector to perform a similarity search across the index.
   * @param vector Input vector that will be used to drive the similarity search.
   * @param options Configuration options to massage the returned data.
   * @returns A promise that resolves with matched and scored vectors.
   */
  public query(
    vector: VectorFloatArray | number[],
    options?: VectorizeQueryOptions
  ): Promise<VectorizeMatches>;
  /**
   * Use the provided vector-id to perform a similarity search across the index.
   * @param vectorId Id for a vector in the index against which the index should be queried.
   * @param options Configuration options to massage the returned data.
   * @returns A promise that resolves with matched and scored vectors.
   */
  public queryById(
    vectorId: string,
    options?: VectorizeQueryOptions
  ): Promise<VectorizeMatches>;
  /**
   * Insert a list of vectors into the index dataset. If a provided id exists, an error will be thrown.
   * @param vectors List of vectors that will be inserted.
   * @returns A promise that resolves with a unique identifier of a mutation containing the insert changeset.
   */
  public insert(vectors: VectorizeVector[]): Promise<VectorizeAsyncMutation>;
  /**
   * Upsert a list of vectors into the index dataset. If a provided id exists, it will be replaced with the new values.
   * @param vectors List of vectors that will be upserted.
   * @returns A promise that resolves with a unique identifier of a mutation containing the upsert changeset.
   */
  public upsert(vectors: VectorizeVector[]): Promise<VectorizeAsyncMutation>;
  /**
   * Delete a list of vectors with a matching id.
   * @param ids List of vector ids that should be deleted.
   * @returns A promise that resolves with a unique identifier of a mutation containing the delete changeset.
   */
  public deleteByIds(ids: string[]): Promise<VectorizeAsyncMutation>;
  /**
   * Get a list of vectors with a matching id.
   * @param ids List of vector ids that should be returned.
   * @returns A promise that resolves with the raw unscored vectors matching the id set.
   */
  public getByIds(ids: string[]): Promise<VectorizeVector[]>;
}
