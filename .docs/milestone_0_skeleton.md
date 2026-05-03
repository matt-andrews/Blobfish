# Milestone 0: Skeleton

**Goal:** Prove the Rust project shape. Nothing persists yet — just wire the skeleton together and confirm the whole thing builds and runs in Docker.

## Deliverables

- [x] Cargo workspace with all crate stubs (`blobfish-server`, `blobfish-api`, `blobfish-core`, `blobfish-meta`, `blobfish-store`, `blobfish-node`, `blobfish-client`)
- [x] `blobfish serve` CLI command (clap)
- [x] Config loading from TOML (`dev.toml`)
- [x] `GET /healthz` → 200
- [x] Structured logging with `tracing` wired up (`RUST_LOG` respected)
- [x] `PlacementEngine` trait stub (returns hardcoded local node ID)
- [x] Dockerfile (multi-stage, `debian:bookworm-slim`)
- [x] `docker-compose.yml` (single node, volume mount for data + config)
- [x] `docker compose up --build` works end to end

## Learning Focus

- Rust workspace structure and inter-crate dependencies
- Axum basics: router, handler, state injection
- Config deserialization with `serde` + `toml`
- Error types with `thiserror`
- Multi-stage Dockerfile for Rust
- `RUST_LOG` / `tracing_subscriber` setup

## Config Shape (dev.toml)

```toml
[node]
cluster_id   = "dev-cluster"
node_id      = "auto"
bind_addr    = "0.0.0.0:8080"
storage_root = "/data/blobfish"

[storage]
chunk_size_bytes    = 8388608
staging_ttl_seconds = 86400

[metadata]
engine = "redb"
path   = "/data/blobfish/meta/blobfish.db"

[placement]
replication_factor = 1

[observability]
log_level = "debug"
```

## Done When

```bash
docker compose up --build
curl localhost:8080/healthz   # → 200 OK
```
