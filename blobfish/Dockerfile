FROM rust:1 AS builder
WORKDIR /app
# Cache dependency compilation separately from application code.
# Copy manifests first, build a dummy main, then copy src.
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release --bin blobfish

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -m blobfish
RUN mkdir -p /data/blobfish/chunks && chown -R blobfish:blobfish /data/blobfish
RUN mkdir -p /data/blobfish/perm && chown -R blobfish:blobfish /data/blobfish
USER blobfish
WORKDIR /app
COPY --from=builder /app/target/release/blobfish /usr/local/bin/blobfish
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/healthz || exit 1
CMD ["blobfish", "serve", "--config", "/etc/blobfish/config.toml"]