# UbiHome Builder image (esphome-builder style).
#
# Fully decoupled from the UbiHome repository: the image contains only the
# builder + the Rust toolchain. It CLONES UbiHome on demand at runtime and builds
# any tagged version in an isolated git worktree. Build context is `builder/`:
#   docker build -f builder/Dockerfile -t ubihome-builder builder
# (docker-compose.yml sets this automatically.)
#
# The runtime image carries the Rust toolchain because binaries are compiled at
# request time — that toolchain is the bulk of the size. Sources, caches and the
# clone live in the /cache volume, keeping the image itself lean.

# ---- Stage 1: build the Angular dashboard ----------------------------------
FROM node:22-bookworm-slim AS frontend
WORKDIR /fe
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build
# → /fe/dist/ubihome-builder/browser

# ---- Stage 2: build the builder server (embeds the SPA) --------------------
FROM rust:1-slim-bookworm AS backend
WORKDIR /b
COPY Cargo.toml ./
COPY engine ./engine
COPY cli ./cli
COPY server ./server
COPY --from=frontend /fe/dist/ubihome-builder/browser ./frontend/dist/ubihome-builder/browser
RUN cargo build --release -p ubihome-builder-server

# ---- Stage 3: runtime ------------------------------------------------------
FROM rust:1-slim-bookworm AS runtime
# git: clone UbiHome at runtime. dev libs: needed to compile UbiHome's components.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       pkg-config libdbus-1-dev libasound2-dev git ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=backend /b/target/release/ubihome-builder-server /usr/local/bin/ubihome-builder-server

ENV BUILDER_DATA=/data \
    BUILDER_WORK=/cache \
    BUILDER_BIND=0.0.0.0:8080 \
    BUILDER_REPO_URL=https://github.com/UbiHome/UbiHome.git

VOLUME ["/data", "/cache"]
EXPOSE 8080
CMD ["ubihome-builder-server"]
