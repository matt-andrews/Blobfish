# Milestone 1: Buckets

**Goal:** Persistent bucket metadata. First real use of redb; first durable state that survives a container restart.

## Deliverables

- [ ] `PUT /buckets/{bucket}` → 201 / 200 / 400
- [ ] `GET /buckets` → 200 JSON array
- [ ] `DELETE /buckets/{bucket}` → 204 / 409 / 404
- [ ] Bucket name validation (3–63 chars, lowercase alnum + hyphens, no leading/trailing/consecutive hyphens)
- [ ] redb metadata store wired into `blobfish-meta` behind `MetadataStore` trait
- [ ] All redb calls wrapped in `tokio::task::spawn_blocking`
- [ ] `GET /readyz` → checks DB opens, storage root is writable, node descriptor exists
- [ ] Bucket creation is idempotent
- [ ] Bucket deletion fails with 409 when not empty (always passes for now — no objects yet)
- [ ] Data survives `docker compose restart`

## Validation Rules

**Valid bucket name:** 3–63 chars. `[a-z0-9]` and `-` only. No leading `-`, no trailing `-`, no `--`.

## MetadataStore Methods Used

```rust
create_bucket(bucket: Bucket) -> Result<CreateBucketResult>
get_bucket(name: &str)        -> Result<Option<Bucket>>
delete_bucket(name: &str)     -> Result<DeleteBucketResult>
list_buckets()                -> Result<Vec<Bucket>>
```

## Error Responses

```json
{ "error": "InvalidBucketName", "message": "..." }
{ "error": "BucketNotFound",    "message": "..." }
{ "error": "BucketNotEmpty",    "message": "..." }
```

## Learning Focus

- `redb` table definition and read/write transactions
- `spawn_blocking` for sync I/O inside async handlers
- Trait-based repository pattern
- Serde serialization of domain types for storage
- Error conversion at the API boundary

## Done When

```bash
curl -X PUT  localhost:8080/buckets/photos        # 201
curl -X PUT  localhost:8080/buckets/photos        # 200 (idempotent)
curl         localhost:8080/buckets               # JSON list
curl -X DELETE localhost:8080/buckets/photos      # 204
docker compose restart
curl         localhost:8080/buckets               # still empty (delete persisted)
```
