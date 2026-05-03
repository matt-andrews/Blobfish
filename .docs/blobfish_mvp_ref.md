# Blobfish MVP — LLM Reference

Rust learning project. Single-node object store with distributed-shaped internals. Not S3-compatible.

---

## Stack

| Concern | Crate |
|---|---|
| HTTP | axum, tower-http |
| Async runtime | tokio |
| Metadata | redb (sync — wrap all calls in `spawn_blocking`) |
| Checksums | sha2 (SHA-256) |
| IDs | uuid v4 |
| Serialization | serde, toml |
| Logging | tracing, tracing-subscriber |
| Errors | thiserror |

---

## Non-Goals (MVP)

Full S3 compat, erasure coding, Raft/consensus, cross-node replication, auth beyond dev token, lifecycle rules, object locking, compression/encryption at rest, web UI.

---

## Core Types

```rust
struct ClusterId(Uuid);
struct NodeId(Uuid);

struct NodeDescriptor {
    node_id: NodeId,
    address: String,
    storage_root: PathBuf,
    capacity_bytes: Option<u64>,
    status: NodeStatus,
}

struct Bucket {
    name: String,
    created_at: DateTime<Utc>,
    versioning: VersioningMode,
}

struct ObjectVersion {
    bucket: String,
    key: String,
    version_id: Uuid,
    size_bytes: u64,
    content_type: Option<String>,
    etag: String,           // SHA-256 hex of full object content, RFC 7232 double-quoted
    checksum_sha256: String,
    manifest_id: Uuid,
    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

struct ObjectManifest {
    manifest_id: Uuid,
    version_id: Uuid,
    chunks: Vec<ChunkDescriptor>,
    total_size_bytes: u64,
    checksum_sha256: String,
}

struct ChunkDescriptor {
    chunk_id: Uuid,
    ordinal: u32,
    offset: u64,
    size_bytes: u64,
    checksum_sha256: String,
    node_id: NodeId,
    local_path: PathBuf,
}
```

---

## Validation Rules

**Bucket name:** 3–63 chars. Lowercase ASCII letters, digits, hyphens only. No leading/trailing hyphens. No consecutive hyphens.

**Object key:** 1–1024 bytes (UTF-8). No null bytes. No leading/trailing whitespace. Slashes allowed but carry no storage-engine meaning.

---

## HTTP API

```
PUT    /buckets/{bucket}                   201 created / 200 exists / 400 invalid name
GET    /buckets                            200 JSON array
DELETE /buckets/{bucket}                   204 deleted / 409 not empty / 404 not found

PUT    /objects/{bucket}/{*key}            201 + version headers
GET    /objects/{bucket}/{*key}            200 streaming body
HEAD   /objects/{bucket}/{*key}            200 headers only
DELETE /objects/{bucket}/{*key}            204 (soft delete — marks version deleted)
GET    /objects/{bucket}?prefix=&limit=&start_after=   200 JSON array

GET    /healthz                            200 (process alive)
GET    /readyz                             200 (DB open, storage root writable, node descriptor exists)
GET    /debug/node                         200 JSON node descriptor + cluster ID + capacity
GET    /debug/objects/{bucket}/{*key}      200 JSON full manifest + chunk list for latest version
```

**PUT /objects response headers:**
```
ETag: "sha256hexoffullobject"
X-Blobfish-Version-Id: {uuid}
X-Blobfish-Checksum-Sha256: {hex}
Content-Length: {n}
Content-Type: {mime}
```

**Request headers (PUT):**
- `Content-Type` — optional
- `X-Blobfish-Checksum-Sha256` — optional client-provided validation

**List pagination:** `start_after` is a cursor key (exclusive lower bound). Response shorter than `limit` means listing is complete. Default + max limit: 1000.

---

## Error Response

All errors:
```json
{ "error": "BucketNotFound", "message": "Bucket 'photos' does not exist" }
```

| Code | HTTP |
|---|---|
| `InvalidBucketName`, `InvalidKey`, `InvalidInput` | 400 |
| `BucketNotFound`, `ObjectNotFound` | 404 |
| `BucketNotEmpty` | 409 |
| `ChecksumMismatch` | 422 |
| `CorruptObject`, `Internal` | 500 |
| `StorageUnavailable`, `MetadataUnavailable` | 503 |

