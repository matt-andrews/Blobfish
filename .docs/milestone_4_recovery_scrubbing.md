# Milestone 4: Recovery and Scrubbing

**Goal:** Detect and surface broken state. The system should never silently serve corrupt data, and it should self-describe what it knows about its storage health.

## Deliverables

- [ ] Startup recovery job (runs once at boot, blocks `readyz` until complete)
- [ ] Periodic scrub job (background tokio task, configurable interval)
- [ ] `GET /debug/objects/{bucket}/{*key}` → full manifest + chunk list JSON
- [ ] `GET /debug/node` → node descriptor, cluster ID, storage root, capacity info
- [ ] Structured log events emitted for every recovery/scrub finding
- [ ] Startup logs clearly distinguish recovery phase from normal operation

## Startup Recovery (blocks readyz)

Order of operations:
1. Verify `storage_root` directory exists and is writable.
2. Open redb database (`metadata.path`). Fail hard if unable.
3. Load or create `node.json` (generate UUID if `node_id = "auto"`, persist).
4. Delete staging upload directories older than `staging_ttl_seconds`.
5. Optional: scan all committed manifests; for each chunk, verify the file exists on disk. Log any missing chunks as `WARN` events. Do not fail startup — just surface the finding.
6. Mark node `readyz`.

## Scrubber (background task)

- Runs on a configurable interval (e.g., every 5 minutes; exact interval TBD in config).
- Samples a batch of manifests from redb (random or sequential scan).
- For each manifest:
  - Verify each referenced chunk file exists.
  - Optionally recompute SHA-256 and compare against stored checksum.
  - Emit a structured log event per finding: `chunk_ok`, `chunk_missing`, `chunk_corrupt`.
- Scrubber should be cancellation-safe (tokio `CancellationToken` or select on shutdown signal).

## Debug Endpoints

**`GET /debug/node`** — JSON:
```json
{
  "node_id": "...",
  "cluster_id": "...",
  "storage_root": "/data/blobfish",
  "capacity_bytes": null,
  "status": "Ready"
}
```

**`GET /debug/objects/{bucket}/{*key}`** — JSON:
```json
{
  "version": { ...ObjectVersion fields... },
  "manifest": {
    "manifest_id": "...",
    "total_size_bytes": 12345,
    "checksum_sha256": "...",
    "chunks": [
      { "ordinal": 0, "chunk_id": "...", "size_bytes": 8388608, "checksum_sha256": "...", "node_id": "...", "local_path": "..." }
    ]
  }
}
```

Returns 404 if object not found, 500 if manifest missing for a committed version.

## Structured Log Fields

Every scrub/recovery event should include:
`node_id`, `chunk_id`, `manifest_id`, `version_id`, `bucket`, `key`, `event` (e.g., `chunk_missing`), `path`

## Learning Focus

- tokio background tasks (`tokio::spawn`)
- Graceful shutdown with `CancellationToken` or `broadcast` channel
- `readyz` gating on async startup work
- Structured tracing spans and events
- Operational thinking: what does a broken node look like?

## Done When

```bash
# Force a broken manifest: delete a chunk file manually
docker exec -it blobfish rm /data/blobfish/chunks/ab/cd/somechunk.blob
docker compose restart   # startup recovery logs WARN for missing chunk

curl localhost:8080/readyz   # 200 (missing chunk doesn't block readyz)
curl localhost:8080/debug/objects/photos/cat.jpg   # shows manifest with chunk details
# Scrubber emits log events within its interval for the missing chunk
```
