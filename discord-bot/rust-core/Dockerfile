# Build aşaması
FROM rust:1.78-slim AS builder

WORKDIR /app

# Bağımlılıkları önce kopyala (cache optimizasyonu)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/discord_bot*

# Gerçek kaynak kodunu kopyala
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

# Çalışma aşaması (minimal imaj)
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/discord-bot .
COPY --from=builder /app/migrations ./migrations

CMD ["./discord-bot"]
