# Milestone 2: Single-Chunk Objects

**Goal:** Store and retrieve small objects safely. Full atomic write protocol, checksums, and streaming responses — but no chunking yet (objects that exceed chunk size are still one blob).

## Deliverables

- [ ] `PUT /objects/{bucket}/{*key}` → 201 + version headers
- [ ] `GET /objects/{bucket}/{*key}` → 200 streaming body
- [ ] `HEAD /objects/{bucket}/{*key}` → 200 headers only
- [ ] `DELETE /objects/{bucket}/{*key}` → 204 (soft delete: sets `deleted_at`)
- [ ] Object key validation (1–1024 bytes UTF-8, no null bytes, no leading/trailing whitespace)
- [ ] SHA-256 computed inline during streaming write (one pass, no re-read)
- [ ] Atomic write: stream → staging file → fsync → rename to final path → fsync parent dir → commit metadata
- [ ] ETag = `"sha256hex"` (double-quoted per RFC 7232)
- [ ] `ObjectVersion` + `ObjectManifest` written atomically in single redb transaction
- [ ] GET returns 404 for deleted objects
- [ ] `Content-Type` header round-trips (stored and returned)
- [ ] Optional: `X-Blobfish-Checksum-Sha256` request header validated on PUT

## Response Headers (PUT / GET / HEAD)

```
ETag: "sha256hexoffullobject"
X-Blobfish-Version-Id: {uuid}
X-Blobfish-Checksum-Sha256: {hex}
Content-Length: {n}
Content-Type: {mime or application/octet-stream}
```

## Write Protocol (single chunk)

1. Validate bucket exists.
2. Generate `upload_id` and `version_id`.
3. Create staging file at `staging/uploads/{upload_id}/part-000000.tmp`.
4. Stream request body into file; feed bytes through `Sha256` hasher simultaneously.
5. Flush + fsync staging file.
6. Rename to `chunks/{xx}/{yy}/{chunk_id}.blob`.
7. Fsync parent directory.
8. Build `ChunkDescriptor` and `ObjectManifest`.
9. Commit `ObjectVersion` + `ObjectManifest` in single redb transaction.
10. Delete staging directory.

## Invariants

- Never commit metadata before chunks are fsynced.
- Staging files left by a crash are invisible — cleaned up at startup.
- SHA-256 is computed during streaming, not after.
- ETag is always the content hash, never a UUID.

## MetadataStore Methods Used

```rust
get_bucket(name)
put_object_version(version, manifest)   // atomic: version + manifest in one txn
get_latest_object_version(bucket, key)
get_manifest(manifest_id)
mark_object_deleted(bucket, key)
```

## Learning Focus

- Axum body streaming (`BodyStream` / `axum::body::Body`)
- `tokio::fs` async file I/O
- `sha2::Sha256` incremental hashing
- `std::fs::rename` + fsync parent directory
- Atomic metadata commit pattern
- HTTP streaming response with `StreamBody`

## Done When

```bash
curl -X PUT  localhost:8080/buckets/photos
curl -X PUT  --data-binary @cat.jpg -H "Content-Type: image/jpeg" \
             localhost:8080/objects/photos/cat.jpg
curl         localhost:8080/objects/photos/cat.jpg -o out.jpg
diff cat.jpg out.jpg                          # identical
curl -I      localhost:8080/objects/photos/cat.jpg   # ETag, X-Blobfish-* headers present
curl -X DELETE localhost:8080/objects/photos/cat.jpg # 204
curl         localhost:8080/objects/photos/cat.jpg   # 404
docker compose restart
curl         localhost:8080/objects/photos/cat.jpg   # still 404 (delete persisted)
```
