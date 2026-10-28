ARG RUST_IMAGE=rust:1.92-slim-bookworm
FROM node:24-bookworm-slim AS web
WORKDIR /build/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run lint && npm test && npm run build

FROM ${RUST_IMAGE} AS backend
RUN apt-get update && apt-get install -y --no-install-recommends build-essential pkg-config ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY migrations/ migrations/
RUN cargo build --locked --release -j 2 -p orisema-data-server \
    && install -Dm755 target/release/orisema-data-server /out/orisema-data-server \
    && rm -rf /build/target /usr/local/cargo/registry

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 --create-home orisema-data
WORKDIR /app
COPY --from=backend /out/orisema-data-server /usr/local/bin/orisema-data-server
COPY --from=web /build/frontend/dist /app/frontend/dist
COPY LICENSE /app/LICENSE
USER 10001
ENV RUSTDB_BIND_ADDRESS=0.0.0.0 RUSTDB_PORT=8080 RUSTDB_STATIC_DIR=/app/frontend/dist
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s CMD curl --fail --silent http://127.0.0.1:8080/api/v1/health || exit 1
CMD ["orisema-data-server"]
