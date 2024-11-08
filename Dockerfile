# Build Stage
FROM rust:1.77.2 AS builder
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
ENV TZ="Europe/Paris"
ENV USER="homers"

COPY --from=builder /usr/local/cargo/bin/homers /usr/local/bin
COPY config.toml /app/config.toml
COPY entrypoint.sh /app/entrypoint.sh
RUN apt-get update && \
    apt-get install -y sqlite3 ca-certificates && \
    cp "/usr/share/zoneinfo/${TZ}" /etc/localtime && \ 
    echo "${TZ}" > /etc/timezone
USER root
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["homers", "--config", "config.toml"]
