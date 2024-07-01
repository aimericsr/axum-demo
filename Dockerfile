# Install cargo-chef
FROM clux/muslrust:1.76.0 AS chef
USER root
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install cargo-chef
#RUN cargo install cargo-chef@0.1.63
WORKDIR /app
#COPY /Users/aimericsorin/Documents/Techno/Rust/tower-otel .

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
FROM alpine:3.20.1 AS runtime
WORKDIR "/usr/local/bin/"
#RUN apk add --no-cache tzdata
#ENV TZ=Europe/Paris
RUN addgroup -S myuser && adduser -S myuser -G myuser
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/axum-demo .
COPY .env .
USER myuser
CMD ["./axum-demo"]

# # Scratch
# FROM scratch AS runtime
# WORKDIR "/usr/local/bin/"
# COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/axum-demo .
# COPY .env .
# ENV TZ=Europe/Paris
# CMD ["./axum-demo"]


