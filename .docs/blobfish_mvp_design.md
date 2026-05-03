# Blobfish MVP Design

## 1. Project Intent

Blobfish is an open source distributed object storage system written in Rust. The purpose is learning-first: build something real enough to teach Rust, async I/O, persistence, checksums, recovery, Dockerized operations, and eventually distributed systems mechanics.

The project should begin as a single-node object store, but its internal model should avoid single-node assumptions. The MVP should feel like a small object storage service, not a file server.

## 2. Guiding Principles

1. **Single-node first, distributed-shaped from day one**
   - One process, one local disk path, one metadata store.
   - Internally model nodes, placements, object versions, chunks, manifests, and health checks.
   - The single node is simply a cluster of one.

2. **Object storage, not filesystem semantics**
   - Objects are addressed by bucket and key.
   - Writes replace whole objects atomically.
   - Reads return immutable object versions.
   - No POSIX directory semantics in the core model.

3. **Safety before performance**
   - Checksums everywhere.
   - Write temp files, fsync where appropriate, then atomic rename.
   - Metadata commits should happen after data durability.
   - Startup recovery should detect incomplete writes.

4. **Docker as the default runtime**
   - Every milestone should run locally through Docker Compose.
   - The developer should be able to delete volumes, recreate nodes, and observe recovery behavior.

5. **Learn Rust through hard but bounded problems**
   - Async HTTP streaming.
   - Error handling.
   - Durable local storage.
   - Trait-based abstraction.
   - Background tasks.
   - Concurrency control.
   - Future distributed protocols.

## 3. Non-Goals for the MVP

The MVP should not attempt these yet:

- Full S3 compatibility.
- Erasure coding.
- Raft or distributed consensus.
- Cross-node replication.
- Authentication beyond a simple development token.
- Multi-tenant policy management.
- Lifecycle rules.
- Object locking / legal hold.
- Compression or encryption at rest.
- Web UI.

These can be added later once the storage engine and metadata model are reliable.

## 4. MVP User Experience

A developer should be able to run:

```bash
docker compose up --build
```

Then interact with Blobfish through HTTP:

```bash
curl -X PUT localhost:8080/buckets/photos
curl -X PUT --data-binary @cat.jpg localhost:8080/objects/photos/cat.jpg
curl localhost:8080/objects/photos/cat.jpg -o cat-copy.jpg
curl -I localhost:8080/objects/photos/cat.jpg
curl -X DELETE localhost:8080/objects/photos/cat.jpg
```

The MVP should expose health and debug endpoints:

```bash
curl localhost:8080/healthz
curl localhost:8080/readyz
curl localhost:8080/debug/node
curl localhost:8080/debug/objects/photos/cat.jpg
```

## 5. High-Level Architecture

```text
+------------------+
| HTTP API          |
| axum/tower        |
+---------+--------+
          |
          v
+------------------+
| Object Service    |
| validation, auth, |
| versioning rules  |
+---------+--------+
          |
          v
+------------------+        +------------------+
| Metadata Store    | <----> | Placement Engine |
| buckets, objects, |        | cluster-of-one   |
| versions, chunks  |        | today, N nodes    |
+---------+--------+        | later            |
          |                 +------------------+
          v
+------------------+
| Blob Store        |
| chunk files on    |
| local disk        |
+---------+--------+
          |
          v
+------------------+
| Background Jobs   |
| recovery, scrub,  |
| compaction later  |
+------------------+
```

## 6. Core Concepts

### Cluster

