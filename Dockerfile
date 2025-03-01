FROM --platform=$BUILDPLATFORM tonistiigi/xx:1.6.1 AS xx

FROM --platform=$BUILDPLATFORM rust:1.85-slim-bookworm AS build
COPY --from=xx / /

WORKDIR /app
COPY . .

RUN apt update && apt install -y --no-install-recommends clang file cmake curl git pkg-config libssl-dev binutils-common musl-tools lld

ARG TARGETPLATFORM
RUN TARGET=$(xx-cargo --print-target-triple) && \
    case "$TARGET" in \
        "x86_64-unknown-linux-gnu") \
            dpkg --add-architecture amd64 && \
            apt update && \
            apt install -y --no-install-recommends gcc-x86-64-linux-gnu g++-x86-64-linux-gnu libssl-dev:amd64 && \
            rustup target add x86_64-unknown-linux-gnu ;; \
        "aarch64-unknown-linux-gnu") \
            dpkg --add-architecture arm64 && \
            apt update && \
            apt install -y --no-install-recommends gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libssl-dev:arm64 && \
            rustup target add aarch64-unknown-linux-gnu ;; \
        "armv7-unknown-linux-gnueabihf") \
            dpkg --add-architecture armhf && \
            apt update && \
            apt install -y --no-install-recommends gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf libssl-dev:armhf && \
            rustup target add armv7-unknown-linux-gnueabihf ;; \
        "s390x-unknown-linux-gnu") \
            dpkg --add-architecture s390x && \
            apt update && \
            apt install -y --no-install-recommends gcc-s390x-linux-gnu g++-s390x-linux-gnu libssl-dev:s390x && \
            rustup target add s390x-unknown-linux-gnu ;; \
        "powerpc64le-unknown-linux-gnu") \
            dpkg --add-architecture ppc64el && \
            apt update && \
            apt install -y --no-install-recommends gcc-powerpc64le-linux-gnu g++-powerpc64le-linux-gnu libssl-dev:ppc64el && \
            rustup target add powerpc64le-unknown-linux-gnu ;; \
        "i686-unknown-linux-gnu") \
            dpkg --add-architecture i386 && \
            apt update && \
            apt install -y --no-install-recommends gcc-i686-linux-gnu g++-i686-linux-gnu libssl-dev:i386 && \
            rustup target add i686-unknown-linux-gnu ;; \
        *) echo "No matching packages for $TARGET"; exit 1 ;; \
    esac

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/x86_64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/x86_64-linux-gnu-gcc
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/aarch64-linux-gnu-gcc
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=/usr/bin/arm-linux-gnueabihf-gcc
ENV CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/s390x-linux-gnu-gcc
ENV CARGO_TARGET_POWERPC64LE_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/powerpc64le-linux-gnu-gcc
ENV CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/i686-linux-gnu-gcc

RUN cargo fetch
RUN cargo build --release --target=$(xx-cargo --print-target-triple)
RUN mkdir build && \
mv target/$(xx-cargo --print-target-triple)/release/axum-demo build
RUN xx-verify ./build/axum-demo

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY .env .
COPY --from=build --chmod=755 /app/build/axum-demo .
ENTRYPOINT ["./axum-demo"]

# Alpine static build
# FROM --platform=$BUILDPLATFORM rust:alpine3.21 AS build
# COPY --from=xx / /
# ARG TARGETPLATFORM

# WORKDIR /app
# COPY . .

# RUN apk add mold musl-dev pkgconfig
# RUN xx-apk add openssl-dev openssl-libs-static
# ENTRYPOINT [ "/bin/sh" ]
# RUN rustup target add $(xx-cargo --print-target-triple)
# ENV PKG_CONFIG_ALLOW_CROSS=1
# ENV PKG_CONFIG_ALL_STATIC=true
# ENV RUSTFLAGS="-Clink-arg=-fuse-ld=mold" 
# ENV OPENSSL_STATIC=true
# ENV OPENSSL_DIR=/x86_64-alpine-linux-musl/usr
# x86_64-unknown-linux-musl/usr/lib
#CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/x86_64-linux-gnu-gcc
# RUN OPENSSL_DIR=/x86_64-alpine-linux-musl/usr cargo build --target x86_64-unknown-linux-musl
# RUN mkdir build && \
#     mv target/$(xx-cargo --print-target-triple)/debug/axum-demo build
# RUN xx-verify ./build/axum-demo

# FROM alpine:3.21 AS runtime
# RUN apk add libssl3
# WORKDIR /app
# COPY .env .
# COPY --from=build --chmod=755 /app/build/axum-demo .
# ENTRYPOINT ["./axum-demo"]

# ENTRYPOINT [ "/bin/bash" ]