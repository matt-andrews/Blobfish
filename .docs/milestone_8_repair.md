# Milestone 8: Repair

**Goal:** Automatically heal under-replicated chunks. When the scrubber detects that a chunk has fewer live replicas than `replication_factor`, a repair worker should copy the chunk from a healthy replica to a new node.

## Deliverables

- [ ] Scrubber extended to detect under-replicated chunks (not just missing local files)
- [ ] Scrubber emits `chunk_under_replicated` structured log event with `chunk_id`, `healthy_replicas`, `required_replicas`
- [ ] Repair worker: background task that consumes under-replicated chunk events
- [ ] Repair fetches chunk bytes from a healthy replica node via internal API
- [ ] Repair writes chunk to a new target node (selected by `PlacementEngine`)
- [ ] Repair updates `ChunkDescriptor` in metadata to reflect the new replica node
- [ ] Repair is idempotent: re-running on an already-healthy chunk is a no-op
- [ ] `GET /debug/objects/{bucket}/{*key}` shows replica health per chunk (which nodes are alive vs. missing)
- [ ] Repair operations logged with `chunk_id`, `source_node`, `target_node`, `outcome`

## Replication Health Check (Scrubber Extension)

For each `ChunkDescriptor` in a manifest:
1. For each `node_id` in the replica list, check if the chunk file exists on that node.
   - Local node: check filesystem directly.
   - Remote node: `HEAD /internal/chunks/{chunk_id}` (new internal endpoint).
2. Count healthy replicas.
3. If `healthy < replication_factor`: emit `chunk_under_replicated` event.

## Internal API Addition

```
HEAD /internal/chunks/{chunk_id}
  Response: 200 exists | 404 not found
```

## Repair Worker

- Runs as a background tokio task.
- Receives under-replicated chunk events from scrubber (via channel or periodic scan).
- For each under-replicated chunk:
  1. Pick a healthy source replica.
  2. `GET /internal/chunks/{chunk_id}` from source (streaming).
  3. Write to target node via `PUT /internal/chunks/{chunk_id}`.
  4. Verify checksum on receipt.
  5. Update `ChunkDescriptor` metadata to add the new replica node.
- Cancellation-safe.

## Internal API Addition

```
GET /internal/chunks/{chunk_id}
  Response: 200 raw bytes | 404 not found
  Headers: X-Blobfish-Chunk-Checksum: {sha256hex}
```

## Idempotency

- Before writing to target node, check if the chunk already exists there (`HEAD`).
- Before updating metadata, verify the target node is not already in the replica list.

## Learning Focus

- Anti-entropy repair as an idempotent background process
- Coordinating reads and writes across nodes without a lock
- Channel-based work queues in tokio
- Metadata mutation for repair (updating replica lists)

## Done When

```bash
# Start with replication_factor = 2
docker compose -f docker-compose.multi.yml up --build
curl -X PUT --data-binary @cat.jpg localhost:8081/objects/photos/cat.jpg

# Manually delete chunk from one replica
docker exec blobfish-b rm /data/blobfish/chunks/.../chunk.blob

# Wait for scrubber interval
# Logs: "chunk_under_replicated" then "repair_started" then "repair_complete"

curl localhost:8081/debug/objects/photos/cat.jpg
# → chunk now shows 2 healthy replicas again (one may be on node-c now)
```
