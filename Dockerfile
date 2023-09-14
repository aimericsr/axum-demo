# FROM lukemathwalker/cargo-chef:latest-rust-1.72.0 as chef
# WORKDIR /app
# RUN apt update && apt install lld clang -y

# FROM chef as planner
# COPY . .
# # Compute a lock-like file for our project
# RUN cargo chef prepare --recipe-path recipe.json

# FROM chef as builder
# COPY --from=planner /app/recipe.json recipe.json
# # Build our project dependencies, not our application!
# RUN cargo chef cook --release --recipe-path recipe.json
# # Up to this point, if our dependency tree stays the same,
# # all layers should be cached.
# COPY . .
# ENV SQLX_OFFLINE true
# # Build our project
# RUN cargo build --release --bin axum-demo

# #FROM debian:stable-slim AS runtime
# FROM alpine:3.17.1 AS runtime
# WORKDIR /app

# RUN apk update && apk upgrade --no-cache

# # RUN apt-get update -y \
# #     && apt-get install -y --no-install-recommends openssl ca-certificates \
# #  #Clean up
# #     && apt-get autoremove -y \
# #     && apt-get clean -y \
# #     && rm -rf /var/lib/apt/lists/*

# COPY --from=builder /app/target/release/axum-demo axum-demo
# #COPY .cargo/config.toml .cargo/config.toml
# #ENV APP_ENVIRONMENT production
# ENV SERVICE_WEB_FOLDER static
# CMD ls
# ENTRYPOINT ["./app/axum-demo"]




FROM clux/muslrust:stable AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin axum-demo

FROM alpine AS runtime
RUN addgroup -S myuser && adduser -S myuser -G myuser
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/axum-demo /usr/local/bin/
# COPY configuration configuration
# COPY firebase-key.json firebase-key.json
# ENV APP_ENVIRONMENT production
COPY .cargo/config.toml /usr/local/bin/.cargo/config.toml
ENV SERVICE_WEB_FOLDER static
ENV RUST_LOG=debug 
USER myuser
CMD ["/usr/local/bin/axum-demo"]