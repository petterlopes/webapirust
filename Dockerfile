# syntax=docker/dockerfile:1

FROM rust:1.90 as builder
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev libpq-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
COPY Cargo.lock ./
COPY configuration ./configuration
COPY migrations ./migrations
COPY src ./src
COPY tests ./tests

RUN cargo fetch --locked
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/webrust /usr/local/bin/webrust
COPY configuration ./configuration
COPY migrations ./migrations

EXPOSE 8080
CMD ["webrust"]
