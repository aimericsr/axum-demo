# Statically linked : Alpine, Scratch
#FROM rust:1.82-alpine3.19 AS chef
#FROM lukemathwalker/cargo-chef:latest-rust-1.81.0-alpine3.20 AS chef
#RUN apt install -y x86_64-linux-gnu-gcc

# Build Stage
FROM clux/muslrust:1.81.0-stable AS builder
USER root
WORKDIR /app
COPY . .

# Define architecture mapping
ARG TARGETARCH
RUN case "$TARGETARCH" in \
    "amd64")  RUST_TARGET="x86_64-unknown-linux-musl" ;; \
    "arm64")  RUST_TARGET="aarch64-unknown-linux-musl" ;; \
    "arm/v7") RUST_TARGET="armv7-unknown-linux-musleabihf" ;; \
    *) echo "Unsupported architecture: $TARGETARCH" && exit 1 ;; \
    esac && \
    rustup target add $RUST_TARGET && \
    cargo install cargo-chef && \
    cargo chef prepare --recipe-path recipe.json && \
    cargo chef cook --release --target $RUST_TARGET --recipe-path recipe.json && \
    cargo build --release --target $RUST_TARGET --bin axum-demo

# Alpine Runtime Stage
FROM alpine:3.20.1 AS runtime
WORKDIR /usr/local/bin/
RUN addgroup -S myuser && adduser -S myuser -G myuser
ARG TARGETARCH
RUN case "$TARGETARCH" in \
    "amd64")  RUST_TARGET="x86_64-unknown-linux-musl" ;; \
    "arm64")  RUST_TARGET="aarch64-unknown-linux-musl" ;; \
    "arm/v7") RUST_TARGET="armv7-unknown-linux-musleabihf" ;; \
    *) echo "Unsupported architecture: $TARGETARCH" && exit 1 ;; \
    esac
COPY --from=builder /app/target/$RUST_TARGET/release/axum-demo .
COPY .env .
USER myuser
CMD ["./axum-demo"]

# Scratch Runtime (Alternative)
# FROM scratch AS runtime
# RUN apk add --no-cache tzdata
# ENV TZ=Europe/Paris
