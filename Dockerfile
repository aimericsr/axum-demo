FROM --platform=$BUILDPLATFORM tonistiigi/xx:1.6.1 AS xx

FROM --platform=$BUILDPLATFORM rust:1.81-slim-bookworm AS build
COPY --from=xx / /

RUN apt update && apt install -y \
    clang lld pkg-config file cmake curl git 

RUN apt install -y \
    gcc-x86-64-linux-gnu g++-x86-64-linux-gnu \
    gcc-aarch64-linux-gnu g++-aarch64-linux-gnu \
    gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf \
    gcc-s390x-linux-gnu g++-s390x-linux-gnu \
    gcc-powerpc64le-linux-gnu g++-powerpc64le-linux-gnu \
    gcc-12-i686-linux-gnu g++-12-i686-linux-gnu

RUN dpkg --add-architecture amd64
RUN dpkg --add-architecture arm64
RUN dpkg --add-architecture armhf
RUN dpkg --add-architecture s390x
RUN dpkg --add-architecture ppc64el
RUN dpkg --add-architecture i686

RUN apt update
RUN apt install -y \
    libssl-dev:amd64 libssl-dev:arm64 libssl-dev:armhf \
    libssl-dev:s390x libssl-dev:ppc64el libssl-dev:i686

RUN rustup target add \
    x86_64-unknown-linux-gnu \   
    aarch64-unknown-linux-gnu \ 
    armv7-unknown-linux-gnueabihf \ 
    s390x-unknown-linux-gnu \ 
    powerpc64le-unknown-linux-gnu \
    i686-unknown-linux-gnu 

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/x86_64-linux-gnu-gcc
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/aarch64-linux-gnu-gcc
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=/usr/bin/arm-linux-gnueabihf-gcc
ENV CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/s390x-linux-gnu-gcc
ENV CARGO_TARGET_POWERPC64LE_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/powerpc64le-linux-gnu-gcc
ENV CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/i686-linux-gnu-gcc

WORKDIR /app
COPY . .

ARG TARGETPLATFORM
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
ENTRYPOINT ["axum-demo"]
