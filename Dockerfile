# Install cargo-chef
FROM clux/muslrust:1.76.0 AS chef
USER root
RUN cargo install cargo-chef
#RUN cargo install cargo-chef@0.1.63
WORKDIR /app

# Install dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build the binary
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin axum-demo

# Alpine 
# FROM alpine:3.18.4 AS runtime
# RUN addgroup -S myuser && adduser -S myuser -G myuser
# COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/axum-demo /usr/local/bin/
# COPY .env /usr/local/bin/
# USER myuser
# WORKDIR "/usr/local/bin/"
# CMD ["./axum-demo"]

# Scratch
FROM scratch AS runtime
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/axum-demo /usr/local/bin/
WORKDIR "/usr/local/bin/"
CMD ["./axum-demo"]


