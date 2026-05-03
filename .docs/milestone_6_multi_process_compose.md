# Milestone 6: Multi-Process Compose Without Replication

**Goal:** Run several independent Blobfish nodes side by side. No cross-node writes yet — each node is a fully isolated single-node store. The point is to prove the operational shape works and to observe distributed system behaviour before implementing it.

## Deliverables

- [ ] `docker-compose.multi.yml` with 3 named nodes (`blobfish-a`, `blobfish-b`, `blobfish-c`)
- [ ] Each node has its own named Docker volume for data
- [ ] Each node has its own config file with a unique `node_id` (or `"auto"` each generating its own)
- [ ] Each node exposed on a different host port (e.g., 8081, 8082, 8083)
- [ ] Nodes can list their statically-configured peers via `GET /debug/node` (peers list, even if unused)
- [ ] `GET /healthz` and `GET /readyz` work independently on each node
- [ ] Losing one node's volume does not affect the others

## docker-compose.multi.yml Sketch

```yaml
services:
  blobfish-a:
    build: .
    ports:
      - "8081:8080"
    volumes:
      - blobfish-a-data:/data/blobfish
      - ./models/node-a.toml:/etc/blobfish/models.toml:ro
    environment:
      RUST_LOG: blobfish=debug,tower_http=info

  blobfish-b:
    build: .
    ports:
      - "8082:8080"
    volumes:
      - blobfish-b-data:/data/blobfish
      - ./models/node-b.toml:/etc/blobfish/models.toml:ro
    environment:
      RUST_LOG: blobfish=debug,tower_http=info

  blobfish-c:
    build: .
    ports:
      - "8083:8080"
    volumes:
      - blobfish-c-data:/data/blobfish
      - ./models/node-c.toml:/etc/blobfish/models.toml:ro
    environment:
      RUST_LOG: blobfish=debug,tower_http=info

volumes:
  blobfish-a-data:
  blobfish-b-data:
  blobfish-c-data:
```

## Config Addition: Static Peers

```toml
[cluster]
peers = [
  "http://blobfish-b:8080",
  "http://blobfish-c:8080",
]
```

Peers are informational for now — no RPC calls made. They appear in `GET /debug/node`.

## `GET /debug/node` Addition

```json
{
  "node_id": "...",
  "cluster_id": "dev-cluster",
  "address": "blobfish-a:8080",
  "storage_root": "/data/blobfish",
  "peers": ["http://blobfish-b:8080", "http://blobfish-c:8080"],
  "status": "Ready"
}
```

## Learning Focus

- Docker Compose networking and service discovery by hostname
- Multiple bind mounts for per-node config files
- Volume isolation between containers
- What "independent nodes" looks like before replication is added
- Container hostname resolution (`blobfish-b:8080` is valid inside the compose network)

## Done When

```bash
docker compose -f docker-compose.multi.yml up --build

# Each node independent
curl localhost:8081/debug/node   # node-a, peers: b, c
curl localhost:8082/debug/node   # node-b, peers: a, c
curl localhost:8083/debug/node   # node-c, peers: a, b

# Objects stored to one node are NOT on others (no replication yet)
curl -X PUT --data-binary @cat.jpg localhost:8081/objects/photos/cat.jpg
curl localhost:8081/objects/photos/cat.jpg   # 200
curl localhost:8082/objects/photos/cat.jpg   # 404

# Node isolation: nuke node-a volume, others unaffected
docker compose -f docker-compose.multi.yml down
docker volume rm blobfish_blobfish-a-data
docker compose -f docker-compose.multi.yml up
curl localhost:8082/objects/photos/cat.jpg   # still 404 (was never there)
```