Never include internal paths, stack traces, or chunk IDs in error messages.

---

## Storage Layout

```
/data/blobfish/
  node.json                         ← node descriptor, persisted on first boot
  meta/
    blobfish.db                     ← redb database
  chunks/
    {xx}/
      {yy}/
        {chunk_id}.blob             ← final chunk file
  staging/
    uploads/
      {upload_id}/
        part-000000.tmp
        part-000001.tmp
  trash/
  logs/
```

Chunk path: first 2 hex chars of UUID → subdir, next 2 → subdir, then `{chunk_uuid}.blob`.
No sidecar `.meta` files — all chunk metadata lives in redb only.

---

## MetadataStore Trait

```rust
#[async_trait]
trait MetadataStore {
    async fn create_bucket(&self, bucket: Bucket) -> Result<CreateBucketResult>;
    async fn get_bucket(&self, name: &str) -> Result<Option<Bucket>>;
    async fn delete_bucket(&self, name: &str) -> Result<DeleteBucketResult>;
    async fn list_buckets(&self) -> Result<Vec<Bucket>>;

    async fn put_object_version(&self, version: ObjectVersion, manifest: ObjectManifest) -> Result<()>;
    async fn get_latest_object_version(&self, bucket: &str, key: &str) -> Result<Option<ObjectVersion>>;
    async fn mark_object_deleted(&self, bucket: &str, key: &str) -> Result<()>;
    async fn get_manifest(&self, manifest_id: Uuid) -> Result<Option<ObjectManifest>>;
    async fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        start_after: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ObjectVersion>>;
}
```

`put_object_version` must write version + manifest in a single atomic redb transaction.

redb table layout:
```
clusters        key: cluster_id
nodes           key: node_id
buckets         key: bucket_name
object_versions key: (bucket, key, version_id)
manifests       key: manifest_id
uploads         key: upload_id
```

---

## PlacementEngine Trait

Exists as a stub from Milestone 0. M5 is when the node descriptor becomes real data.

```rust
trait PlacementEngine {
    fn place_object(&self, request: PlacementRequest) -> Result<PlacementPlan>;
}

struct PlacementRequest { bucket: String, key: String, size_hint: Option<u64>, desired_replication: u8 }
struct PlacementPlan    { chunk_size_bytes: u64, replicas: Vec<NodeId> }
```

MVP impl: `SingleNodePlacementEngine` always returns `[local_node_id]`.

---

## Write Path & Invariants

```
PUT /objects/…
  → validate bucket
  → create upload session
  → PlacementEngine::place_object
  → BlobStore::write_stream_as_chunks
  → build manifest
  → MetadataStore::put_object_version   ← visibility boundary
  → return version_id, etag, checksum
```

**Invariants:**
- Data written and fsynced before metadata commit.
- Never overwrite chunk files in place.
- Metadata commit is the sole visibility boundary — nothing is visible until it commits.
- Failed writes leave only staging files.
- SHA-256 accumulates in a hasher fed inline during streaming — never re-read after writing.
- After renaming chunk from staging to final path, fsync the parent directory.
- Staging directories cleaned on startup if older than `staging_ttl_seconds`.

Chunk size: 8 MiB default. Small objects = 1 chunk. Large objects stream into N chunks.

---

## Read Path & Invariants

```
GET /objects/…
  → MetadataStore::get_latest_object_version
  → MetadataStore::get_manifest
  → BlobStore::open_chunk_streams
  → stream bytes to client (do not buffer full object)
```

If metadata references a missing chunk: return 500 + log corruption event (repair path later).

---

## Background Jobs

**Startup recovery (once at boot):**
- Delete stale staging uploads.
- Verify storage root exists and is writable.
- Verify redb opens.
- Optionally scan manifests for missing chunks.

**Scrubber (periodic):**
- Sample manifests, verify chunk files exist.
- Optionally recompute checksum.
- Emit structured log events.