Even in the MVP, Blobfish should have a cluster model.

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
```

For MVP:

- The cluster contains one node.
- The node descriptor is loaded from config or created on first boot.
- The placement engine always returns the local node.

Later:

- Add node gossip or static peer config.
- Add replication factor.
- Add health-aware placement.
- Add rebalance jobs.

### Bucket

A bucket is a namespace for object keys.

```rust
struct Bucket {
    name: String,
    created_at: DateTime<Utc>,
    versioning: VersioningMode,
}
```

MVP bucket rules:

- Bucket names are globally unique inside the cluster.
- Bucket creation is idempotent.
- Bucket deletion only succeeds when empty.
- Valid names: 3–63 characters, lowercase ASCII letters, digits, and hyphens only. No leading/trailing hyphens, no consecutive hyphens.

### Object Key

An object key is an opaque UTF-8 string. It may contain slashes, but slashes have no storage-engine meaning.

Constraints: 1–1024 bytes (UTF-8 encoded). No null bytes. Leading/trailing whitespace is an error. Everything else is allowed including unicode.

Examples:

- `cat.jpg`
- `users/123/avatar.png`
- `backups/2026/05/03/db.dump`

### Object Version

Every successful PUT creates an immutable object version, even if external versioning is not exposed yet.

```rust
struct ObjectVersion {
    bucket: String,
    key: String,
    version_id: Uuid,
    size_bytes: u64,
    content_type: Option<String>,
    etag: String,
    checksum_sha256: String,
    manifest_id: Uuid,
    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}
```

For MVP:

- Only the latest non-deleted version is returned by GET/HEAD.
- DELETE creates a delete marker or marks the latest version deleted.
- Old versions may remain physically present until garbage collection exists.

This teaches safe immutable writes and prepares the system for replication.

### Chunk

Objects are stored as one or more chunks.

```rust
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

For MVP:

- Use fixed-size chunks, for example 8 MiB.
- Small objects have one chunk.
- Large objects stream into multiple chunks.
- The manifest records ordered chunks.

Chunking early is important because it makes later multipart upload, replication, healing, and erasure coding easier.

### Manifest

The manifest is the durable description of an object version.

```rust
struct ObjectManifest {
    manifest_id: Uuid,
    version_id: Uuid,
    chunks: Vec<ChunkDescriptor>,
    total_size_bytes: u64,
    checksum_sha256: String,
}
```

The read path should trust metadata only after verifying local chunk existence. Optional checksum verification can happen inline for MVP or through background scrubbing.

## 7. HTTP API MVP

The API should be intentionally small and S3-shaped, but not fully S3-compatible.

### Create Bucket

```http
PUT /buckets/{bucket}
```

Responses:

- `201 Created` when created.
- `200 OK` when already exists.
- `400 Bad Request` for invalid bucket name.

### List Buckets

```http
GET /buckets
```

Returns JSON.

### Delete Bucket

```http
DELETE /buckets/{bucket}
```

Responses:

- `204 No Content` when deleted.
- `409 Conflict` when not empty.

### Put Object

```http
PUT /objects/{bucket}/{*key}
```

Headers:

- `Content-Type`, optional.
- `X-Blobfish-Checksum-Sha256`, optional for client-provided validation.

Behavior:

1. Validate bucket exists.
2. Create upload ID.
3. Stream body into temporary chunks.
4. Compute checksum while streaming.
5. Persist chunks.
6. Create manifest.
7. Commit metadata transaction.
8. Return object version info.

### Get Object

```http
GET /objects/{bucket}/{*key}
```

Behavior:

1. Find latest visible version.
2. Load manifest.
3. Stream chunks in order.
4. Support `Range` later, but not required in the first slice.

### Head Object

```http
HEAD /objects/{bucket}/{*key}
```

Returns metadata headers without body.

### Delete Object

```http
DELETE /objects/{bucket}/{*key}
```

For MVP, mark latest version deleted. Physical chunk deletion can wait for garbage collection.

### List Objects

```http
GET /objects/{bucket}?prefix=foo/&limit=100&start_after=foo/bar.jpg
```

Returns JSON array of object metadata (key, size, etag, content_type, version_id, created_at).

Pagination: use `start_after` as a cursor — return only objects whose key sorts strictly after `start_after`. This maps naturally to redb range scans. When the response contains fewer items than `limit`, the listing is complete. Default limit 1000, max 1000.

### Health

```http
GET /healthz
GET /readyz
```

- `healthz`: process is alive.
- `readyz`: metadata store can be opened, storage root is writable, node descriptor exists.

### Debug

```http
GET /debug/node
GET /debug/objects/{bucket}/{*key}
```

