# ======== Build stage (Alpine + musl) ========
FROM rust:1.90-alpine AS builder

# Install needed build tools
RUN apk add --no-cache musl-dev openssl-dev pkgconfig curl

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build release binary (server only)
RUN cargo build --bin server --release

# ======== Runtime stage (minimal Alpine) ========
FROM alpine:3.20

# SSL certs (needed for reqwest, rustls, or anything HTTPS)
RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy the musl-linked binary
COPY --from=builder /app/target/release/server /app/server

EXPOSE 3000

ENTRYPOINT ["/app/server"]