**GC (post-MVP):** find unreferenced chunks, move to trash, delete after grace period.

---

## Crate Layout

```
blobfish/
  Cargo.toml (workspace)
  crates/
    blobfish-server/   binary entrypoint: CLI, startup, shutdown
    blobfish-api/      HTTP only: routes, extractors, response types
    blobfish-core/     business logic: ObjectService, placement, models, errors
    blobfish-meta/     MetadataStore trait + redb impl
    blobfish-store/    BlobStore trait + local impl, checksum streaming
    blobfish-node/     config, NodeDescriptor, startup recovery
    blobfish-client/   node-to-node HTTP client (placeholder until M7)
```

---

## Config (dev.toml)

```toml
[node]
cluster_id   = "dev-cluster"
node_id      = "auto"          # generate UUID on first boot, persist to node.json
bind_addr    = "0.0.0.0:8080"
storage_root = "/data/blobfish"

[storage]
chunk_size_bytes    = 8388608  # 8 MiB
staging_ttl_seconds = 86400

[metadata]
engine = "redb"
path   = "/data/blobfish/meta/blobfish.db"

[placement]
replication_factor = 1

[observability]
log_level = "debug"
```

---

## Dockerfile (multi-stage)

```dockerfile
FROM rust:1 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -m blobfish
USER blobfish
COPY --from=builder /app/target/release/blobfish /usr/local/bin/blobfish
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/healthz || exit 1
CMD ["blobfish", "serve", "--config", "/etc/blobfish/config.toml"]
```

All core crates are pure Rust; glibc from bookworm-slim is sufficient. `ca-certificates` needed for outbound HTTPS (M7+).

---

## Milestones

| # | Name | Deliverables |
|---|---|---|
| 0 | Skeleton | workspace, `blobfish serve`, config, `/healthz`, Dockerfile, Compose, tracing |
| 1 | Buckets | create/list/delete bucket, redb persistence, validation |
| 2 | Single-Chunk Objects | PUT/GET/HEAD/DELETE, SHA-256, atomic write protocol |
| 3 | Chunked Objects | fixed-size chunking, manifests, ordered streaming, per-chunk + object checksums |
| 4 | Recovery & Scrubbing | staging cleanup, manifest scan, periodic scrubber, `/debug/objects` |
| 5 | Distributed Shape | real NodeDescriptor, ClusterId, PlacementEngine with stored plans, `/debug/node` |
| 6 | Multi-Process Compose | 3-node compose, per-node volumes, peer reporting, no cross-node writes |
| 7 | Replicated Writes | replication factor, node-to-node chunk PUT, read fallback to replica |
| 8 | Repair | scrubber detects under-replication, repair worker, replication health endpoint |
| 9 | Multipart Upload | initiate/upload-part/list/complete/abort |
| 10 | Erasure Coding | Reed-Solomon prototype behind feature flag, encode/reconstruct |

**MVP cut line: M0–M4.** First slice (fastest demo): `PUT /buckets`, `PUT /objects`, `GET /objects`, `GET /healthz`.

---

## Observability Log Fields

`request_id`, `bucket`, `key`, `version_id`, `upload_id`, `node_id`, `chunk_id`, `bytes_written`, `duration_ms`

---

## Key Decisions / Invariants Summary

- Every PUT creates an immutable `ObjectVersion` even without external versioning.
- Every version points to a `Manifest`; every manifest points to ordered `ChunkDescriptors`.
- `ChunkDescriptor.node_id` is stored on every chunk from day one (enables future replication).
- Metadata commit = visibility boundary. Never the other way around.
- SHA-256 flows inline through the streaming write loop — one pass only.
- ETag = `"sha256hex"` (content hash, RFC 7232 double-quoted). Never a version UUID.
- redb is synchronous — all calls need `tokio::task::spawn_blocking`.
- `PlacementEngine` and `MetadataStore` are traits from day one; single-node impls are just the simplest case.
- DELETE is a soft delete (mark `deleted_at`). Physical GC is post-MVP.
- `node_id = "auto"` generates a UUID on first boot and persists it to `node.json`.