- `/debug/node`: returns node descriptor, cluster ID, storage root, and capacity info as JSON.
- `/debug/objects/{bucket}/{key}`: returns the full manifest and chunk list for the latest version — useful for inspecting storage internals and verifying recovery.

## 8. Storage Layout on Disk

Use a layout that avoids massive single directories.

```text
/data/blobfish/
  node.json
  meta/
    blobfish.db
  chunks/
    ab/
      cd/
        abcd...{chunk_id}.blob
  staging/
    uploads/
      {upload_id}/
        part-000000.tmp
        part-000001.tmp
  trash/
  logs/
```

All chunk metadata (checksum, size, ordinal, node_id) lives in the redb metadata store. There are no sidecar `.meta` files — if the metadata store is lost, chunks become orphaned blobs. This is acceptable for MVP; a future scrub/repair job could reconstruct metadata from chunk content if needed.

### Chunk File Naming

Chunk file path can be based on the chunk content hash or generated UUID.

Recommended MVP:

- Use UUID for chunk identity.
- Store checksum in metadata.
- Later experiment with content-addressed chunks.

Example:

```text
chunks/{first_two_hex}/{next_two_hex}/{chunk_id}.blob
```

### Temporary Write Protocol

For each chunk:

1. Create file in staging directory.
2. Stream request body into file.
3. Compute checksum as bytes arrive.
4. Flush file.
5. Rename into final chunk path.
6. Record chunk descriptor in memory.

After all chunks are finalized:

1. Create manifest.
2. Commit metadata transaction.
3. Remove staging directory.

On restart:

- Delete stale staging directories older than a configured threshold.
- Verify final chunks referenced by committed manifests exist.

## 9. Metadata Store

The metadata store should be embedded for MVP. Good candidates:

- `redb`: pure Rust embedded key-value database.
- `sled`: pure Rust embedded database, but project status should be checked before depending on it heavily.
- SQLite through `sqlx`: familiar relational model, very robust, not pure Rust.

Recommended MVP choice: **redb** for learning Rust and keeping the stack Rust-native.

> **Important async note:** redb is a synchronous library. Every redb call inside an `async fn` must be wrapped in `tokio::task::spawn_blocking` to avoid blocking the async executor. This is a good early lesson in bridging sync I/O into async Rust. The `#[async_trait]` facade below hides this detail from callers.

Suggested logical tables (as key-value namespaces in redb):

```text
clusters        key: cluster_id
nodes           key: node_id
buckets         key: bucket_name
object_versions key: (bucket, key, version_id)
manifests       key: manifest_id
uploads         key: upload_id
```

Even if the physical store is key-value, wrap it in a repository trait:

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

`put_object_version` must write both the version record and its manifest atomically in a single redb transaction. This is the metadata visibility boundary.

Important: keep storage-engine details behind traits so you can later swap in SQLite, Postgres, or a replicated metadata service.

## 10. Write Path

```text
Client
  |
  v
PUT /objects/photos/cat.jpg
  |
  v
ObjectService::put_object
  |
  +--> validate bucket
  +--> create upload session
  +--> PlacementEngine::place_chunks(size_hint?)
  +--> BlobStore::write_stream_as_chunks
  +--> build manifest
  +--> MetadataStore::commit_object_version
  +--> return version_id, etag, checksum
```

Important write invariants:

- Never expose an object version until its chunks are durable.
- Never overwrite chunk files in place.
- Metadata commit is the visibility boundary.
- Failed writes leave only staging files, never visible object versions.
- Retrying the same PUT is allowed but creates a new version unless idempotency is added later.
- SHA-256 must accumulate as bytes stream through — do not re-read chunks after writing. Use a `Sha256` hasher in the streaming loop, feed bytes into it as they arrive, finalize after the last byte.
- After renaming a chunk from staging to its final path, fsync the parent directory to guarantee the directory entry is durable. Without this, a crash before the next sync can lose the rename.

## 11. Read Path

```text
Client
  |
  v
GET /objects/photos/cat.jpg
  |
  v
ObjectService::get_object
  |
  +--> MetadataStore::get_latest_object_version
  +--> MetadataStore::get_manifest
  +--> BlobStore::open_chunk_streams
  +--> stream bytes to client
```

