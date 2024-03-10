# Build Stage
FROM rust:1.74.0 AS builder
WORKDIR /usr/src/

RUN USER=root cargo new homers
WORKDIR /usr/src/homers
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

RUN cargo install --path .

# Bundle Stage
FROM debian:trixie-slim
WORKDIR /app
ENV ROCKET_ADDRESS=0.0.0.0
COPY --from=builder /usr/local/cargo/bin/homers /usr/local/bin
COPY config.toml /app/config.toml
RUN apt-get update && \
    apt-get install -y sqlite3 ca-certificates
USER 1000
CMD ["homers", "--config", "config.toml"]
