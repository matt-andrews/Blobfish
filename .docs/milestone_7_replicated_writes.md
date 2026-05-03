# Milestone 7: Replicated Writes

**Goal:** Write object chunks to multiple nodes. One node acts as the coordinator for a PUT; it writes locally and forwards chunks to peer nodes. Reads can fall back to another replica if the primary chunk is unavailable.

## Deliverables

- [ ] `replication_factor` config drives how many replicas per chunk
- [ ] Internal node-to-node chunk PUT API (`PUT /internal/chunks/{chunk_id}`)
- [ ] Coordinator writes chunk to local disk and then forwards to `replication_factor - 1` peers
- [ ] `PlacementEngine` selects which nodes receive each chunk (round-robin or random from peer list)
- [ ] `ChunkDescriptor.node_id` correctly records which node holds each replica
- [ ] Write fails if fewer than `replication_factor` replicas acknowledge successfully
- [ ] Partial write cleanup: if a peer write fails after some chunks are forwarded, the coordinator should attempt to delete the partial forwarded chunks
- [ ] GET: if a chunk file is missing locally but another `node_id` is recorded in the manifest, fetch from that peer
- [ ] `blobfish-client` crate filled in (HTTP client for node-to-node calls)
- [ ] Timeouts on all inter-node calls (configurable)

## Internal API (not exposed on public port)

```
PUT /internal/chunks/{chunk_id}
  Body: raw chunk bytes
  Headers: X-Blobfish-Chunk-Checksum: {sha256hex}
  Response: 201 OK | 400 checksum mismatch | 500 storage error
```

This endpoint is on the same port but only called node-to-node. Authentication can be left as a future concern.

## PlacementEngine Update

`place_object` now returns multiple `NodeId`s when `replication_factor > 1`:

```rust
PlacementPlan {
    chunk_size_bytes: 8_388_608,
    replicas: vec![node_a_id, node_b_id],  // first = local coordinator
}
```

Selection strategy for MVP: pick the local node + random selection from configured peers until `replication_factor` nodes are chosen.

## Write Path Change

After writing each chunk locally:
1. For each additional replica node in the placement plan:
   a. POST the chunk bytes to `peer/internal/chunks/{chunk_id}`.
   b. Verify the peer returns 201.
2. If any peer rejects: attempt cleanup (DELETE on already-accepted peers), return 500.
3. Only build the manifest + commit metadata after all replicas have acknowledged.

## Read Path Change

On `BlobStore::open_chunk_streams`:
1. Attempt to open the local chunk file.
2. If file is missing but `chunk.node_id != local_node_id`, fetch from that peer.
3. Log the fallback as a `WARN` event (this indicates local data loss or a misplaced chunk).

## Learning Focus

- RPC design in Rust (reqwest or hyper client in `blobfish-client`)
- Handling partial success in distributed writes
- Timeout and retry behaviour
- Updating the placement engine from a stub to a real implementation
- Coordinated multi-node failure scenarios

## Done When

```bash
docker compose -f docker-compose.multi.yml up --build
# Set replication_factor = 2 in configs

curl -X PUT --data-binary @cat.jpg localhost:8081/objects/photos/cat.jpg
# → coordinator is node-a; chunks land on node-a and node-b

curl localhost:8081/debug/objects/photos/cat.jpg
# → manifest shows two node_ids per chunk

# Simulate node-a disk failure for that chunk
docker exec blobfish-a rm /data/blobfish/chunks/.../chunk.blob
curl localhost:8081/objects/photos/cat.jpg
# → 200, served from node-b replica (WARN log visible)
```