Important read invariants:

- If metadata points to missing chunks, return `500` for MVP and record a corruption event.
- Later this becomes a repair path.
- GET should stream rather than loading the full object into memory.

## 12. Placement Engine

The placement engine trait should exist from Milestone 0 as a stub — even if it only returns `[local_node_id]`. Milestone 5 is when the node descriptor and cluster model become real data rather than hardcoded values, not when the trait is introduced.

```rust
trait PlacementEngine {
    fn place_object(&self, request: PlacementRequest) -> Result<PlacementPlan>;
}

struct PlacementRequest {
    bucket: String,
    key: String,
    size_hint: Option<u64>,
    desired_replication: u8,
}

struct PlacementPlan {
    chunk_size_bytes: u64,
    replicas: Vec<NodeId>,
}
```

MVP implementation:

```text
SingleNodePlacementEngine -> always returns [local_node_id]
```

Future implementations:

- Rendezvous hashing.
- Replication-factor-aware placement.
- Rack-aware placement.
- Disk-pressure-aware placement.
- Erasure-set placement.

This is one of the most important seams in the project.

## 13. Checksums and ETags

Use SHA-256 internally for data integrity.

For MVP:

- Compute SHA-256 per chunk.
- Compute SHA-256 for the full object.
- Store both.
- Return an `ETag` header, but do not promise S3-compatible ETag semantics.

Recommended response headers:

```http
ETag: "{sha256_hex_of_full_object}"
X-Blobfish-Version-Id: {uuid}
X-Blobfish-Checksum-Sha256: {hex}
Content-Length: {size}
Content-Type: {content_type}
```

ETag must be the content hash, not the version UUID. ETags are semantically "does the content match" — using a random ID breaks conditional GETs and multipart ETag verification later. Wrap in double-quotes as required by RFC 7232.

Later:

- Add client-supplied checksum validation.
- Add background scrub jobs.
- Add repair from replicas.

## 14. Background Jobs

MVP background jobs should be simple and visible in logs.

### Startup Recovery

Runs once at boot:

- Remove stale staging uploads.
- Verify storage root exists.
- Verify metadata database opens.
- Optionally scan manifests and report missing chunks.

### Scrubber

Runs periodically:

- Sample object manifests.
- Verify chunk files exist.
- Optionally recompute checksum.
- Emit structured events.

### Garbage Collector

Can wait until after MVP, but design for it:

- Find chunks not referenced by live manifests.
- Move to trash first.
- Delete after grace period.

## 15. Docker Setup

### Dockerfile

Use a multi-stage build:

```dockerfile
FROM rust:1 AS builder
WORKDIR /app
# Cache dependency compilation separately from application code.
# Copy manifests first, build a dummy main, then copy src.
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -m blobfish
USER blobfish
WORKDIR /app
COPY --from=builder /app/target/release/blobfish /usr/local/bin/blobfish
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/healthz || exit 1
CMD ["blobfish", "serve", "--config", "/etc/blobfish/config.toml"]
```

Note: `ca-certificates` is needed if the binary ever makes outbound HTTPS calls (e.g., to remote peers). All core crates (`redb`, `axum`, `sha2`) are pure Rust with no native C dependencies, so glibc from `bookworm-slim` is sufficient.

### docker-compose.yml

```yaml
services:
  blobfish:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - blobfish-data:/data/blobfish
      - ./models/dev.toml:/etc/blobfish/models.toml:ro
    environment:
      RUST_LOG: blobfish=debug,tower_http=info

volumes:
  blobfish-data:
```

Later, add multi-node compose:

```yaml
services:
  blobfish-a:
  blobfish-b:
  blobfish-c:
```

Do this before implementing true replication so the operational shape is ready.

## 16. Suggested Rust Crate Layout

