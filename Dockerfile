FROM rust:1.70-bookworm as builder

COPY Cargo.lock /src/
COPY Cargo.toml /src/
RUN mkdir /src/src
RUN echo "fn main() {}" > /src/src/main.rs
WORKDIR /src

RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build

COPY src /src/src

RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build