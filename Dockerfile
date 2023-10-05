# Builder stage
FROM rust:alpine3.17 AS builder
WORKDIR /app
RUN apk update
RUN apk add --no-cache lld clang musl-dev openssl-dev pkgconf openssl
COPY . .

# Set environment variables for OpenSSL static link
ENV SQLX_OFFLINE=true

RUN cargo build -p web_server --release --target=aarch64-unknown-linux-musl


# Runtime stage
FROM alpine:3 AS runtime
WORKDIR /app
COPY --from=builder /app/target/aarch64-unknown-linux-musl/release/web_server web_server
COPY configuration configuration
ENV APP_ENVIRONMENT local
ENTRYPOINT ["./web_server"]