```text
blobfish/
  Cargo.toml
  crates/
    blobfish-server/      # binary entrypoint: CLI, startup, shutdown
      src/main.rs
    blobfish-api/         # HTTP layer only: routing, extractors, response types
      src/routes.rs
      src/extractors.rs
      src/responses.rs
    blobfish-core/        # business logic: object service, placement, models
      src/object_service.rs
      src/models.rs
      src/errors.rs
      src/placement.rs
    blobfish-meta/        # metadata persistence (redb impl)
      src/lib.rs
      src/redb_store.rs
    blobfish-store/       # blob I/O: chunk files, checksum streaming
      src/lib.rs
      src/local_blob_store.rs
      src/checksum.rs
    blobfish-node/        # node identity, config loading, startup recovery
      src/config.rs
      src/node.rs
      src/recovery.rs
    blobfish-client/      # HTTP client for internal node-to-node calls (Milestone 7+)
      src/lib.rs
```

`blobfish-client` is a placeholder until Milestone 7 (replicated writes). Start it as an empty crate to hold the workspace shape, and fill it in when inter-node communication is needed.

For a learning project, a workspace is worth it. It forces clean boundaries and teaches crate organization.

## 17. Error Model

Use one internal error type per crate, but convert to API errors at the edge.

Example API error categories:

- `InvalidBucketName`
- `BucketNotFound`
- `BucketNotEmpty`
- `ObjectNotFound`
- `ChecksumMismatch`
- `StorageUnavailable`
- `MetadataUnavailable`
- `CorruptObject`
- `Internal`

HTTP mapping:

```text
400 Invalid input
404 Missing bucket/object
409 Conflict
422 Checksum mismatch
500 Internal/corrupt state
503 Storage unavailable
```

All error responses use a consistent JSON body:

```json
{
  "error": "BucketNotFound",
  "message": "Bucket 'photos' does not exist"
}
```

The `error` field is a machine-readable code from the list above. The `message` field is human-readable. Do not embed internal paths, stack traces, or chunk IDs in production error messages.

## 18. Observability

Use structured logs from day one.

Recommended fields:

- `request_id`
- `bucket`
- `key`
- `version_id`
- `upload_id`
- `node_id`
- `chunk_id`
- `bytes_written`
- `duration_ms`

Expose metrics later:

- Total objects.
- Total bytes.
- PUT/GET latency.
- Bytes read/written.
- Failed writes.
- Missing chunks.
- Scrub failures.

Tracing is especially valuable because later distributed replication will need request correlation across nodes.

## 19. Testing Strategy

### Unit Tests

- Bucket name validation.
- Object key validation.
- Chunk path calculation.
- Manifest serialization.
- Placement engine behavior.
- Error mapping.

### Integration Tests

Use temporary directories and real metadata store:

- Create bucket.
- Put object.
- Get object.
- Head object.
- Delete object.
- List objects.
- Restart service and verify object still exists.
- Simulate failed upload and verify it is not visible.

### Docker Tests

- `docker compose up` starts service.
- Data survives container restart.
- Data disappears only when volume is deleted.

### Property-ish Tests

Later:

- Random object sizes.
- Random keys.
- Random interrupted writes.
- Verify read bytes equal written bytes.

## 20. Milestone Plan

Each milestone has its own file in this directory.

| # | File | Goal |
|---|---|---|
| 0 | [milestone_0_skeleton.md](milestone_0_skeleton.md) | Cargo workspace, healthz, Dockerfile, Compose |
| 1 | [milestone_1_buckets.md](milestone_1_buckets.md) | Persistent bucket metadata, redb, validation |
| 2 | [milestone_2_single_chunk_objects.md](milestone_2_single_chunk_objects.md) | PUT/GET/HEAD/DELETE, SHA-256, atomic write |
| 3 | [milestone_3_chunked_objects.md](milestone_3_chunked_objects.md) | Multi-chunk streaming, manifests, list objects |
| 4 | [milestone_4_recovery_scrubbing.md](milestone_4_recovery_scrubbing.md) | Startup recovery, scrubber, debug endpoints |
| 5 | [milestone_5_distributed_shape.md](milestone_5_distributed_shape.md) | Real NodeDescriptor, ClusterId, placement data |
| 6 | [milestone_6_multi_process_compose.md](milestone_6_multi_process_compose.md) | 3-node compose, peer config, no replication yet |
| 7 | [milestone_7_replicated_writes.md](milestone_7_replicated_writes.md) | Cross-node chunk writes, read fallback |
| 8 | [milestone_8_repair.md](milestone_8_repair.md) | Scrubber detects under-replication, repair worker |
| 9 | [milestone_9_multipart_upload.md](milestone_9_multipart_upload.md) | Initiate/part/complete/abort upload |
| 10 | [milestone_10_erasure_coding.md](milestone_10_erasure_coding.md) | Reed-Solomon experiment behind feature flag |

