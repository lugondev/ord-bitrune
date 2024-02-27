FROM rust:1.75.0-bookworm as builder

WORKDIR /usr/src/ord

COPY . .

RUN cargo build --bin ord --release

FROM debian:bookworm-slim

COPY --from=builder /usr/src/ord/target/release/ord /usr/local/bin

RUN apt update && apt-get install libssl-dev -y

ENV RUST_BACKTRACE=1
ENV RUST_LOG=info
