FROM docker.io/library/rust:alpine as builder

WORKDIR /app

RUN --mount=type=cache,target=/var/cache/apk \
    apk add --cache-dir /var/cache/apk musl-dev

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release

FROM docker.io/library/alpine:latest

WORKDIR /app

RUN --mount=type=cache,target=/var/cache/apk \
    apk add --cache-dir /var/cache/apk ca-certificates

COPY --from=builder /app/target/release/cf-ddns /app/cf-ddns

CMD ["/app/cf-ddns", "service", "run"]
