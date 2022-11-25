FROM rust:latest as builder

WORKDIR /cf-ddns
COPY . .

RUN cargo install --path .

FROM debian:stable-slim

COPY --from=builder /usr/local/cargo/bin/cf-ddns /cf-ddns/cf-ddns

CMD ["/cf-ddns/cf-ddns"]

VOLUME /cf-ddns/config.json
