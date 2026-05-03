# Milestone 5: Single-Node "Distributed Shape"

**Goal:** Replace hardcoded single-node assumptions with a real cluster model. The system still runs on one node, but every abstraction that will matter for replication is now real data, not hardcoded values.

## Deliverables

- [ ] `NodeDescriptor` persisted to `node.json` on first boot (UUID generated if `node_id = "auto"`)
- [ ] `ClusterId` loaded from config and stored in metadata
- [ ] `PlacementEngine` trait returns the real local `NodeId` (not a placeholder)
- [ ] Every `ChunkDescriptor` written with the correct `node_id` (from placement plan)
- [ ] `PlacementPlan` stored in the manifest (or derivable from chunk descriptors)
- [ ] `GET /debug/node` returns live data from `NodeDescriptor` (not static stubs)
- [ ] `GET /debug/objects` shows `node_id` on each chunk

## What Changes from Previous Milestones

Prior milestones could use `NodeId::nil()` or a hardcoded constant. After this milestone:
- `NodeId` comes from `node.json` (loaded at startup by `blobfish-node`)
- `PlacementEngine::place_object` receives a real `PlacementRequest` and returns a `PlacementPlan` referencing the persisted `NodeId`
- Any code that assumed a single node should now route through the placement engine

## Key Types (now real)

```rust
struct PlacementRequest {
    bucket: String,
    key: String,
    size_hint: Option<u64>,
    desired_replication: u8,   // always 1 for now
}

struct PlacementPlan {
    chunk_size_bytes: u64,
    replicas: Vec<NodeId>,     // always [local_node_id] for now
}
```

`SingleNodePlacementEngine` reads `local_node_id` from config/startup state rather than a hardcoded value.

## node.json Format

```json
{
  "node_id": "550e8400-e29b-41d4-a716-446655440000",
  "cluster_id": "dev-cluster",
  "address": "0.0.0.0:8080",
  "storage_root": "/data/blobfish",
  "created_at": "2026-05-03T00:00:00Z"
}
```

Generated on first boot if missing. Reloaded on every subsequent boot. Startup fails if file is present but malformed.

## Learning Focus

- Trait design: `PlacementEngine` as a seam, not just an interface
- Dependency injection: `NodeDescriptor` flows through `AppState` into service layer
- The difference between "stub" and "real but trivial" implementation
- Preparing data models for multi-node without implementing multi-node

## Done When

```bash
docker compose down -v && docker compose up --build
# node.json created with stable UUID
curl localhost:8080/debug/node
# → shows real node_id matching node.json

curl -X PUT --data-binary @cat.jpg localhost:8080/objects/photos/cat.jpg
curl localhost:8080/debug/objects/photos/cat.jpg
# → chunks show correct node_id matching debug/node

docker compose down && docker compose up
# node_id unchanged (loaded from node.json, not regenerated)
```
