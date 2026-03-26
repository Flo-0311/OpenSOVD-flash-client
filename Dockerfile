FROM rust:latest AS builder

WORKDIR /app
COPY . .


RUN cargo build --release -p sovd-cli


FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sovd-flash /usr/local/bin/sovd-flash

ENTRYPOINT ["sovd-flash"]