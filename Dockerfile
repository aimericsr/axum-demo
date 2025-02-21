# Build Stage
FROM clux/muslrust:1.81.0-stable AS builder
USER root
WORKDIR /app
COPY . .

ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
        "linux/amd64")  echo "x86_64-unknown-linux-musl"  >> /tmp/target ;; \
        "linux/arm64")  echo "aarch64-unknown-linux-musl"  >> /tmp/target ;; \
        "linux/arm/v7") echo "armv7-unknown-linux-musleabihf" >> /tmp/target ;; \
        *)             echo "Unsupported TARGETPLATFORM: $TARGETPLATFORM" && exit 1 ;; \
    esac

RUN rustup target add $(cat /tmp/target)
RUN cargo install cargo-chef
RUN cargo chef prepare --recipe-path recipe.json  
RUN cargo chef cook --release --target $(cat /tmp/target) --recipe-path recipe.json
RUN cargo build --release --target $(cat /tmp/target) --bin axum-demo
RUN mkdir -p target/common/release && \
    mv target/$(cat /tmp/target)/release/axum-demo target/common/release/axum-demo

# Runtime Stage
FROM alpine:3.21 AS runtime
WORKDIR /usr/local/bin/
RUN addgroup -S myuser && adduser -S myuser -G myuser
RUN apk add --no-cache tzdata
ENV TZ=Europe/Paris
COPY --from=builder /app/target/common/release/axum-demo .
COPY .env .
USER myuser
CMD ["./axum-demo"]

# Statically linked : Alpine, Scratch
#FROM rust:1.82-alpine3.19 AS chef
#FROM lukemathwalker/cargo-chef:latest-rust-1.81.0-alpine3.20 AS chef
#RUN apt install -y x86_64-linux-gnu-gcc

# Scratch Runtime (Alternative)
# FROM scratch AS runtime
# RUN apk add --no-cache tzdata
# ENV TZ=Europe/Paris
