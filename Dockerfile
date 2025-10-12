# Build Stage
FROM rust:1.89.0 AS builder
WORKDIR /usr/src/

RUN USER=root cargo new homers
WORKDIR /usr/src/homers
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo install --locked --path .

# Bundle Stage
FROM debian:trixie-slim
WORKDIR /app
ENV ROCKET_ADDRESS=0.0.0.0
ENV TZ="Europe/Paris"
ENV USER="homers"
ARG S6_OVERLAY_VERSION=3.2.0.2




COPY --from=builder /usr/local/cargo/bin/homers /usr/local/bin
RUN apt-get update && \
    apt-get install xz-utils -y && \
    apt-get install -y sqlite3 ca-certificates && \
    cp "/usr/share/zoneinfo/${TZ}" /etc/localtime && \
    echo "${TZ}" > /etc/timezone
COPY config.toml /app/config.toml
COPY entrypoint.sh /app/entrypoint.sh
COPY root /
# add s6 overlay
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-noarch.tar.xz /tmp
RUN tar -C / -Jxpf /tmp/s6-overlay-noarch.tar.xz
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-x86_64.tar.xz /tmp
RUN tar -C / -Jxpf /tmp/s6-overlay-x86_64.tar.xz
USER root

ENTRYPOINT ["/init"]
