# Milestone 3: Chunked Objects

**Goal:** Support objects larger than memory. Streaming reads and writes across multiple fixed-size chunks, with per-chunk and full-object checksums.

## Deliverables

- [ ] PUT streams request body into fixed-size chunks (default 8 MiB)
- [ ] Small objects still produce exactly one chunk (no behaviour change for M2 users)
- [ ] `ObjectManifest` records all chunks in ordinal order
- [ ] SHA-256 computed per chunk (inline during streaming)
- [ ] SHA-256 computed for full object (accumulated across all chunks, inline)
- [ ] GET streams all chunks in manifest order without loading full object into memory
- [ ] `Content-Length` correct on GET response (from manifest `total_size_bytes`)
- [ ] `GET /objects/{bucket}?prefix=&limit=&start_after=` → paginated JSON list
- [ ] List returns: `key`, `size_bytes`, `etag`, `content_type`, `version_id`, `created_at`
- [ ] `start_after` cursor maps to a redb range scan (exclusive lower bound on key)

## Chunk Layout

```
chunks/{first_2_hex_of_uuid}/{next_2_hex_of_uuid}/{chunk_uuid}.blob
```

Each chunk is at most `chunk_size_bytes` (from config). Last chunk may be smaller.

## Write Protocol (multi-chunk)

1. Validate bucket exists. Generate `upload_id`, `version_id`.
2. Open a `Sha256` hasher for the full object.
3. For each chunk:
   a. Open staging file `staging/uploads/{upload_id}/part-{N:06}.tmp`.
   b. Stream up to `chunk_size_bytes` from the request body into the file.
   c. Feed those bytes through a per-chunk `Sha256` and the full-object hasher.
   d. Fsync staging file.
   e. Rename to final chunk path. Fsync parent dir.
   f. Build `ChunkDescriptor` (ordinal, offset, size, checksum, node_id, path).
4. Finalize full-object checksum.
5. Build `ObjectManifest` with ordered `Vec<ChunkDescriptor>`.
6. Commit `ObjectVersion` + `ObjectManifest` atomically in redb.
7. Remove staging directory.

## Read Protocol (multi-chunk)

1. `get_latest_object_version` → `get_manifest`.
2. Open each chunk file in `manifest.chunks` order.
3. Stream bytes from each file sequentially into the HTTP response body.
4. Do not buffer the full object in memory.

## List Objects

```
GET /objects/{bucket}?prefix=foo/&limit=100&start_after=foo/bar.jpg
```

- Filter: keys with given prefix (empty prefix = all keys).
- Cursor: keys that sort strictly after `start_after`.
- When response length < limit → listing is complete.
- Default + max limit: 1000.

## Learning Focus

- Bounded streaming loops (read N bytes at a time from axum body)
- Backpressure: do not pull faster than you can write to disk
- Accumulating a hasher across chunk boundaries
- Ordered manifest construction
- Streaming HTTP responses from multiple file sources
- redb range scan for cursor pagination

## Done When

```bash
# Large object (>8 MiB)
dd if=/dev/urandom of=big.bin bs=1M count=32
curl -X PUT --data-binary @big.bin localhost:8080/objects/photos/big.bin
curl localhost:8080/objects/photos/big.bin -o big-copy.bin
diff big.bin big-copy.bin   # identical

# List
curl "localhost:8080/objects/photos?limit=10"
curl "localhost:8080/objects/photos?prefix=big&limit=10"
```
