FROM rust:1.82 AS builder

# Update package lists
RUN apt-get update

RUN apt-get install -y libdbus-1-dev pkg-config

WORKDIR /usr/src/app

COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
# FROM alpine:latest

WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/target/release/meshtastic-to-influx .

# Install required runtime dependencies
# RUN apk add --no-cache libgcc

CMD ["./meshtastic-to-influx"]
