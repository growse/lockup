FROM rust:1.72-bookworm as builder

COPY Cargo.lock /build/
COPY Cargo.toml /build/
COPY .cargo /build/
RUN mkdir /build/src
RUN echo "fn main() {}" > /build/src/main.rs
WORKDIR /build

RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --target x86_64-unknown-linux-gnu

COPY src /build/src

RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --target x86_64-unknown-linux-gnu

FROM alpine:latest

COPY --from=builder /build/./target/x86_64-unknown-linux-gnu/debug/lockup /lockup
CMD ["/lockup"]