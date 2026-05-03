# Milestone 10: Erasure Coding Experiment

**Goal:** Learn the storage efficiency and recovery tradeoffs of Reed-Solomon erasure coding versus replication. This is an experiment behind a feature flag — not a replacement for replication.

## Deliverables

- [ ] Reed-Solomon encode/decode behind `--features erasure` or a config flag
- [ ] Object encoded into `k` data shards + `m` parity shards
- [ ] Each shard stored as a chunk on a different node (requires M7 multi-node write path)
- [ ] Object reconstructable from any `k` of `k + m` shards
- [ ] `ObjectManifest` extended to record erasure coding parameters (`k`, `m`, shard index per chunk)
- [ ] `GET /debug/objects` shows erasure coding layout
- [ ] Benchmark: compare storage overhead of `replication_factor = 3` vs RS(6,3)
- [ ] Recovery path: if fewer than `m` shards are missing, reconstruct and re-encode missing shards

## Suggested Configuration

```toml
[erasure]
enabled = false   # feature flag
data_shards   = 6
parity_shards = 3
# Requires at least k+m = 9 nodes to be meaningful
```

## Manifest Extension

```rust
enum StorageScheme {
    Replicated { factor: u8 },
    ErasureCoded { data_shards: u8, parity_shards: u8 },
}

struct ObjectManifest {
    // ... existing fields ...
    storage_scheme: StorageScheme,
    // chunks Vec<ChunkDescriptor> now has one entry per shard
    // ChunkDescriptor.ordinal = shard index (0..k for data, k..k+m for parity)
}
```

## Suggested Crate

`reed-solomon-erasure` (pure Rust, available on crates.io).

## Encode Path

1. Buffer the full object (or stream into a memory buffer — noting the in-memory cost).
2. Split into `k` equal-sized data shards (pad last shard to equal size).
3. Compute `m` parity shards.
4. Write all `k + m` shards as chunks, one per node (via placement engine + internal chunk PUT).
5. Store shard index in each `ChunkDescriptor.ordinal`.

## Decode Path

1. Load available shards from their respective nodes.
2. Mark missing shards as absent.
3. If missing count ≤ `m`: reconstruct via Reed-Solomon decode.
4. Concatenate data shards (strip padding from last shard) → original bytes.
5. If missing count > `m`: return 500, log as `object_unrecoverable`.

## Comparison Exercise

After implementation, run:
- Store 1 GiB of objects with `replication_factor = 3` → measure total disk usage.
- Store the same 1 GiB with RS(6,3) → measure total disk usage.
- Kill 1 node → measure recovery time for each scheme.
- Kill 3 nodes → observe that replication(3) is now unrecoverable, RS(6,3) is still recoverable.

## Learning Focus

- Erasure coding mathematics (data/parity tradeoff)
- Shard-level placement across failure domains
- Memory vs streaming tradeoffs in encoding (large objects require buffering or block-level encoding)
- Why erasure coding and replication solve different problems

## Done When

```bash
# With erasure = true and 9 nodes (or 9 simulated volumes)
curl -X PUT --data-binary @100mb.bin localhost:8081/objects/test/100mb.bin

# Inspect: 6 data shards + 3 parity shards spread across 9 nodes
curl localhost:8081/debug/objects/test/100mb.bin

# Kill 2 nodes (rm their chunk files)
# Object still readable:
curl localhost:8081/objects/test/100mb.bin -o recovered.bin
diff 100mb.bin recovered.bin   # identical

# Kill a 3rd node
# Object still readable (RS(6,3) tolerates up to 3 failures)
```
