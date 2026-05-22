# syntax=docker/dockerfile:1

FROM node:24-alpine AS frontend-builder
WORKDIR /src
COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm npm ci
COPY index.html tsconfig.json vite.config.ts ./
COPY src ./src
COPY under-meintenance.en.svg under-meintenance.jp.svg ./
RUN npm run build

FROM rust:1-slim-bookworm AS server-builder
ENV PATH=/usr/local/cargo/bin:${PATH}
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates git \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src/server
COPY server/Cargo.toml ./Cargo.toml
COPY server/Cargo.lock ./Cargo.lock
COPY server/src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/server/target \
    cargo build --release \
    && cp /src/server/target/release/client-minihttp /tmp/client-minihttp

FROM debian:trixie-slim AS runtime
ARG BUILD_VERSION=""
ARG BUILD_COMMIT=""
LABEL org.opencontainers.image.version="${BUILD_VERSION}"
LABEL org.opencontainers.image.revision="${BUILD_COMMIT}"
RUN apt-get update \
    && apt-get install -y --no-install-recommends --only-upgrade \
        libc-bin \
        libc6 \
        libcap2 \
        libsystemd0 \
        libudev1 \
        sed \
    && rm -rf /var/lib/apt/lists/*
RUN useradd --system --uid 10001 --home /nonexistent --shell /usr/sbin/nologin minihttp
COPY --from=server-builder /tmp/client-minihttp /usr/local/bin/client-minihttp
COPY --from=frontend-builder /src/dist /var/www/html
USER minihttp
ENV BIND=0.0.0.0:8080
ENV WEB_ROOT=/var/www/html
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/client-minihttp"]
