# Build from the repository root:
#   docker build -f archive_frontend/Dockerfile -t lr2ir-archive-frontend .
#
# Run (mount your database files read-only):
#   docker run --rm -p 3000:3000 \
#     -v "$PWD/lr2ir-archive.db:/data/lr2ir-archive.db:ro" \
#     -v "$PWD/tableinfo.db:/data/tableinfo.db:ro" \
#     lr2ir-archive-frontend

FROM rust:1.95-alpine@sha256:606fd313a0f49743ee2a7bd49a0914bab7deedb12791f3a846a34a4711db7ed2 AS builder

WORKDIR /app

# libsqlite3-sys (bundled via sqlx) compiles SQLite from source.
RUN apk add --no-cache musl-dev build-base

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY archive_frontend ./archive_frontend
COPY archive_frontend_tablegen ./archive_frontend_tablegen
COPY archive_parser ./archive_parser

RUN cargo build --release -p lr2ir_archive_frontend

# The actual runtime image is just from linux-from-scratch, get as small as possible.
FROM scratch

ARG OCI_VERSION=0.1.0-dev
ARG OCI_REVISION=unknown
ARG OCI_CREATED=1970-01-01T00:00:00.000Z

LABEL org.opencontainers.image.title="LR2IR Archive Frontend" \
	org.opencontainers.image.description="Read-only web archive for browsing the LR2IR dataset." \
	org.opencontainers.image.url="https://ghcr.io/zkldi/lr2ir-dataset/archive-frontend" \
	org.opencontainers.image.documentation="https://github.com/zkldi/lr2ir-dataset/blob/main/archive_frontend/README.md" \
	org.opencontainers.image.source="https://github.com/zkldi/lr2ir-dataset" \
	org.opencontainers.image.version="${OCI_VERSION}" \
	org.opencontainers.image.revision="${OCI_REVISION}" \
	org.opencontainers.image.created="${OCI_CREATED}" \
	org.opencontainers.image.licenses="CC0-1.0" \
	org.opencontainers.image.vendor="zkldi" \
	org.opencontainers.image.authors="https://github.com/zkldi" \
	org.opencontainers.image.ref.name="archive-frontend" \
	org.opencontainers.image.base.name="scratch"

COPY --from=builder /app/target/release/lr2ir_archive_frontend /lr2ir_archive_frontend

ENV BIND=0.0.0.0:3000 \
	DATABASE_PATH=/data/lr2ir-archive.db \
	TABLEINFO_PATH=/data/tableinfo.db

EXPOSE 3000

ENTRYPOINT ["/lr2ir_archive_frontend"]
CMD ["serve"]
