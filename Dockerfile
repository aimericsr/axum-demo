FROM --platform=$BUILDPLATFORM tonistiigi/xx:1.6.1 AS xx

FROM --platform=$BUILDPLATFORM rust:1.81-slim-bookworm AS build
COPY --from=xx / /

WORKDIR /app
COPY . .

RUN apt update && apt install -y \
    clang lld file cmake curl git pkg-config libssl-dev

ARG TARGETPLATFORM
RUN TARGET=$(xx-cargo --print-target-triple) && \
    case "$TARGET" in \
        "x86_64-unknown-linux-gnu") \
            dpkg --add-architecture amd64 && \
            apt update && \
            apt install -y gcc-x86-64-linux-gnu g++-x86-64-linux-gnu libssl-dev:amd64 && \
            rustup target add x86_64-unknown-linux-gnu ;; \
        "aarch64-unknown-linux-gnu") \
            dpkg --add-architecture arm64 && \
            apt update && \
            apt install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libssl-dev:arm64 && \
            rustup target add aarch64-unknown-linux-gnu ;; \
        "armv7-unknown-linux-gnueabihf") \
            dpkg --add-architecture armhf && \
            apt update && \
            apt install -y gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf libssl-dev:armhf && \
            rustup target add armv7-unknown-linux-gnueabihf ;; \
        "s390x-unknown-linux-gnu") \
            dpkg --add-architecture s390x && \
            apt update && \
            apt install -y gcc-s390x-linux-gnu g++-s390x-linux-gnu libssl-dev:s390x && \
            rustup target add s390x-unknown-linux-gnu ;; \
        "powerpc64le-unknown-linux-gnu") \
            dpkg --add-architecture ppc64el && \
            apt update && \
            apt install -y gcc-powerpc64le-linux-gnu g++-powerpc64le-linux-gnu libssl-dev:ppc64el && \
            rustup target add powerpc64le-unknown-linux-gnu ;; \
        "i686-unknown-linux-gnu") \
            dpkg --add-architecture i386 && \
            apt update && \
            apt install -y gcc-i686-linux-gnu g++-i686-linux-gnu libssl-dev:i386 && \
            rustup target add i686-unknown-linux-gnu ;; \
        *) echo "No matching packages for $TARGET"; exit 1 ;; \
    esac

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/x86_64-linux-gnu-gcc
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/aarch64-linux-gnu-gcc
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=/usr/bin/arm-linux-gnueabihf-gcc
ENV CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/s390x-linux-gnu-gcc
ENV CARGO_TARGET_POWERPC64LE_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/powerpc64le-linux-gnu-gcc
ENV CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/i686-linux-gnu-gcc

RUN cargo fetch
RUN cargo build --target=$(xx-cargo --print-target-triple)
RUN mkdir build && \
mv target/$(xx-cargo --print-target-triple)/debug/axum-demo build
RUN xx-verify ./build/axum-demo

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY .env .
COPY --from=build /app/build/axum-demo /
ENTRYPOINT ["./axum-demo"]
