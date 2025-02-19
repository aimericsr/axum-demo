# Statically linked : Alpine, Scratch
#FROM rust:1.82-alpine3.19 AS chef
#FROM lukemathwalker/cargo-chef:latest-rust-1.81.0-alpine3.20 AS chef
#RUN apt install -y x86_64-linux-gnu-gcc
FROM clux/muslrust:1.81.0-stable AS builder
USER root
WORKDIR /app
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install cargo-chef
RUN cargo chef prepare --recipe-path recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
RUN cargo build --release --target x86_64-unknown-linux-musl --bin axum-demo

# Alpine 
#RUN apk add --no-cache tzdata
#ENV TZ=Europe/Paris
FROM alpine:3.20.1 AS runtime
WORKDIR /usr/local/bin/
RUN addgroup -S myuser && adduser -S myuser -G myuser
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/axum-demo .
COPY .env .
USER myuser
CMD ["./axum-demo"]

# # # Scratch
# # FROM scratch AS runtime
# # WORKDIR "/usr/local/bin/"
# # COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/axum-demo .
# # COPY .env .
# # ENV TZ=Europe/Paris
# # CMD ["./axum-demo"]


