# syntax=docker/dockerfile:1
FROM node:20 as web

COPY web /app/web
WORKDIR /app/web/
RUN --mount=type=cache,target=/app/web/.npm npm ci --cache .npm --prefer-offline && npm run build

FROM rust:1.78-bookworm as builder

COPY Cargo.lock /build/
COPY Cargo.toml /build/
COPY .cargo/config.toml /build/.cargo/config.toml
RUN mkdir /build/src
RUN echo "fn main() {}" > /build/src/main.rs
WORKDIR /build

RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build

COPY src/ /build/src/
COPY .sqlx /build/.sqlx/
COPY migrations /build/migrations/

RUN rm -rf /build/target
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release

FROM scratch

COPY --from=builder /build/target/x86_64-unknown-linux-gnu/release/lockup /lockup
COPY templates/ /templates/
COPY --from=web /app/static /static
COPY Rocket.toml /Rocket.toml
VOLUME /data
CMD ["/lockup"]
