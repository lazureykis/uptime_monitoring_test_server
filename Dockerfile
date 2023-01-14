# Rust as the base image
FROM rust:1 AS builder
WORKDIR /build
COPY . .

ARG service

RUN cargo build --release -j 4

FROM debian:11-slim

WORKDIR /app
COPY --from=builder /build/target/release/uptime_monitoring_test_server /app/

EXPOSE 5555

CMD /app/uptime_monitoring_test_server