**MVP cut line: Milestones 0–4.** First implementation slice: `PUT /buckets`, `PUT /objects`, `GET /objects`, `GET /healthz`.

## 21. Recommended MVP Cut Line

The first “real MVP” should include milestones 0 through 4:

- Dockerized service.
- Persistent buckets.
- PUT/GET/HEAD/DELETE objects.
- Chunked object storage.
- Checksums.
- Startup recovery.
- Basic scrubber.

This is enough to feel real, demonstrate safe storage mechanics, and create a strong base for distributed work.

Do not wait for multi-node replication before calling it useful. The single-node version will already teach most of the Rust mechanics you need.

## 22. Concrete First Implementation Slice

Build this first:

```text
PUT /buckets/{bucket}
PUT /objects/{bucket}/{key}
GET /objects/{bucket}/{key}
GET /healthz
```

Use:

- Axum for HTTP.
- Tokio for async runtime.
- `tracing` for logs.
- `serde` and `toml` for config.
- `uuid` for IDs.
- `sha2` for checksums.
- `redb` for metadata.
- Local filesystem for chunks.

Defer:

- Listing.
- Delete.
- Multipart.
- Replication.
- Auth.
- Range reads.

This gives you the fastest path to a satisfying demo.

## 23. Example Configuration

```toml
[node]
cluster_id = "dev-cluster"
node_id = "auto"
bind_addr = "0.0.0.0:8080"
storage_root = "/data/blobfish"

[storage]
chunk_size_bytes = 8388608
staging_ttl_seconds = 86400

[metadata]
engine = "redb"
path = "/data/blobfish/meta/blobfish.db"

[placement]
replication_factor = 1

[observability]
log_level = "debug"
```

## 24. Design Risks

### Risk: Metadata and data get out of sync

Mitigation:

- Data first, metadata second.
- Startup scanner.
- Scrubber.
- Do not expose uncommitted uploads.

### Risk: Trying to clone S3 too early

Mitigation:

- Keep S3-shaped paths and headers.
- Avoid promising exact S3 behavior.
- Add compatibility slowly.

### Risk: Single-node shortcuts block distribution

Mitigation:

- Introduce node IDs and placement plans immediately.
- Store node ID on every chunk descriptor.
- Use traits for placement and blob storage.

### Risk: Rust complexity stalls progress

Mitigation:

- Avoid too many generic abstractions early.
- Prefer concrete implementations behind small traits.
- Keep the first demo tiny.

## 25. Why This Project Is a Strong Rust Learning Vehicle

Blobfish will force you to learn Rust in areas that matter:

- Ownership across async boundaries.
- Streaming request and response bodies.
- Error modeling with `thiserror` and HTTP mapping.
- Traits for storage and metadata abstractions.
- Serialization and durable data formats.
- Background workers and cancellation.
- Dockerized operational workflows.
- Eventually distributed failure handling.

It is also useful enough to stay motivating. Even the single-node version can store real files, survive restarts, and expose inspectable internals.

## 26. The North Star

Blobfish should evolve through these identities:

1. A safe local object store.
2. A chunked object store with manifests and checksums.
3. A cluster-shaped object store running as multiple Docker containers.
4. A replicated object store.
5. A self-healing object store.
6. An erasure-coded object store.
7. A partially S3-compatible educational object store.

The most important early decision is to make every object version immutable and every chunk addressable through a manifest. That one design choice keeps the MVP simple while making the distributed future plausible.

